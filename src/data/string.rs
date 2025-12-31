use serde::{Serialize, Serializer, Deserialize, Deserializer};
use encoding_rs::SHIFT_JIS;

const PLCSTRING_MAX_BYTES: usize = 64;
const PLCSTRING_MAX_DEVICE_SIZE: usize = PLCSTRING_MAX_BYTES / 2;

const SHIFT_JIS_NULL_CODE: u8 = 0x00;

pub const PLCSTRING_QUERY_SPLITTER: &str = "#|#";

/// String is stored as u8 array (max: 64 byte).
/// Character code is Shift-JIS.
#[derive(Clone, Copy)]
pub struct PLCString {
    pub data: [u8; PLCSTRING_MAX_BYTES],
    pub(crate) effective_len: u8,
    pub(crate) device_size: u8,
}

impl PLCString {

    pub fn from_shift_jis_bytes(bytes: &[u8], device_size: u8) -> Self {
        let nul_pos = bytes.iter().position(|&b| b == SHIFT_JIS_NULL_CODE).unwrap_or(bytes.len());

        let effective_len = nul_pos.min(device_size as usize * 2);
        let device_size = device_size.min(PLCSTRING_MAX_DEVICE_SIZE as u8);

        let mut data = [SHIFT_JIS_NULL_CODE; PLCSTRING_MAX_BYTES];
        data[..effective_len].copy_from_slice(&bytes[..effective_len]);

        PLCString { data, effective_len: effective_len as u8, device_size}
    }

    pub fn as_bytes(&self) -> &[u8] {
        let bytes = self.device_size as usize * 2;
        &self.data[..bytes]
    }

    pub fn as_str(&self) -> std::borrow::Cow<'_, str> {
        let bytes = &self.data[..self.effective_len as usize];
        let (cow, _, _) = encoding_rs::SHIFT_JIS.decode(bytes);
        cow
    }

    pub fn to_string(&self) -> String {
        self.as_str().into_owned()
    }

    pub fn is_empty(&self) -> bool {
        self.effective_len == 0
    }
}

impl From<(&str, u8)> for PLCString {
    fn from(s: (&str, u8)) -> Self {
        let (shift_jis_bytes, _, _) = SHIFT_JIS.encode(s.0);
        Self::from_shift_jis_bytes(&shift_jis_bytes, s.1)
    }
}

impl std::fmt::Display for PLCString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::fmt::Debug for PLCString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("PLCString")
            .field(&self.as_str())
            .finish()
    }
}

impl PartialEq for PLCString {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for PLCString {}

impl PartialOrd for PLCString {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PLCString {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(&other.as_str())
    }
}


impl Serialize for PLCString {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.as_str().into_owned())
    }
}

impl<'de> Deserialize<'de> for PLCString {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {

        let combined = String::deserialize(deserializer)?;
        let parts: Vec<&str> = combined.splitn(2, PLCSTRING_QUERY_SPLITTER).collect();

        if parts.len() != 2 {
            return Err(serde::de::Error::custom(format!(
                "String　input must be expressed in the form 'device_size{}text'",
                PLCSTRING_QUERY_SPLITTER
            )));
        }

        let device_size_str = parts[0].trim();
        let text = parts[1].trim().to_string();

        let device_size: u8 = device_size_str.parse().map_err(|_| {
            serde::de::Error::custom(format!(
                "Invalid device_size provided: {}",
                device_size_str
            ))
        })?;

        if !(1..=PLCSTRING_MAX_DEVICE_SIZE as u8).contains(&device_size) {
            return Err(serde::de::Error::custom(format!(
                "device_size must be between 1 and {}",
                PLCSTRING_MAX_DEVICE_SIZE
            )));
        }

        let (shift_jis_bytes, _, had_errors) = SHIFT_JIS.encode(&text);

        if had_errors {
            return Err(serde::de::Error::custom("Contains characters not representable in Shift-JIS"));
        }

        let required_byte_size = shift_jis_bytes.len();
        let allowed_byte_size = device_size as usize * 2;

        if required_byte_size > allowed_byte_size {
            return Err(serde::de::Error::custom(format!(
                "Device size is too small to store Shift-JIS string: Specified size: {}, Required: {})",
                device_size, required_byte_size
            )));
        }

        Ok(Self::from_shift_jis_bytes(&shift_jis_bytes, device_size))
    }
}
