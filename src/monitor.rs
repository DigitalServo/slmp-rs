use std::net::SocketAddr;
use serde::{Deserialize, Serialize};
use crate::{DeviceData, SLMP4EConnectionProps, TypedData, TypedDevice, device::DeviceSize};

/// Mitsubishi PLC allow only the signle-word access and double-word access.
/// Multi-word access which used for f64 and string is not supported by default.
/// This library supporrts the multi-word access using signle-word access.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[cfg_attr(feature = "json-api", serde(rename_all = "camelCase"))]
pub struct MonitorList {
    pub sorted_devices: Vec<(usize, TypedDevice)>,
    pub(crate) single_word_access_points: u8,
    pub(crate) double_word_access_points: u8,
    pub(crate) multi_word_access_points: u8,
    pub(crate) single_word_access_points_for_multi_word_communication: u8,
}

impl From<&[TypedDevice]> for MonitorList {
    fn from(value: &[TypedDevice]) -> Self {
        let mut sorted_devices: Vec<(usize, TypedDevice)> = value
            .into_iter()
            .enumerate()
            .map(|(i, typed_device)| (i, typed_device.clone()))
            .collect();

        // Sort by data_type make the order of devices in the following order; single-word device, multi-word-device, and double-word device.
        // This is defined by the definition of DataType.
        sorted_devices.sort_by_key(|p| p.1.device.address);
        sorted_devices.sort_by_key(|p| p.1.data_type);

        let multi_word_devices = sorted_devices
            .iter()
            .filter(|x| matches!(x.1.data_type.device_size(), DeviceSize::MultiWord(_)));

        let multi_word_access_points = multi_word_devices.clone().count() as u8;

        let single_word_access_points_for_multi_word_communication = multi_word_devices
            .fold(0, |a, b| {
                if let DeviceSize::MultiWord(n) = b.1.data_type.device_size() { a + n } else { a }
            });

        let single_word_access_points: u8 = sorted_devices
            .iter()
            .filter(|x| matches!(x.1.data_type.device_size(), DeviceSize::SingleWord | DeviceSize::Bit))
            .count() as u8 + single_word_access_points_for_multi_word_communication;

        let double_word_access_points: u8 = sorted_devices
            .iter()
            .filter(|x| matches!(x.1.data_type.device_size(), DeviceSize::DoubleWord))
            .count() as u8;

        Self {
            sorted_devices,
            single_word_access_points,
            double_word_access_points,
            multi_word_access_points,
            single_word_access_points_for_multi_word_communication
        }
    }
}

impl MonitorList {
    pub fn new() -> Self {
        const MAX_MONITOR_LIST: usize = 256;
        Self {
            sorted_devices: Vec::with_capacity(MAX_MONITOR_LIST),
            single_word_access_points: 0,
            double_word_access_points: 0,
            multi_word_access_points: 0,
            single_word_access_points_for_multi_word_communication: 0,
        }
    }

    pub fn parse(&self, data: &[u8]) -> Vec<DeviceData> {

        const SINGLE_WORD_BYTELEN: usize = 2;
        const DOUBLE_WORD_BYTELEN: usize = 4;

        let total_access_points: usize = self.single_word_access_points as usize + self.double_word_access_points as usize;
        let single_word_data_byte: usize = (self.single_word_access_points as usize) * SINGLE_WORD_BYTELEN;    // It include single-word data and multi-word data
        let multi_word_data_byte: usize = (self.single_word_access_points_for_multi_word_communication as usize) * SINGLE_WORD_BYTELEN;

        let single_word_data: &[u8] = &data[..(single_word_data_byte - multi_word_data_byte)];
        let multi_word_data: &[u8] = &data[(single_word_data_byte - multi_word_data_byte)..single_word_data_byte];
        let double_word_data: &[u8] = &data[single_word_data_byte..];

        let mut ret: Vec<(usize, DeviceData)> = Vec::with_capacity(total_access_points);

        let mut i = 0;

        for x in single_word_data.chunks_exact(SINGLE_WORD_BYTELEN) {
            ret.push((self.sorted_devices[i].0, DeviceData {
                device: self.sorted_devices[i].1.device,
                data: TypedData::from((x, self.sorted_devices[i].1.data_type)),
            }));
            i += 1;
        }

        let mut buffer_start_addr = 0;
        for _ in 0..self.multi_word_access_points {
            let dev = self.sorted_devices[i];
            let buffer_next_addr = buffer_start_addr + dev.1.data_type.byte_size();
            let data = &multi_word_data[buffer_start_addr..buffer_next_addr];
            ret.push((dev.0, DeviceData {
                device: dev.1.device,
                data: TypedData::from((data, dev.1.data_type))
            }));
            buffer_start_addr = buffer_next_addr;
            i += 1;
        }

        for x in double_word_data.chunks_exact(DOUBLE_WORD_BYTELEN) {
            ret.push((self.sorted_devices[i].0, DeviceData {
                device: self.sorted_devices[i].1.device,
                data: TypedData::from((x, self.sorted_devices[i].1.data_type)),
            }));
            i += 1;
        }

        ret.sort_by_key(|x| x.0);

        let ret: Vec<DeviceData> = ret.into_iter().map(|x| x.1).collect();

        ret
    }
}


#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[cfg_attr(feature = "json-api", serde(rename_all = "camelCase"))]
pub struct MonitorRequest<'a> {
    pub connection_props: &'a SLMP4EConnectionProps,
    pub monitor_device: TypedDevice
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[cfg_attr(feature = "json-api", serde(rename_all = "camelCase"))]
pub struct MonitoredDevice {
    pub socket_addr: SocketAddr,
    pub monitor_device: TypedDevice
}
