use serde::{Deserialize, Serialize};

mod commands;
mod device;
mod manager;

use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream};
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};

use commands::read::*;
use commands::write::*;

use device::DeviceSize;

// Public
pub use device::{AccessType, Device, DeviceType, DeviceData, DeviceBlock, BlockedDeviceData, TypedDevice};
pub use manager::{SLMPConnectionManager, SLMPWorker, MonitorDevice, PLCData, PollingInterval};

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
pub enum CPU {A, Q, R, F, L}

/// Available data type for SLMP communication.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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
    const fn byte_size(&self) -> usize {
        match self {
            DataType::Bool => 1,
            DataType::U16 | DataType::I16 => 2,
            DataType::U32 | DataType::I32 | DataType::F32=> 4,
            DataType::F64 => 8,
        }
    }

    #[inline(always)]
    const fn device_size(&self) -> DeviceSize {
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
pub enum TypedData {
    Bool(bool),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    F32(f32),
    F64(f64),
}

impl TypedData {
    #[inline(always)]
    const fn from(value: &[u8], data_type: DataType) -> Self {
        match data_type {
            DataType::Bool => TypedData::Bool(value[0] == 1),
            DataType::U16 => TypedData::U16(u16::from_le_bytes([value[0], value[1]])),
            DataType::I16 => TypedData::I16(i16::from_le_bytes([value[0], value[1]])),
            DataType::U32 => TypedData::U32(u32::from_le_bytes([value[0], value[1], value[2], value[3]])),
            DataType::I32 => TypedData::I32(i32::from_le_bytes([value[0], value[1], value[2], value[3]])),
            DataType::F32 => TypedData::F32(f32::from_le_bytes([value[0], value[1], value[2], value[3]])),
            DataType::F64 => TypedData::F64(f64::from_le_bytes([value[0], value[1], value[2], value[3], value[4], value[5], value[6], value[7]])),
        }
    }

    #[inline(always)]
    const fn to_bytes(&self) -> &[u8] {
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
    const fn get_type(&self) -> DataType {
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


#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SLMP4EConnectionProps {
    pub ip: &'static str,
    pub port : u16,
    pub cpu: CPU,
    pub serial_id: u16,
    pub network_id: u8,
    pub pc_id: u8,
    pub io_id: u16,
    pub area_id: u8,
    pub cpu_timer: u16,
}

impl TryFrom<SLMP4EConnectionProps> for SocketAddr {
    type Error = std::io::Error;
    fn try_from(value: SLMP4EConnectionProps) -> Result<Self, Self::Error> {
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
            REQUEST_CODE[0],
            REQUEST_CODE[1],
            serial_id[0],
            serial_id[1],
            BLANK_CODE,
            BLANK_CODE,
            self.network_id,
            self.pc_id,
            io_id[0],
            io_id[1],
            self.area_id,
            command_len[0],
            command_len[1],
            cpu_timer[0],
            cpu_timer[1],
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
        
        let addr: (&str, u16) = (self.connection_props.ip, self.connection_props.port);
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

    pub async fn bulk_write<'a>(&mut self, start_device: Device, data: &'a [TypedData]) -> std::io::Result<()>
    {
        let query = SLMPBulkWriteQuery {
            connection_props: self.connection_props,
            start_device,
            data,
        };
        let cmd: SLMPBulkWriteCommand = query.try_into()?;

        self.request_response(&cmd).await.map(|_| ())
    }


    pub async fn random_write<'a>(&mut self, data: &'a [DeviceData]) -> std::io::Result<()>
    {
        let mut sorted_data: Vec<DeviceData> = data.iter()
            .filter(|x| !matches!(x.data, TypedData::F64(_)))
            .copied()
            .collect();
        sorted_data.sort_by_key(|p| p.device.address);
        sorted_data.sort_by_key(|p| p.data.get_type());

        let bit_access_points: u8 = sorted_data.iter().filter(|x| x.data.get_type().device_size() == DeviceSize::Bit).count() as u8;
        let single_word_access_points: u8 = sorted_data.iter().filter(|x| x.data.get_type().device_size() == DeviceSize::SingleWord).count() as u8;
        let double_word_access_points: u8 = sorted_data.iter().filter(|x| x.data.get_type().device_size() == DeviceSize::DoubleWord).count() as u8;
        
        let query = SLMPRandomWriteQuery {
            connection_props: self.connection_props,
            sorted_data: &sorted_data,
            bit_access_points,
            single_word_access_points,
            double_word_access_points
        };
        let cmd: SLMPRandomWriteCommand = query.try_into()?;

        self.request_response(&cmd).await.map(|_| ())
    }

    pub async fn block_write<'a>(&mut self, data: &'a [BlockedDeviceData<'a>]) -> std::io::Result<()>
    {
        let mut sorted_data = data.to_vec();
        sorted_data.sort_by_key(|p| p.access_type);

        let word_access_points: u8 = sorted_data.iter().filter(|x| x.access_type == AccessType::Word).count() as u8;
        let bit_access_points: u8 = sorted_data.iter().filter(|x| x.access_type == AccessType::Bit).count() as u8;
        
        let query = SLMPBlockWriteQuery {
            connection_props: self.connection_props,
            sorted_data: &sorted_data,
            word_access_points,
            bit_access_points
        };
        let cmd: SLMPBlockWriteCommand = query.try_into()?;

        self.request_response(&cmd).await.map(|_| ())
    }

    pub async fn bulk_read(&mut self, start_device: Device, device_num: usize, data_type: DataType) ->  std::io::Result<Vec<DeviceData>> 
    {
        let query = SLMPBulkReadQuery {
            connection_props: self.connection_props,
            start_device,
            device_num,
            data_type,
        };
        let cmd: SLMPBulkReadCommand = query.try_into()?;

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
                        data: TypedData::from(data, data_type)
                    });
                }

                Ok(ret)
            }
        }
    }

    pub async fn random_read(&mut self, devices: &[TypedDevice]) ->  std::io::Result<Vec<DeviceData>> 
    {
        const SINGLE_WORD_BYTELEN: usize = 2;
        const DOUBLE_WORD_BYTELEN: usize = 4;

        let mut sorted_devices: Vec<TypedDevice> = devices.iter()
            .filter(|x| !matches!(x.data_type, DataType::F64 | DataType::Bool))
            .copied()
            .collect();
        sorted_devices.sort_by_key(|p| p.device.address);
        sorted_devices.sort_by_key(|p| p.data_type);

        let single_word_access_points: u8 = sorted_devices.iter().filter(|x| x.data_type.device_size() == DeviceSize::SingleWord).count() as u8;
        let double_word_access_points: u8 = sorted_devices.iter().filter(|x| x.data_type.device_size() == DeviceSize::DoubleWord).count() as u8;
        let total_access_points: usize = (single_word_access_points + double_word_access_points) as usize;

        let single_word_data_byte_len: usize = single_word_access_points as usize * SINGLE_WORD_BYTELEN;

        let query = SLMPRandomReadQuery {
            connection_props: self.connection_props,
            sorted_devices: &sorted_devices,
            single_word_access_points,
            double_word_access_points,
        };
        let cmd: SLMPRandomReadCommand = query.try_into()?;

        let recv: &[u8] = &(self.request_response(&cmd).await?);

        let single_word_data: &[u8] = &recv[..single_word_data_byte_len];
        let double_word_data: &[u8] = &recv[single_word_data_byte_len..];

        let mut ret: Vec<DeviceData> = Vec::with_capacity(total_access_points);

        let mut i = 0;

        for x in single_word_data.chunks_exact(SINGLE_WORD_BYTELEN) {
            ret.push(DeviceData {
                device: sorted_devices[i].device,
                data: TypedData::from(x, sorted_devices[i].data_type),
            });
            i += 1;
        }
        for x in double_word_data.chunks_exact(DOUBLE_WORD_BYTELEN) {
            ret.push(DeviceData {
                device: sorted_devices[i].device,
                data: TypedData::from(x, sorted_devices[i].data_type),
            });
            i += 1;
        }

        Ok(ret)
    }


    pub async fn block_read(&mut self, device_blocks: &[DeviceBlock]) ->  std::io::Result<Vec<DeviceData>> 
    {
        const WORD_RESPONSE_BYTEELEN: usize = 2;
        const BIT_RESPONSE_BYTEELEN: usize = 1;

        let mut sorted_block = device_blocks.to_vec();
        sorted_block.sort_by_key(|p| p.start_device.address);
        sorted_block.sort_by_key(|p| p.access_type);

        let word_access_points: u8 = sorted_block.iter().filter(|x| x.access_type == AccessType::Word).count() as u8;
        let bit_access_points: u8 = sorted_block.iter().filter(|x| x.access_type == AccessType::Bit).count() as u8;

        let query = SLMPBlockReadQuery {
            connection_props: self.connection_props,
            sorted_block: &sorted_block,
            word_access_points,
            bit_access_points,
        };
        let cmd: SLMPBlockReadCommand = query.try_into()?;

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
                            data: TypedData::from(x, DataType::U16),
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

}


#[inline(always)]
pub(crate) const fn div_ceil(a: usize, b: usize) -> usize {
    (a + b - 1) / b
}

#[inline(always)]
pub(crate) const fn u8_to_bits(n: u8) -> [bool; 8] {
    [ n & 1 != 0, n & 2 != 0, n & 4 != 0, n & 8 != 0, n & 16 != 0, n & 32 != 0, n & 64 != 0, n & 128 != 0 ]
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