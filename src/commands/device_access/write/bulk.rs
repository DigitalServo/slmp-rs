use crate::{AccessType, CPU, Device, TypedData, div_ceil};
use crate::commands::COMMAND_BYTELEN;

const COMMAND_BULK_WRITE: u16 = 0x1401;

pub(crate) struct SLMPBulkWriteQuery<'a> {
    pub cpu: &'a CPU,
    pub start_device: Device,
    pub data: &'a [TypedData],
}

pub(crate) struct SLMPBulkWriteCommand(pub Vec<u8>);
impl std::ops::Deref for SLMPBulkWriteCommand {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> From<SLMPBulkWriteQuery<'a>> for SLMPBulkWriteCommand {
    fn from(value: SLMPBulkWriteQuery) -> Self {
        let cmd = construct_frame(value);
        Self(cmd)
    }
}

fn construct_frame(query: SLMPBulkWriteQuery) -> Vec<u8> {

    let access_type: AccessType = match query.data.iter().all(|x| matches!(x, TypedData::Bool(_))) {
        true => AccessType::Bit,
        false => AccessType::Word
    };

    const COMMAND: [u8; 2] = COMMAND_BULK_WRITE.to_le_bytes();
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

    let mut data_packet: Vec<u8> = vec![];

    data_packet.extend(start_address);
    match access_type {
        AccessType::Word => {
            let mut data_code: Vec<u8> = vec![];
            for x in query.data {
                data_code.extend(x.to_bytes());
            }
            let word_size: usize = data_code.len() / 2;
            let device_size_code: [u8; 2] = (word_size as u16).to_le_bytes();

            data_packet.extend(device_size_code);
            data_packet.extend(data_code);
        }
        AccessType::Bit => {
            let byte_size = div_ceil(query.data.len(), 2);
            let mut bit_array = vec![false; byte_size * 2];
            for (i, x) in query.data.iter().enumerate() {
                bit_array[i] = matches!(x, TypedData::Bool(true));
            }

            let data_code: Vec<u8> = bit_array.chunks_exact(2)
                    .map(|x| (x[1] as u8) + ((x[0] as u8) << 4))
                    .collect();
            let device_size_code: [u8; 2] = (query.data.len() as u16).to_le_bytes();

            data_packet.extend(device_size_code);
            data_packet.extend(data_code);
        }
    }

    let mut packet: Vec<u8> = Vec::with_capacity(COMMAND_BYTELEN + data_packet.len());
    packet.extend(COMMAND);
    packet.extend(subcommand);
    packet.extend(data_packet);

    packet
}
