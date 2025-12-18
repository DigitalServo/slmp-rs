use serde::{Deserialize, Serialize};
use crate::{CPU, DataType, TypedData};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "serde-camel", serde(rename_all = "camelCase"))]
pub enum AccessType {
    Bit = 2,
    Word = 1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "serde-camel", serde(rename_all = "camelCase"))]
pub(crate) enum DeviceSize {
    Bit = 1,
    SingleWord = 2,
    DoubleWord = 3,
    QuadrupleWord = 4,
}

impl From<DeviceSize> for u16 {
    #[inline(always)]
    fn from(value: DeviceSize) -> Self {
        match value {
            DeviceSize::Bit => 1,
            DeviceSize::SingleWord => 1,
            DeviceSize::DoubleWord => 2,
            DeviceSize::QuadrupleWord => 4,
        }
    }
}


/// Device type used in Mitsubishi PLC.
/// 
/// Available devices: X, Y, M, L, F, V, B, D, W, S, Z, R, TS, TC, TN, SS, SC, SN, CS, CC, CN, SB, SD, SM, SW, DX, DY, ZR,
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DeviceType {
    X,
    Y,
    M,
    L,
    F,
    V,
    B,
    D,
    W,
    S,
    Z,
    R,
    TS,
    TC,
    TN,
    SS,
    SC,
    SN,
    CS,
    CC,
    CN,
    SB,
    SD,
    SM,
    SW,
    DX,
    DY,
    ZR,
}

impl DeviceType {
    /// Convert a device_type into a byte code for the SLMP communication
    pub fn to_code(&self) -> u8 {
        match self {
            Self::X => 0x9c,
            Self::Y => 0x9d,
            Self::M => 0x90,
            Self::L => 0x92,
            Self::F => 0x93,
            Self::V => 0x94,
            Self::B => 0xa0,
            Self::D => 0xa8,
            Self::W => 0xb4,
            Self::S => 0x98,
            Self::Z => 0xcc,
            Self::R => 0xaf,
            Self::TS => 0xc1,
            Self::TC => 0xc0,
            Self::TN => 0xc2,
            Self::SS => 0xc7,
            Self::SC => 0xc6,
            Self::SN => 0xc8,
            Self::CS => 0xc4,
            Self::CC => 0xc3,
            Self::CN => 0xc5,
            Self::SB => 0xa1,
            Self::SD => 0xa9,
            Self::SM => 0x91,
            Self::SW => 0xb5,
            Self::DX => 0xa2,
            Self::DY => 0xa3,
            Self::ZR => 0xb0,
        }
    }
}

/// It works as a device pointer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "serde-camel", serde(rename_all = "camelCase"))]
pub struct Device {
    pub device_type: DeviceType,
    pub address: usize,
}

impl Device {
    /// Convert a device pointer to a byte code for SLMP communication.
    pub fn serialize(&self, cpu: CPU) -> std::io::Result<Box<[u8]>> {
        let device_code: u8 = self.device_type.to_code();
        let address: [u8; 8] = self.address.to_le_bytes();
        let ret = match cpu {
            CPU::Q | CPU::L => [address[0], address[1], address[2], device_code].into(),
            CPU::R => [address[0], address[1], address[2], 0x00, device_code, 0x00].into(),
            _ => return Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Unsupported CPU"))
        };
        Ok(ret)
    }

    pub fn addr_code_len(cpu: CPU) -> std::io::Result<u8> {
        match cpu {
            CPU::Q | CPU::L => Ok(4),
            CPU::R => Ok(6),
            _ => return Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Unsupported CPU"))
        }
    }
}

/// Device pointer with type annotation.
/// It is used for random-read request.
/// Results of random-read are typed as requested.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "serde-camel", serde(rename_all = "camelCase"))]
pub struct TypedDevice {
    pub device: Device,
    pub data_type: DataType,
}

/// Block unit of the device pointer.
/// It is used for block-read request.
/// Multiple blocks are acceptable for block-read request.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
#[cfg_attr(feature = "serde-camel", serde(rename_all = "camelCase"))]
pub struct DeviceBlock {
    pub access_type: AccessType,
    pub start_device: Device,
    pub size: usize
}

/// Data of the specified device.
/// It is used for random-write request and all of read requests.
/// 
/// Results of the read requets are unified in the form of this struct.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
#[cfg_attr(feature = "serde-camel", serde(rename_all = "camelCase"))]
pub struct DeviceData {
    pub device: Device,
    pub data: TypedData,
}

/// Blocked data used for block-write request.
/// Multiple blocks are acceptable for block-write request.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize)]
#[cfg_attr(feature = "serde-camel", serde(rename_all = "camelCase"))]
pub struct BlockedDeviceData<'a> {
    pub access_type: AccessType,
    pub start_device: Device,
    pub data: &'a [TypedData],
}