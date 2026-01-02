use crate::{AccessType, CPU, DataType, Device};
use crate::commands::COMMAND_BYTELEN;

const COMMAND_BULK_READ: u16 = 0x0401;

pub(crate) struct SLMPBulkReadQuery<'a> {
    pub cpu: &'a CPU,
    pub start_device: Device,
    pub device_num: usize,
    pub data_type: DataType
}

pub(crate) struct SLMPBulkReadCommand(pub Vec<u8>);
impl std::ops::Deref for SLMPBulkReadCommand {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> From<SLMPBulkReadQuery<'a>> for SLMPBulkReadCommand {
    fn from(value: SLMPBulkReadQuery) -> Self {
        let cmd = construct_frame(value);
        Self(cmd)
    }
}

fn construct_frame (query: SLMPBulkReadQuery) -> Vec<u8> {

    let access_type: AccessType = match query.data_type {
        DataType::Bool => AccessType::Bit,
        _ => AccessType::Word
    };

    const COMMAND: [u8; 2] = COMMAND_BULK_READ.to_le_bytes();
    let subcommand: [u8; 2] = match access_type {
        AccessType::Bit => match query.cpu {
            CPU::Q | CPU::L => [0x01, 0x00],
            CPU::R => [0x03, 0x00],
        },
        AccessType::Word => match query.cpu {
            CPU::Q | CPU::L => [0x00, 0x00],
            CPU::R => [0x02, 0x00],
        }
    };

    let start_address: Box<[u8]> = query.start_device.serialize(query.cpu);
    let device_size_code: [u8; 2] = ((query.device_num * usize::from(query.data_type.device_size())) as u16).to_le_bytes();

    let device_addr_len: u8 = Device::addr_code_len(query.cpu);
    let data_packet_len = device_addr_len as usize + 2;

    let mut data_packet: Vec<u8> = Vec::with_capacity(data_packet_len as usize);
    data_packet.extend_from_slice(&start_address);
    data_packet.extend_from_slice(&device_size_code);

    let mut packet: Vec<u8> = Vec::with_capacity(COMMAND_BYTELEN + data_packet_len);
    packet.extend(COMMAND);
    packet.extend(subcommand);
    packet.extend(data_packet);

    packet
}
