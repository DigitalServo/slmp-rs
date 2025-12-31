use serde::{Serialize, Deserialize};
use crate::{bits_to_u16, device::DeviceSize, u16_to_bits};

pub(crate) mod string;
use string::PLCString;

/// Available data type for SLMP communication.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "json-api", serde(rename_all = "PascalCase"))]
pub enum DataType {
    Bool = 1,
    BitArray16 = 2,
    U16 = 3,
    I16 = 4,
    U32 = 7,
    I32 = 8,
    F32 = 9,
    F64 = 5,
    /// You should provide a word size to be accessed.
    String(u8) = 6,
}

impl DataType {
    #[inline(always)]
    pub(crate) const fn byte_size(&self) -> usize {
        match self {
            DataType::Bool => 1,
            DataType::BitArray16 | DataType::U16 | DataType::I16 => 2,
            DataType::U32 | DataType::I32 | DataType::F32 => 4,
            DataType::F64 => 8,
            DataType::String(n) => *n as usize * 2  // n: device size -> (2 * n): byte size
        }
    }

    #[inline(always)]
    pub(crate) const fn device_size(&self) -> DeviceSize {
        match self {
            DataType::Bool => DeviceSize::Bit,
            DataType::BitArray16 | DataType::U16 | DataType::I16 => DeviceSize::SingleWord,
            DataType::U32 | DataType::I32 | DataType::F32 => DeviceSize::DoubleWord,
            DataType::F64 => DeviceSize::MultiWord(4),
            DataType::String(n) => DeviceSize::MultiWord(*n)
        }
    }
}

/// Available typed-data for SLMP communication.
/// It is used for all of write requests.
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "json-api", serde(rename_all = "PascalCase"))]
#[serde(tag = "type", content = "value")]
pub enum TypedData {
    Bool(bool),
    BitArray16([bool; 16]),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    F32(f32),
    F64(f64),
    ///If you send string request with json, is should expressed in the form {type: String, value: `${device_size}${BOUNDSTRING_QUERY_SPLITTER}${text}`}.
    /// BOUNDSTRING_QUERY_SPLITTER is publicly available on this crate.
    String(PLCString),
}

impl From<(&[u8], DataType)> for TypedData {
    #[inline(always)]
    fn from(value: (&[u8], DataType)) -> Self {
        match value.1 {
            DataType::Bool => Self::Bool(u16::from_le_bytes([value.0[0], value.0[1]]) & 0x01 == 1),
            DataType::BitArray16 => Self::BitArray16(u16_to_bits(u16::from_le_bytes([value.0[0], value.0[1]]))),
            DataType::U16 => Self::U16(u16::from_le_bytes([value.0[0], value.0[1]])),
            DataType::I16 => Self::I16(i16::from_le_bytes([value.0[0], value.0[1]])),
            DataType::U32 => Self::U32(u32::from_le_bytes([value.0[0], value.0[1], value.0[2], value.0[3]])),
            DataType::I32 => Self::I32(i32::from_le_bytes([value.0[0], value.0[1], value.0[2], value.0[3]])),
            DataType::F32 => Self::F32(f32::from_le_bytes([value.0[0], value.0[1], value.0[2], value.0[3]])),
            DataType::F64 => Self::F64(f64::from_le_bytes([value.0[0], value.0[1], value.0[2], value.0[3], value.0[4], value.0[5], value.0[6], value.0[7]])),
            DataType::String(n) => Self::String(PLCString::from_shift_jis_bytes(value.0, n)),
        }
    }
}

impl TypedData {
    #[inline(always)]
    pub fn to_bytes(&self) -> &[u8] {
        unsafe {
            match self {
                TypedData::Bool(true)  => &[1, 0],
                TypedData::Bool(false) => &[0, 0],
                TypedData::BitArray16(v) => std::slice::from_raw_parts(bits_to_u16(*v) as *const u16 as *const u8, 2),
                TypedData::U16(v) => std::slice::from_raw_parts(v as *const u16 as *const u8, 2),
                TypedData::I16(v) => std::slice::from_raw_parts(v as *const i16 as *const u8, 2),
                TypedData::U32(v) => std::slice::from_raw_parts(v as *const u32 as *const u8, 4),
                TypedData::I32(v) => std::slice::from_raw_parts(v as *const i32 as *const u8, 4),
                TypedData::F32(v) => std::slice::from_raw_parts(v as *const f32 as *const u8, 4),
                TypedData::F64(v) => std::slice::from_raw_parts(v as *const f64 as *const u8, 8),
                TypedData::String(v) => v.as_bytes(),
            }
        }
    }

    #[inline(always)]
    pub const fn get_type(&self) -> DataType {
        match self {
            TypedData::Bool(_) => DataType::Bool,
            TypedData::BitArray16(_) => DataType::BitArray16,
            TypedData::U16(_) => DataType::U16,
            TypedData::I16(_) => DataType::I16,
            TypedData::U32(_) => DataType::U32,
            TypedData::I32(_) => DataType::I32,
            TypedData::F32(_) => DataType::F32,
            TypedData::F64(_) => DataType::F64,
            TypedData::String(v) => DataType::String(v.device_size)
        }
    }
}

impl From<(&str, u8)> for TypedData {
    fn from(value: (&str, u8)) -> Self {
        Self::String(PLCString::from(value))
    }
}

impl From<bool> for TypedData {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<[bool; 16]> for TypedData {
    fn from(value: [bool; 16]) -> Self {
        Self::BitArray16(value)
    }
}

impl From<u16> for TypedData {
    fn from(value: u16) -> Self {
        Self::U16(value)
    }
}

impl From<i16> for TypedData {
    fn from(value: i16) -> Self {
        Self::I16(value)
    }
}

impl From<u32> for TypedData {
    fn from(value: u32) -> Self {
        Self::U32(value)
    }
}

impl From<i32> for TypedData {
    fn from(value: i32) -> Self {
        Self::I32(value)
    }
}

impl From<f32> for TypedData {
    fn from(value: f32) -> Self {
        Self::F32(value)
    }
}

impl From<f64> for TypedData {
    fn from(value: f64) -> Self {
        Self::F64(value)
    }
}
