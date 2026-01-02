use crate::device::DeviceSize;
use crate::{CPU, Device, MonitorList};
use crate::commands::COMMAND_BYTELEN;

const COMMAND_RANDOM_READ: u16 = 0x0403;

pub(crate) struct SLMPRandomReadQuery<'a>{
    pub cpu: &'a CPU,
    pub monitor_list: &'a MonitorList
}

pub(crate) struct SLMPRandomReadCommand(pub Vec<u8>);
impl std::ops::Deref for SLMPRandomReadCommand {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> From<SLMPRandomReadQuery<'a>> for SLMPRandomReadCommand {
    fn from(value: SLMPRandomReadQuery<'a>) -> Self {
        let cmd = construct_frame(value);
        Self(cmd)
    }
}

fn construct_frame (query: SLMPRandomReadQuery) -> Vec<u8> {

    const ACCESS_POINTS_BYTELEN: usize = 2;

    const COMMAND: [u8; 2] = COMMAND_RANDOM_READ.to_le_bytes();
    let subcommand: [u8; 2] = match query.cpu {
        CPU::Q | CPU::L => [0x00, 0x00],
        CPU::R => [0x02, 0x00]
    };

    let device_addr_bytelen: usize = Device::addr_code_len(query.cpu) as usize;
    let total_access_points: usize = (query.monitor_list.single_word_access_points + query.monitor_list.double_word_access_points) as usize;

    let data_packet_len: usize = ACCESS_POINTS_BYTELEN + (total_access_points * device_addr_bytelen);
    let mut data_packet: Vec<u8> = Vec::with_capacity(data_packet_len);

    data_packet.extend([query.monitor_list.single_word_access_points, query.monitor_list.double_word_access_points,]);

    // The devices "sorted_device" is in the order of single-word, multi-word, and double-word.
    // A multi-word read-request is to be decomposed to single-word read-requests.
    for device in &query.monitor_list.sorted_devices {
        match device.1.data_type.device_size() {
            DeviceSize::MultiWord(n) => {
                let mut target_device = device.1.device;
                for _ in 0..n {
                    data_packet.extend(target_device.serialize(query.cpu));
                    target_device.address += 1 as usize;
                }
            },
            _ => data_packet.extend(device.1.device.serialize(query.cpu)),
        };
    }

    let mut packet: Vec<u8> = Vec::with_capacity(COMMAND_BYTELEN + data_packet_len);
    packet.extend(COMMAND);
    packet.extend(subcommand);
    packet.extend(data_packet);

    packet
}
