mod commands;
mod data;
mod device;
mod manager;
mod monitor;


use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream};
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};
use serde::{Deserialize, Serialize};

use crate::commands::device_access::{read::*, write::*};
use crate::commands::unit_control;

use device::DeviceSize;

// Public
pub use data::{DataType, TypedData, string::{PLCString, PLCSTRING_QUERY_SPLITTER}};
pub use device::{AccessType, Device, DeviceType, DeviceData, DeviceBlock, BlockedDeviceData, TypedDevice, PLCData};
pub use monitor::{MonitorList, MonitorRequest, MonitoredDevice};
pub use manager::{SLMPConnectionManager, SLMPWorker};


// Constants
const BUFSIZE: usize = 1024;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(1);
const DEFAULT_SEND_TIMEOUT_SEC: Duration = Duration::from_secs(1);
const DEFAULT_RECV_TIMEOUT_SEC: Duration = Duration::from_secs(1);

macro_rules! invalidDataError {
    ($msg:expr) => {
        std::io::Error::new(std::io::ErrorKind::InvalidData, $msg)
    };
}
macro_rules! check {
    ($data:expr, $idx:expr, $expected:expr, $msg:expr) => {
        if $data[$idx] != $expected {
            return Err(invalidDataError!($msg));
        }
    };
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "json-api", serde(rename_all = "PascalCase"))]
pub enum CPU {Q, R, L}


#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "json-api", serde(rename_all = "camelCase"))]
pub struct SLMP4EConnectionProps {
    pub ip: String,
    pub port : u16,
    pub cpu: CPU,
    pub serial_id: u16,
    pub network_id: u8,
    pub pc_id: u8,
    pub io_id: u16,
    pub area_id: u8,
    pub cpu_timer: u16,
}

impl<'a> TryFrom<&'a SLMP4EConnectionProps> for SocketAddr {
    type Error = std::io::Error;
    fn try_from(value: &'a SLMP4EConnectionProps) -> Result<Self, Self::Error> {
        let ip: IpAddr = value.ip.parse::<IpAddr>()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
        let port: u16 = value.port;
        Ok(SocketAddr::new(ip, port))
    }
}

impl SLMP4EConnectionProps {
    #[inline(always)]
    const fn generate_header(&self, command_len: u16) -> [u8; 15] {
        const BLANK_CODE: u8 = 0x00;
        const REQUEST_CODE: [u8; 2] = [0x54, 0x00];

        let serial_id: [u8; 2] = self.serial_id.to_le_bytes();
        let io_id: [u8; 2] = self.io_id.to_le_bytes();
        let cpu_timer: [u8; 2] = self.cpu_timer.to_le_bytes();
        let command_len: [u8; 2] = command_len.to_le_bytes();

        [
            REQUEST_CODE[0], REQUEST_CODE[1],
            serial_id[0], serial_id[1],
            BLANK_CODE, BLANK_CODE,
            self.network_id,
            self.pc_id,
            io_id[0], io_id[1],
            self.area_id,
            command_len[0], command_len[1],
            cpu_timer[0], cpu_timer[1],
        ]

    }
}

#[derive(Clone)]
pub struct SLMPClient {
    connection_props: SLMP4EConnectionProps,
    stream: Arc<Mutex<Option<TcpStream>>>,
    send_timeout: Duration,
    recv_timeout: Duration,
    buffer: [u8; BUFSIZE],
}

impl SLMPClient {
    pub fn new(connection_props: SLMP4EConnectionProps) -> Self {
        Self {
            connection_props,
            stream: Arc::new(Mutex::new(None)),
            send_timeout: DEFAULT_SEND_TIMEOUT_SEC,
            recv_timeout: DEFAULT_RECV_TIMEOUT_SEC,
            buffer: [0; BUFSIZE],
        }
    }

    pub async fn close(&self) {
        let mut lock = self.stream.lock().await;
        if let Some(mut stream) = lock.take() {
            let _ = stream.shutdown().await;
        }
    }

    #[allow(dead_code)]
    pub fn set_send_timeout(&mut self, dur: Duration) {
        self.send_timeout = dur;
    }

    #[allow(dead_code)]
    pub fn set_recv_timeout(&mut self, dur: Duration) {
        self.recv_timeout = dur;
    }

    pub async fn connect(&self) -> std::io::Result<()> {
        self.close().await;

        let addr: (&str, u16) = (&self.connection_props.ip, self.connection_props.port);
        let socket_addr: SocketAddr = addr
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "resolve failed"))?;

        let stream: TcpStream = tokio::time::timeout(CONNECT_TIMEOUT, TcpStream::connect(socket_addr))
            .await.map_err(|_| std::io::Error::new(std::io::ErrorKind::TimedOut,"Connect Failed (Timeout)"))
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::TimedOut,"Connect Failed (Timeout)"))??;

        let mut lock = self.stream.lock().await;
        *lock = Some(stream);

        Ok(())
    }

    async fn request_response(&mut self, msg: &[u8]) -> std::io::Result<&[u8]> {
        const RECVFRAME_PREFIX_FIXED_LEN: usize = 15;

        let mut stream = self.stream.lock().await;
        let stream = stream.as_mut().ok_or(std::io::Error::new(std::io::ErrorKind::NotConnected, "Not Connected"))?;

        timeout(self.send_timeout, stream.write_all(&msg)).await
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::TimedOut,"Send Failed (Timeout)"))??;

        let bytes_read = timeout(self.recv_timeout, stream.read(&mut self.buffer)).await
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::TimedOut,"Read Failed (Timeout)"))??;

        self.validate_response(&self.buffer[..bytes_read])?;

        Ok(&self.buffer[RECVFRAME_PREFIX_FIXED_LEN..bytes_read])
    }

    fn validate_response(&self, data: &[u8]) -> std::io::Result<()> {
        const FIXED_FRAME_LEN: usize = 13;
        const RESPONSE_CODE: [u8; 2] = [0xD4, 0x00];
        const BLANK_CODE: u8 = 0x00;

        let data_len: usize = data.len();
        if data_len < FIXED_FRAME_LEN {
            return Err(invalidDataError!("Received Invalid Length Data"));
        }

        let data_block_len: usize = u16::from_le_bytes([data[11], data[12]]) as usize;
        if data_block_len != data_len - FIXED_FRAME_LEN {
            return Err(invalidDataError!("Received Invalid Data Frame"));
        }

        let error = u16::from_le_bytes([data[13], data[14]]);
        if error != 0 {
            let error_msg = match error {
                0xC059 => "WrongCommand",
                0xC05C => "WrongFormat",
                0xC061 => "WrongLength",
                0xCEE0 => "Busy",
                0xCEE1 => "ExceedReqLength",
                0xCEE2 => "ExceedRespLength",
                0xCF10 => "ServerNotFound",
                0xCF20 => "WrongConfigItem",
                0xCF30 => "PrmIDNotFound",
                0xCF31 => "NotStartExclusiveWrite",
                0xCF70 => "RelayFailure",
                0xCF71 => "TimeoutError",
                _ => "Unknown Error",
            };
            return Err(invalidDataError!(format!("SLMP Returns Error: {error_msg} (0x{error:X})")));
        }

        check!(data, 0..2, RESPONSE_CODE, "Received Invalid Response Data");
        check!(data, 2..4, self.connection_props.serial_id.to_le_bytes(), "Received Invalid Serial ID");
        check!(data, 4..6, [BLANK_CODE; 2], "Received Invalid Blank Code");
        check!(data, 6, self.connection_props.network_id, "Received Invalid Network ID");
        check!(data, 7, self.connection_props.pc_id, "Received Invalid PC ID");
        check!(data, 8..10, self.connection_props.io_id.to_le_bytes(), "Received Invalid IO ID");
        check!(data,10, self.connection_props.area_id, "Received Invalid Area ID");

        Ok(())
    }

    /* Unit Control */

    pub async fn run_cpu(&mut self) -> std::io::Result<()> {
        let cmd = unit_control::remote_run(&self.connection_props);
        self.request_response(&cmd).await.map(|_| ())
    }

    pub async fn stop_cpu(&mut self) -> std::io::Result<()> {
        let cmd = unit_control::remote_stop(&self.connection_props);
        self.request_response(&cmd).await.map(|_| ())
    }

    pub async fn pause_cpu(&mut self) -> std::io::Result<()> {
        let cmd = unit_control::remote_pause(&self.connection_props);
        self.request_response(&cmd).await.map(|_| ())
    }

    pub async fn clear_latch(&mut self) -> std::io::Result<()> {
        let cmd = unit_control::remote_latch_clear(&self.connection_props);
        self.request_response(&cmd).await.map(|_| ())
    }

    pub async fn reset_cpu(&mut self) -> std::io::Result<()> {
        let cmd = unit_control::remote_reset(&self.connection_props);
        self.request_response(&cmd).await.map(|_| ())
    }

    pub async fn get_cpu_type(&mut self) -> std::io::Result<String> {
        let cmd = unit_control::get_cpu_type(&self.connection_props);
        let ret = self.request_response(&cmd).await?;

        const END_CODE: u8 = 0x20;
        let end_pos = ret.iter().position(|&b| b == END_CODE).unwrap_or(ret.len());
        let cpu_type = String::from_utf8_lossy(&ret[..end_pos]).into_owned();

        Ok(cpu_type)
    }

    pub async fn lock_cpu(&mut self, password: &str) -> std::io::Result<()> {
        let cmd = unit_control::lock_cpu(&self.connection_props, password)?;
        self.request_response(&cmd).await.map(|_| ())
    }

    pub async fn unlock_cpu(&mut self, password: &str) -> std::io::Result<()> {
        let cmd = unit_control::unlock_cpu(&self.connection_props, password)?;
        self.request_response(&cmd).await.map(|_| ())
    }

    pub async fn echo(&mut self) -> std::io::Result<()> {
        let cmd = unit_control::echo(&self.connection_props);
        let recv = self.request_response(&cmd).await
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::NetworkDown, "Echo response did not return in time"))?;

        if &recv[2..6] ==  unit_control::ECHO_MESSAGE {
            Ok(())
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Echo mismatch, send: {:02x?}, received: {:02x?}", unit_control::ECHO_MESSAGE, &recv[2..6])
            ))
        }
    }

    /* File Control */

    /* Device Access */

    pub async fn bulk_write<'a>(&mut self, start_device: Device, data: &'a [TypedData]) -> std::io::Result<()>
    {
        if data.len() > 0 {
            let query = SLMPBulkWriteQuery {
                connection_props: &self.connection_props,
                start_device,
                data,
            };
            let cmd: SLMPBulkWriteCommand = query.into();

            self.request_response(&cmd).await.map(|_| ())?;
        }

        Ok(())
    }


    pub async fn random_write<'a>(&mut self, data: &'a [DeviceData]) -> std::io::Result<()>
    {
        // Word access
        let mut sorted_word_data: Vec<DeviceData> = data.iter()
            .filter(|x| !matches!(x.data, TypedData::Bool(_)))
            .copied()
            .collect();
        sorted_word_data.sort_by_key(|p| p.device.address);
        sorted_word_data.sort_by_key(|p| p.data.get_type());

        // Bit access
        let mut sorted_bit_data: Vec<DeviceData> = data.iter()
            .filter(|x| matches!(x.data, TypedData::Bool(_)))
            .copied()
            .collect();
        sorted_bit_data.sort_by_key(|p| p.device.address);

        let single_word_access_points_for_multi_word_communication = sorted_word_data
            .iter()
            .filter(|x| matches!(x.data.get_type().device_size(), DeviceSize::MultiWord(_)))
            .fold(0, |a, b| {
                if let DeviceSize::MultiWord(n) = b.data.get_type().device_size() { a + n } else { a }
            });

        let single_word_access_points: u8 = sorted_word_data
            .iter()
            .filter(|x| x.data.get_type().device_size() == DeviceSize::SingleWord)
            .count() as u8 + single_word_access_points_for_multi_word_communication;

        let double_word_access_points: u8 = sorted_word_data
            .iter()
            .filter(|x| x.data.get_type().device_size() == DeviceSize::DoubleWord)
            .count() as u8;

        let bit_access_points: u8 = sorted_bit_data
            .iter()
            .filter(|x| x.data.get_type().device_size() == DeviceSize::Bit).count() as u8;

        if single_word_access_points + double_word_access_points > 0 {
            let query = SLMPRandomWriteQuery {
                connection_props: &self.connection_props,
                sorted_data: &sorted_word_data,
                access_type: AccessType::Word,
                bit_access_points: 0,
                single_word_access_points,
                double_word_access_points,
            };
            let cmd: SLMPRandomWriteCommand = query.into();

            self.request_response(&cmd).await.map(|_| ())?;
        }

        if bit_access_points > 0 {
            let query = SLMPRandomWriteQuery {
                connection_props: &self.connection_props,
                sorted_data: &sorted_bit_data,
                access_type: AccessType::Bit,
                bit_access_points,
                single_word_access_points: 0,
                double_word_access_points: 0,
            };
            let cmd: SLMPRandomWriteCommand = query.into();

            self.request_response(&cmd).await.map(|_| ())?;
        }

        Ok(())
    }

    pub async fn block_write<'a>(&mut self, data: &'a [BlockedDeviceData<'a>]) -> std::io::Result<()>
    {
        let mut sorted_data = data.to_vec();
        sorted_data.sort_by_key(|p| p.access_type);

        let word_access_points: u8 = sorted_data.iter().filter(|x| x.access_type == AccessType::Word).count() as u8;
        let bit_access_points: u8 = sorted_data.iter().filter(|x| x.access_type == AccessType::Bit).count() as u8;

        if word_access_points + bit_access_points > 0 {
            let query = SLMPBlockWriteQuery {
                connection_props: &self.connection_props,
                sorted_data: &sorted_data,
                word_access_points,
                bit_access_points
            };
            let cmd: SLMPBlockWriteCommand = query.into();

            self.request_response(&cmd).await.map(|_| ())?;
        }

        Ok(())
    }

    pub async fn bulk_read(&mut self, start_device: Device, device_num: usize, data_type: DataType) -> std::io::Result<Vec<DeviceData>>
    {
        let query = SLMPBulkReadQuery {
            connection_props: &self.connection_props,
            start_device,
            device_num,
            data_type,
        };
        let cmd: SLMPBulkReadCommand = query.into();

        let recv: &[u8] = &(self.request_response(&cmd).await?);

        match data_type {
            DataType::Bool => {
                let device_type = start_device.device_type;
                let start_address = start_device.address;

                let mut ret: Vec<DeviceData> = Vec::with_capacity(device_num);
                for (i, data) in recv.iter().flat_map(|&x| [(x >> 4) & 0x01, x & 0x01]).enumerate() {
                    if i < device_num {
                        ret.push(DeviceData {
                            device: Device {device_type, address: start_address + i},
                            data: TypedData::Bool(if data == 1 { true } else { false })
                        })
                    }
                }
                Ok(ret)
            }
            _ => {
                let chunk_size = data_type.byte_size();
                let skip_address = chunk_size / 2;
                let device_type = start_device.device_type;
                let start_address = start_device.address;

                let mut ret: Vec<DeviceData> = Vec::with_capacity(device_num);
                for (i, data) in recv.chunks_exact(chunk_size).enumerate() {
                    ret.push(DeviceData {
                        device: Device {device_type, address: start_address + skip_address * i},
                        data: TypedData::from((data, data_type))
                    });
                }

                Ok(ret)
            }
        }
    }

    pub async fn random_read(&mut self, devices: &[TypedDevice]) -> std::io::Result<Vec<DeviceData>>
    {
        let monitor_list = MonitorList::from(devices);

        let query = SLMPRandomReadQuery {
            connection_props: &self.connection_props,
            monitor_list: &monitor_list
        };
        let cmd: SLMPRandomReadCommand = query.into();

        let recv: &[u8] = &(self.request_response(&cmd).await?);

        Ok(monitor_list.parse(&recv))
    }


    pub async fn block_read(&mut self, device_blocks: &[DeviceBlock]) -> std::io::Result<Vec<DeviceData>>
    {
        const WORD_RESPONSE_BYTEELEN: usize = 2;
        const BIT_RESPONSE_BYTEELEN: usize = 1;

        let mut sorted_block = device_blocks.to_vec();
        sorted_block.sort_by_key(|p| p.start_device.address);
        sorted_block.sort_by_key(|p| p.access_type);

        let word_access_points: u8 = sorted_block.iter().filter(|x| x.access_type == AccessType::Word).count() as u8;
        let bit_access_points: u8 = sorted_block.iter().filter(|x| x.access_type == AccessType::Bit).count() as u8;

        let query = SLMPBlockReadQuery {
            connection_props: &self.connection_props,
            sorted_block: &sorted_block,
            word_access_points,
            bit_access_points,
        };
        let cmd: SLMPBlockReadCommand = query.into();

        let recv: &[u8] = &(self.request_response(&cmd).await?);

        let data_num = sorted_block.iter().fold(0, |a, b| a + b.size);
        let mut ret: Vec<DeviceData> = Vec::with_capacity(data_num);

        let mut read_addr = 0;

        for block in &sorted_block {
            let start_address = block.start_device.address;
            let device_type = block.start_device.device_type;
            let block_bytelen = match block.access_type {
                AccessType::Word => WORD_RESPONSE_BYTEELEN * block.size,
                AccessType::Bit => BIT_RESPONSE_BYTEELEN * div_ceil(block.size, 8)
            };
            let blocked_data = &recv[read_addr..(read_addr + block_bytelen)];
            read_addr += block_bytelen;

            match block.access_type {
                AccessType::Word => {
                    for (i, x) in blocked_data.chunks_exact(WORD_RESPONSE_BYTEELEN).enumerate() {
                        ret.push(DeviceData{
                            device: Device {device_type, address: start_address + i},
                            data: TypedData::from((x, DataType::U16)),
                        });
                    }
                },
                AccessType::Bit => {
                    for (i, x) in blocked_data.chunks_exact(BIT_RESPONSE_BYTEELEN).enumerate() {
                        for (j, y) in u8_to_bits(x[0]).into_iter().enumerate() {
                            let bit_index = 8 * i + j;
                            if bit_index < block.size {
                                ret.push(DeviceData{
                                    device: Device {device_type, address: start_address + bit_index},
                                    data: TypedData::Bool(y),
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(ret)
    }

    pub async fn monitor_register(&mut self, devices: &[TypedDevice]) -> std::io::Result<MonitorList>
    {
        let monitor_list = MonitorList::from(devices);
        let query = SLMPMonitorRegisterQuery {
            connection_props: &self.connection_props,
            monitor_list: &monitor_list
        };
        let cmd: SLMPMonitorRegisterCommand = query.into();
        self.request_response(&cmd).await?;

        Ok(monitor_list)
    }

    pub async fn monitor_read(&mut self, monitor_list: &MonitorList) -> std::io::Result<Vec<DeviceData>>
    {
        let query = SLMPMonitorReadQuery {
            connection_props: &self.connection_props
        };
        let cmd: SLMPMonitorReadCommand = query.into();
        let recv: &[u8] = &(self.request_response(&cmd).await?);

        Ok(monitor_list.parse(&recv))
    }

}


#[inline(always)]
pub(crate) const fn div_ceil(a: usize, b: usize) -> usize {
    (a + b - 1) / b
}

#[inline(always)]
pub(crate) const fn u8_to_bits(n: u8) -> [bool; 8] {
    [ n & 0x01 != 0, n & 0x02 != 0, n & 0x04 != 0, n & 0x08 != 0, n & 0x10 != 0, n & 0x20 != 0, n & 0x40 != 0, n & 0x80 != 0, ]
}

#[inline(always)]
pub(crate) const fn bits_to_u8(bits: [bool; 8]) -> u8 {
    ((bits[0] as u8) << 0) |
    ((bits[1] as u8) << 1) |
    ((bits[2] as u8) << 2) |
    ((bits[3] as u8) << 3) |
    ((bits[4] as u8) << 4) |
    ((bits[5] as u8) << 5) |
    ((bits[6] as u8) << 6) |
    ((bits[7] as u8) << 7)
}

#[inline(always)]
pub(crate) const fn u16_to_bits(n: u16) -> [bool; 16] {
    let bytes: [u8; 2] = n.to_le_bytes();

    let low_bits = u8_to_bits(bytes[0]);
    let high_bits = u8_to_bits(bytes[1]);

    [
        low_bits[0], low_bits[1], low_bits[2], low_bits[3], low_bits[4], low_bits[5], low_bits[6], low_bits[7],
        high_bits[0], high_bits[1], high_bits[2], high_bits[3], high_bits[4], high_bits[5], high_bits[6], high_bits[7],
    ]
}

#[inline(always)]
pub(crate) const fn bits_to_u16(bits: [bool; 16]) -> u16 {
    let low_bits = [bits[0], bits[1], bits[2], bits[3], bits[4], bits[5], bits[6], bits[7]];
    let high_bits = [bits[8], bits[9], bits[10], bits[11], bits[12], bits[13], bits[14], bits[15]];

    let high_byte = bits_to_u8(high_bits);
    let low_byte = bits_to_u8(low_bits);

    u16::from_be_bytes([low_byte, high_byte])
}
