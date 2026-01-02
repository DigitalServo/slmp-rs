use crate::{AccessType, CPU, Device, DeviceBlock};
use crate::commands::{COMMAND_BYTELEN};

use crate::div_ceil;

const COMMAND_BLOCK_READ: u16 = 0x0406;

pub(crate) struct SLMPBlockReadQuery<'a>{
    pub cpu: &'a CPU,
    pub sorted_block: &'a [DeviceBlock],
    pub word_access_points: u8,
    pub bit_access_points: u8,
}

pub(crate) struct SLMPBlockReadCommand(pub Vec<u8>);
impl std::ops::Deref for SLMPBlockReadCommand {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> From<SLMPBlockReadQuery<'a>> for SLMPBlockReadCommand {
    fn from(value: SLMPBlockReadQuery<'a>) -> Self {
        let cmd = construct_frame(value);
        Self(cmd)
    }
}

fn construct_frame (query: SLMPBlockReadQuery) -> Vec<u8> {

    const ACCESS_POINTS_BYTELEN: usize = 2;
    const DEVICE_SIZE_BYTELEN: u8 = 2;
    let device_addr_bytelen: u8 = Device::addr_code_len(query.cpu);
    let device_rreq_bytelen: u8 = device_addr_bytelen + DEVICE_SIZE_BYTELEN;

    const COMMAND: [u8; 2] = COMMAND_BLOCK_READ.to_le_bytes();
    let subcommand: [u8; 2] = match query.cpu {
        CPU::Q | CPU::L => [0x00, 0x00],
        CPU::R => [0x02, 0x00],
    };

    let total_access_points: u8 = query.word_access_points + query.bit_access_points;

    let data_packet_len: usize = ACCESS_POINTS_BYTELEN + (total_access_points * device_rreq_bytelen) as usize;
    let mut data_packet: Vec<u8> = Vec::with_capacity(data_packet_len as usize);

    data_packet.extend([query.word_access_points, query.bit_access_points]);
    for block in query.sorted_block {
        let request_size = match block.access_type {
            AccessType::Word => block.size,
            AccessType::Bit => div_ceil(block.size, 8),
        } as u16;

        data_packet.extend(block.start_device.serialize(query.cpu));
        data_packet.extend(request_size.to_le_bytes());
    }

    let mut packet: Vec<u8> = Vec::with_capacity(COMMAND_BYTELEN + data_packet_len);
    packet.extend(COMMAND);
    packet.extend(subcommand);
    packet.extend(data_packet);

    packet
}
