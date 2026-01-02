use crate::device::DeviceSize;
use crate::{AccessType, CPU, Device, DeviceData, TypedData};
use crate::commands::COMMAND_BYTELEN;

const COMMAND_RANDOM_WRITE: u16 = 0x1402;

pub(crate) struct SLMPRandomWriteQuery<'a> {
    pub cpu: &'a CPU,
    pub sorted_data: &'a [DeviceData],
    pub access_type: AccessType,
    pub bit_access_points: u8,
    pub single_word_access_points: u8,
    pub double_word_access_points: u8,
}

pub(crate) struct SLMPRandomWriteCommand(pub Vec<u8>);
impl std::ops::Deref for SLMPRandomWriteCommand {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> From<SLMPRandomWriteQuery<'a>> for SLMPRandomWriteCommand {
    fn from(value: SLMPRandomWriteQuery) -> Self {
        let cmd = construct_frame(value);
        Self(cmd)
    }
}

fn construct_frame(query: SLMPRandomWriteQuery) -> Vec<u8> {

    const SINGLE_WORD_BYTELEN: u8 = 2;
    const DOUBLE_WORD_BYTELEN: u8 = 4;

    let bit_bytelen = match query.cpu {
        CPU::Q | CPU::L => 1,
        CPU::R => 2,
    };

    let device_addr_bytelen: u8 = Device::addr_code_len(query.cpu);
    let bit_wreq_bytelen: u8 = device_addr_bytelen + bit_bytelen;
    let single_word_wreq_bytelen: u8 = device_addr_bytelen + SINGLE_WORD_BYTELEN;
    let double_word_wreq_bytelen: u8 = device_addr_bytelen + DOUBLE_WORD_BYTELEN;

    const COMMAND: [u8; 2] = COMMAND_RANDOM_WRITE.to_le_bytes();
    let subcommand: [u8; 2] = match (query.access_type, query.cpu) {
        (AccessType::Bit, CPU::Q | CPU::L) => [0x01, 0x00],
        (AccessType::Bit, CPU::R) => [0x03, 0x00],
        (AccessType::Word, CPU::Q | CPU::L) => [0x00, 0x00],
        (AccessType::Word, CPU::R) => [0x02, 0x00],
    };

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
                // The devices "sorted_device" is in the order of single-word, multi-word, and double-word.
                // A multi-word read-request is to be decomposed to single-word read-requests.
                match x.data.get_type().device_size() {
                    DeviceSize::MultiWord(n) => {
                        let mut target_device = x.device;
                        let bytelen = n as usize * 2;
                        let data = &x.data.to_bytes()[..bytelen];
                        for word_data in data.chunks_exact(SINGLE_WORD_BYTELEN as usize) {
                            data_packet.extend(target_device.serialize(query.cpu));
                            data_packet.extend(word_data);
                            target_device.address += 1 as usize;
                        }
                    },
                    _ => {
                        data_packet.extend(&x.device.serialize(query.cpu));
                        data_packet.extend(x.data.to_bytes());
                    }
                }
            }
        }
        AccessType::Bit => {
            data_packet.push(query.bit_access_points);
             for x in query.sorted_data {
                data_packet.extend(&x.device.serialize(query.cpu));
                data_packet.push(matches!(x.data, TypedData::Bool(true)) as u8);
                if query.cpu == &CPU::R {
                    data_packet.push(0);
                }
             }
        }
    }

    let mut packet: Vec<u8> = Vec::with_capacity(COMMAND_BYTELEN + data_packet_len);
    packet.extend(COMMAND);
    packet.extend(subcommand);
    packet.extend(data_packet);

    packet
}
