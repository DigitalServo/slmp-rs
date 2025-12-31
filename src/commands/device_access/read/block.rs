use crate::{AccessType, CPU, Device, DeviceBlock, SLMP4EConnectionProps};
use crate::commands::{HEADER_BYTELEN, CPUTIMER_BYTELEN, COMMAND_PREFIX_BYTELEN};

use crate::div_ceil;

const COMMAND_BLOCK_READ: u16 = 0x0406;

pub struct SLMPBlockReadQuery<'a>{
    pub connection_props: &'a SLMP4EConnectionProps,
    pub sorted_block: &'a [DeviceBlock],
    pub word_access_points: u8,
    pub bit_access_points: u8,
}

pub struct SLMPBlockReadCommand(pub Vec<u8>);
impl std::ops::Deref for SLMPBlockReadCommand {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> TryFrom<SLMPBlockReadQuery<'a>> for SLMPBlockReadCommand {
    type Error = std::io::Error;
    fn try_from(value: SLMPBlockReadQuery<'a>) -> Result<Self, Self::Error> {
        let cmd = construct_frame(value)?;
        Ok(Self(cmd))
    }
}

fn construct_frame (query: SLMPBlockReadQuery) -> std::io::Result<Vec<u8>> {

    const ACCESS_POINTS_BYTELEN: usize = 2;
    const DEVICE_SIZE_BYTELEN: u8 = 2;
    let device_addr_bytelen: u8 = Device::addr_code_len(query.connection_props.cpu)?;
    let device_rreq_bytelen: u8 = device_addr_bytelen + DEVICE_SIZE_BYTELEN;

    #[allow(nonstandard_style)]
    const command: [u8; 2] = COMMAND_BLOCK_READ.to_le_bytes();
    let subcommand: [u8; 2] = match query.connection_props.cpu {
        CPU::Q | CPU::L => Ok([0x00, 0x00]),
        CPU::R => Ok([0x02, 0x00]),
        _ => Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Unsupported CPU"))
    }?;

    let total_access_points: u8 = query.word_access_points + query.bit_access_points;

    let data_packet_len: usize = ACCESS_POINTS_BYTELEN + (total_access_points * device_rreq_bytelen) as usize;
    let mut data_packet: Vec<u8> = Vec::with_capacity(data_packet_len as usize);

    data_packet.extend([query.word_access_points, query.bit_access_points]);
    for block in query.sorted_block {
        let request_size = match block.access_type {
            AccessType::Word => block.size,
            AccessType::Bit => div_ceil(block.size, 8),
        } as u16;

        data_packet.extend(block.start_device.serialize(query.connection_props.cpu)?);
        data_packet.extend(request_size.to_le_bytes());
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
