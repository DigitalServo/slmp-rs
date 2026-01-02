use crate::{AccessType, BlockedDeviceData, CPU, TypedData, bits_to_u8, div_ceil};
use crate::commands::COMMAND_BYTELEN;

const COMMAND_BLOCK_WRITE: u16 = 0x1406;

pub(crate) struct SLMPBlockWriteQuery<'a> {
    pub cpu: &'a CPU,
    pub sorted_data: &'a [BlockedDeviceData<'a>],
    pub word_access_points: u8,
    pub bit_access_points: u8,
}

pub(crate) struct SLMPBlockWriteCommand(pub Vec<u8>);
impl std::ops::Deref for SLMPBlockWriteCommand {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> From<SLMPBlockWriteQuery<'a>> for SLMPBlockWriteCommand {
    fn from(value: SLMPBlockWriteQuery) -> Self {
        let cmd = construct_frame(value);
        Self(cmd)
    }
}

fn construct_frame(query: SLMPBlockWriteQuery) -> Vec<u8> {

    const COMMAND: [u8; 2] = COMMAND_BLOCK_WRITE.to_le_bytes();
    let subcommand: [u8; 2] = match query.cpu {
        CPU::Q | CPU::L => [0x00, 0x00],
        CPU::R => [0x02, 0x00],
    };

    let mut data_packet: Vec<u8> = vec![];

    data_packet.extend([query.word_access_points, query.bit_access_points]);
    for block in query.sorted_data {
        let start_address: Box<[u8]> = block.start_device.serialize(query.cpu);

        match block.access_type {
            AccessType::Word => {
                let mut data_code: Vec<u8> = vec![];
                for x in block.data {
                    data_code.extend(x.to_bytes());
                }
                let word_size = data_code.len() / 2;
                let device_size_code: [u8; 2] = (word_size as u16).to_le_bytes();

                data_packet.extend(start_address);
                data_packet.extend(device_size_code);
                data_packet.extend(data_code);
            },
            AccessType::Bit => {
                const BYTE_BIT_SIZE: usize = 8;
                const WORD_BIT_SIZE: usize = 16;

                let word_size = div_ceil(block.data.len(), WORD_BIT_SIZE);
                let mut bit_array = vec![false; word_size * WORD_BIT_SIZE];
                for (i, x) in block.data.iter().enumerate() {
                    bit_array[i] = matches!(x, TypedData::Bool(true));
                }
                let data_code: Vec<u8> = bit_array.chunks_exact(BYTE_BIT_SIZE)
                    .map(|x| bits_to_u8([x[0], x[1], x[2], x[3], x[4], x[5], x[6], x[7]]))
                    .collect();

                let device_size_code: [u8; 2] = (word_size as u16).to_le_bytes();

                data_packet.extend(start_address);
                data_packet.extend(device_size_code);
                data_packet.extend(data_code);
            }
        }
    }

    let mut packet: Vec<u8> = Vec::with_capacity(COMMAND_BYTELEN + data_packet.len());
    packet.extend(COMMAND);
    packet.extend(subcommand);
    packet.extend(data_packet);

    packet
}
