use crate::{CPU, Device, SLMP4EConnectionProps, TypedDevice};
use crate::commands::{HEADER_BYTELEN, CPUTIMER_BYTELEN, COMMAND_PREFIX_BYTELEN};

const COMMAND_RANDOM_READ: u16 = 0x0403;

pub struct SLMPRandomReadQuery<'a>{
    pub connection_props: SLMP4EConnectionProps,
    pub sorted_devices: &'a [TypedDevice],
    pub single_word_access_points: u8,
    pub double_word_access_points: u8,
}

pub struct SLMPRandomReadCommand(pub Vec<u8>);
impl std::ops::Deref for SLMPRandomReadCommand {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> TryFrom<SLMPRandomReadQuery<'a>> for SLMPRandomReadCommand {
    type Error = std::io::Error;
    fn try_from(value: SLMPRandomReadQuery<'a>) -> Result<Self, Self::Error> {
        let cmd = construct_frame(value)?;
        Ok(Self(cmd))
    }
}

fn construct_frame (query: SLMPRandomReadQuery) -> std::io::Result<Vec<u8>> {
    
    const ACCESS_POINTS_BYTELEN: usize = 2;

    #[allow(nonstandard_style)]
    const command: [u8; 2] = COMMAND_RANDOM_READ.to_le_bytes();
    let subcommand: [u8; 2] = match query.connection_props.cpu {
        CPU::Q | CPU::L => [0x00, 0x00],
        CPU::R => [0x02, 0x00],
        _ => return Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Unsupported CPU"))
    };

    let device_addr_bytelen: u8 = Device::addr_code_len(query.connection_props.cpu)?;
    let total_access_points: u8 = query.single_word_access_points + query.double_word_access_points;

    let data_packet_len: usize = ACCESS_POINTS_BYTELEN + (total_access_points * device_addr_bytelen) as usize;
    let mut data_packet: Vec<u8> = Vec::with_capacity(data_packet_len);

    data_packet.extend([query.single_word_access_points, query.double_word_access_points,]);
    for device in query.sorted_devices {
        data_packet.extend(device.device.serialize(query.connection_props.cpu)?);
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