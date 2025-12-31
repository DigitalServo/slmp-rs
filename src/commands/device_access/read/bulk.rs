use crate::{AccessType, CPU, DataType, Device, DeviceSize, SLMP4EConnectionProps};
use crate::commands::{HEADER_BYTELEN, CPUTIMER_BYTELEN, COMMAND_PREFIX_BYTELEN};

const COMMAND_BULK_READ: u16 = 0x0401;

pub struct SLMPBulkReadQuery<'a> {
    pub connection_props: &'a SLMP4EConnectionProps,
    pub start_device: Device,
    pub device_num: usize,
    pub data_type: DataType
}

pub struct SLMPBulkReadCommand(pub Vec<u8>);
impl std::ops::Deref for SLMPBulkReadCommand {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> TryFrom<SLMPBulkReadQuery<'a>> for SLMPBulkReadCommand {
    type Error = std::io::Error;
    fn try_from(value: SLMPBulkReadQuery) -> Result<Self, Self::Error> {
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

fn construct_frame (query: SLMPBulkReadQuery) -> std::io::Result<Vec<u8>> {

    let access_type: AccessType = match query.data_type {
        DataType::Bool => AccessType::Bit,
        _ => AccessType::Word
    };

    #[allow(nonstandard_style)]
    const command: [u8; 2] = COMMAND_BULK_READ.to_le_bytes();
    let subcommand: [u8; 2] = get_subcommand(query.connection_props.cpu, access_type)?;

    let start_address: Box<[u8]> = query.start_device.serialize(query.connection_props.cpu)?;
    let device_size_code: [u8; 2] = (query.device_num as u16 * <DeviceSize as Into<u16>>::into(query.data_type.device_size())).to_le_bytes();

    let device_addr_bytelen: u8 = Device::addr_code_len(query.connection_props.cpu)?;
    let data_packet_bytelen: u8 = device_addr_bytelen + 2;

    let mut data_packet: Vec<u8> = Vec::with_capacity(data_packet_bytelen as usize);
    data_packet.extend_from_slice(&start_address);
    data_packet.extend_from_slice(&device_size_code);

    let command_len: u16 = (COMMAND_PREFIX_BYTELEN + data_packet_bytelen as usize) as u16;
    let header: [u8; HEADER_BYTELEN + CPUTIMER_BYTELEN] = query.connection_props.generate_header(command_len);

    let mut packet: Vec<u8> = Vec::with_capacity(HEADER_BYTELEN + command_len as usize);
    packet.extend(header);
    packet.extend(command);
    packet.extend(subcommand);
    packet.extend(data_packet);

    Ok(packet)
}
