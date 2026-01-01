use crate::{CPU, Device, SLMP4EConnectionProps, MonitorList};
use crate::commands::{HEADER_BYTELEN, CPUTIMER_BYTELEN, COMMAND_PREFIX_BYTELEN};

const COMMAND_REGISTER_MONITOR: u16 = 0x0801;
const COMMAND_READ_MONITOR: u16 = 0x0802;

pub struct SLMPMonitorRegisterQuery<'a>{
    pub connection_props: &'a SLMP4EConnectionProps,
    pub monitor_list: &'a MonitorList
}

pub struct SLMPMonitorRegisterCommand(pub Vec<u8>);
impl std::ops::Deref for SLMPMonitorRegisterCommand {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> From<SLMPMonitorRegisterQuery<'a>> for SLMPMonitorRegisterCommand {
    fn from(value: SLMPMonitorRegisterQuery<'a>) -> Self{
        let cmd = construct_frame(value);
        Self(cmd)
    }
}

fn construct_frame (query: SLMPMonitorRegisterQuery) -> Vec<u8> {

    const ACCESS_POINTS_BYTELEN: usize = 2;

    #[allow(nonstandard_style)]
    const command: [u8; 2] = COMMAND_REGISTER_MONITOR.to_le_bytes();
    let subcommand: [u8; 2] = match query.connection_props.cpu {
        CPU::Q | CPU::L => [0x00, 0x00],
        CPU::R => [0x02, 0x00],
    };

    let device_addr_bytelen: usize = Device::addr_code_len(query.connection_props.cpu) as usize;
    let total_access_points: usize = (query.monitor_list.single_word_access_points + query.monitor_list.double_word_access_points) as usize;

    let data_packet_len: usize = ACCESS_POINTS_BYTELEN + (total_access_points * device_addr_bytelen);
    let mut data_packet: Vec<u8> = Vec::with_capacity(data_packet_len);

    data_packet.extend([query.monitor_list.single_word_access_points, query.monitor_list.double_word_access_points,]);
    for device in &query.monitor_list.sorted_devices {
        data_packet.extend(device.1.device.serialize(query.connection_props.cpu));
    }

    let command_len: u16 = (COMMAND_PREFIX_BYTELEN + data_packet_len) as u16;
    let header: [u8; HEADER_BYTELEN + CPUTIMER_BYTELEN] = query.connection_props.generate_header(command_len);

    let mut packet: Vec<u8> = Vec::with_capacity(HEADER_BYTELEN + command_len as usize);
    packet.extend(header);
    packet.extend(command);
    packet.extend(subcommand);
    packet.extend(data_packet);

    packet
}


pub struct SLMPMonitorReadQuery<'a>{
    pub connection_props: &'a SLMP4EConnectionProps
}

pub struct SLMPMonitorReadCommand(pub Vec<u8>);
impl std::ops::Deref for SLMPMonitorReadCommand {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> From<SLMPMonitorReadQuery<'a>> for SLMPMonitorReadCommand {
    fn from(value: SLMPMonitorReadQuery<'a>) -> Self {

        #[allow(nonstandard_style)]
        const command: [u8; 2] = COMMAND_READ_MONITOR.to_le_bytes();
        let subcommand: [u8; 2] = [0x00, 0x00];

        let command_len: u16 = COMMAND_PREFIX_BYTELEN as u16;
        let header: [u8; HEADER_BYTELEN + CPUTIMER_BYTELEN] = value.connection_props.generate_header(command_len);

        let mut packet: Vec<u8> = Vec::with_capacity(HEADER_BYTELEN + command_len as usize);
        packet.extend(header);
        packet.extend(command);
        packet.extend(subcommand);

        Self(packet)
    }
}
