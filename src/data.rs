use serde::{Deserialize, Serialize};

use crate::device::DeviceSize;

/// Available data type for SLMP communication.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "json-api", serde(rename_all = "PascalCase"))]
pub enum DataType {
    Bool = 1,
    U16 = 2,
    I16 = 3,
    U32 = 4,
    I32 = 5,
    F32 = 6,
    F64 = 7,
}

impl DataType {
    #[inline(always)]
    pub(crate) const fn byte_size(&self) -> usize {
        match self {
            DataType::Bool => 1,
            DataType::U16 | DataType::I16 => 2,
            DataType::U32 | DataType::I32 | DataType::F32=> 4,
            DataType::F64 => 8,
        }
    }

    #[inline(always)]
    pub(crate) const fn device_size(&self) -> DeviceSize {
        match self {
            DataType::Bool => DeviceSize::Bit,
            DataType::U16 | DataType::I16 => DeviceSize::SingleWord,
            DataType::U32 | DataType::I32 | DataType::F32 => DeviceSize::DoubleWord,
            DataType::F64 => DeviceSize::QuadrupleWord,
        }
    }
}

/// Available typed-data for SLMP communication.
/// It is used for all of write requests.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "json-api", serde(rename_all = "PascalCase"))]
#[serde(tag = "type", content = "value")]
pub enum TypedData {
    Bool(bool),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    F32(f32),
    F64(f64),
}


impl From<(&[u8], DataType)> for TypedData {
    #[inline(always)]
    fn from(value: (&[u8], DataType)) -> Self {
        match value.1 {
            DataType::Bool => TypedData::Bool(value.0[0] == 1),
            DataType::U16 => TypedData::U16(u16::from_le_bytes([value.0[0], value.0[1]])),
            DataType::I16 => TypedData::I16(i16::from_le_bytes([value.0[0], value.0[1]])),
            DataType::U32 => TypedData::U32(u32::from_le_bytes([value.0[0], value.0[1], value.0[2], value.0[3]])),
            DataType::I32 => TypedData::I32(i32::from_le_bytes([value.0[0], value.0[1], value.0[2], value.0[3]])),
            DataType::F32 => TypedData::F32(f32::from_le_bytes([value.0[0], value.0[1], value.0[2], value.0[3]])),
            DataType::F64 => TypedData::F64(f64::from_le_bytes([value.0[0], value.0[1], value.0[2], value.0[3], value.0[4], value.0[5], value.0[6], value.0[7]])),
        }
    }
}

impl TypedData {
    #[inline(always)]
    pub const fn to_bytes(&self) -> &[u8] {
        unsafe {
            match self {
                TypedData::Bool(true)  => &[1, 0],
                TypedData::Bool(false) => &[0, 0],
                TypedData::U16(v) => std::slice::from_raw_parts(v as *const u16 as *const u8, 2),
                TypedData::I16(v) => std::slice::from_raw_parts(v as *const i16 as *const u8, 2),
                TypedData::U32(v) => std::slice::from_raw_parts(v as *const u32 as *const u8, 4),
                TypedData::I32(v) => std::slice::from_raw_parts(v as *const i32 as *const u8, 4),
                TypedData::F32(v) => std::slice::from_raw_parts(v as *const f32 as *const u8, 4),
                TypedData::F64(v) => std::slice::from_raw_parts(v as *const f64 as *const u8, 8),
            }
        }
    }
    
    #[inline(always)]
    pub const fn get_type(&self) -> DataType {
        match self {
            TypedData::Bool(_) => DataType::Bool,
            TypedData::U16(_) => DataType::U16,
            TypedData::I16(_) => DataType::I16,
            TypedData::U32(_) => DataType::U32,
            TypedData::I32(_) => DataType::I32,
            TypedData::F32(_) => DataType::F32,
            TypedData::F64(_) => DataType::F64,
        }
    }
}