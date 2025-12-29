use crate::{AccessType, CPU, Device, DeviceData, SLMP4EConnectionProps, TypedData};
use crate::commands::{HEADER_BYTELEN, CPUTIMER_BYTELEN, COMMAND_PREFIX_BYTELEN};

const COMMAND_RANDOM_WRITE: u16 = 0x1402;

pub struct SLMPRandomWriteQuery<'a> {
    pub connection_props: &'a SLMP4EConnectionProps,
    pub sorted_data: &'a [DeviceData],
    pub access_type: AccessType,
    pub bit_access_points: u8,
    pub single_word_access_points: u8,
    pub double_word_access_points: u8,
}

pub struct SLMPRandomWriteCommand(pub Vec<u8>);
impl std::ops::Deref for SLMPRandomWriteCommand {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> TryFrom<SLMPRandomWriteQuery<'a>> for SLMPRandomWriteCommand {
    type Error = std::io::Error;
    fn try_from(value: SLMPRandomWriteQuery) -> Result<Self, Self::Error> {
        let cmd = construct_frame(value)?;
        Ok(Self(cmd))
    }
}

fn get_subcommand(cpu: CPU, access_type: AccessType) -> std::io::Result<[u8; 2]> {
    match access_type {
        AccessType::Bit => match cpu {
            CPU::Q | CPU::L => Ok([0x01, 0x00]),
            CPU::R => Ok([0x03, 0x00]),
            _ => Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Unsupported CPU"))
        },
        AccessType::Word => match cpu {
            CPU::Q | CPU::L => Ok([0x00, 0x00]),
            CPU::R => Ok([0x02, 0x00]),
            _ => Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Unsupported CPU"))
        }
    }
}

fn construct_frame(query: SLMPRandomWriteQuery) -> std::io::Result<Vec<u8>> {

    const SINGLE_WORD_BYTELEN: u8 = 2;
    const DOUBLE_WORD_BYTELEN: u8 = 4;
    let bit_bytelen = match query.connection_props.cpu {
        CPU::Q | CPU::L => 1,
        CPU::R => 2,
        _ => return Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Unsupported CPU"))
    };

    let device_addr_bytelen: u8 = Device::addr_code_len(query.connection_props.cpu)?;
    let bit_wreq_bytelen: u8 = device_addr_bytelen + bit_bytelen;
    let single_word_wreq_bytelen: u8 = device_addr_bytelen + SINGLE_WORD_BYTELEN;
    let double_word_wreq_bytelen: u8 = device_addr_bytelen + DOUBLE_WORD_BYTELEN;

    #[allow(nonstandard_style)]
    const command: [u8; 2] = COMMAND_RANDOM_WRITE.to_le_bytes();
    let subcommand: [u8; 2] = get_subcommand(query.connection_props.cpu, query.access_type)?;

    let data_packet_len = match query.access_type {
        AccessType::Word => {
            const LENGTH_SPECIFIER_BYTELEN: u8 = 2;
            let single_words_wreq_bytelen: u8 = query.single_word_access_points * single_word_wreq_bytelen;
            let double_words_wreq_bytelen: u8 = query.double_word_access_points * double_word_wreq_bytelen;
            LENGTH_SPECIFIER_BYTELEN + single_words_wreq_bytelen + double_words_wreq_bytelen
        },
        AccessType::Bit => {
            const LENGTH_SPECIFIER_BYTELEN: u8 = 1;
            let bits_wreq_bytelen: u8 = query.bit_access_points * bit_wreq_bytelen;
            LENGTH_SPECIFIER_BYTELEN + bits_wreq_bytelen
        }
    } as usize;

    let mut data_packet: Vec<u8> = Vec::with_capacity(data_packet_len);

    match query.access_type {
        AccessType::Word => {
            data_packet.extend([query.single_word_access_points, query.double_word_access_points]);
            for x in query.sorted_data {
                data_packet.extend(&x.device.serialize(query.connection_props.cpu)?);
                data_packet.extend(x.data.to_bytes());
            }
        }
        AccessType::Bit => {
            data_packet.push(query.bit_access_points);
             for x in query.sorted_data {
                data_packet.extend(&x.device.serialize(query.connection_props.cpu)?);
                data_packet.push(matches!(x.data, TypedData::Bool(true)) as u8);
                if query.connection_props.cpu == CPU::R {
                    data_packet.push(0);
                }
             }
        }
    }

    let command_len: u16 = (COMMAND_PREFIX_BYTELEN + data_packet_len) as u16;
    let header: [u8; HEADER_BYTELEN + CPUTIMER_BYTELEN] = query.connection_props.generate_header(command_len);

    let mut packet: Vec<u8> = Vec::with_capacity(HEADER_BYTELEN + command_len as usize);
    packet.extend(header);
    packet.extend(command);
    packet.extend(subcommand);
    packet.extend(data_packet);

    Ok(packet)
}
