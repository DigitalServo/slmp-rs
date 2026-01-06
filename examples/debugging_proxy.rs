use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;
use std::error::Error;

const PROXY_LISTEN_ADDR: &str = "127.0.0.1:8000";
const TARGET_ADDR: &str = "192.168.3.10:5007";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Proxy server listening on {}", PROXY_LISTEN_ADDR);
    println!("Forwarding to {}", TARGET_ADDR);

    let listener = TcpListener::bind(PROXY_LISTEN_ADDR).await?;

    let (client_stream, client_addr) = listener.accept().await?;
    println!("\nConnected from {}", client_addr);

    let target_addr = TARGET_ADDR.to_string();
    let handle = proxy_connection(client_stream, target_addr);

    handle.await?;

    Ok(())
}

fn proxy_connection(
    mut client: TcpStream,
    target_addr: String,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut server = match TcpStream::connect(&target_addr).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to connect to target {}: {}", target_addr, e);
                return;
            }
        };
        println!("Connected to {}", target_addr);

        let (mut client_read, mut client_write) = client.split();
        let (mut server_read, mut server_write) = server.split();

        let mut client_buf = [0u8; 4096];
        let mut server_buf = [0u8; 4096];

        loop {
            tokio::select! {
                res = client_read.read(&mut client_buf) => {
                    match res {
                        Ok(0) => break,
                        Ok(n) => {
                            let data = &client_buf[..n];
                            if let Ok(slmp_send_packet) = SlmpCommandPacket::try_from(data) {
                                println!("---\nSend to SLMP Server:\n{}", slmp_send_packet);

                                if let Err(e) = server_write.write_all(data).await {
                                    eprintln!("Failed to forward to server: {}", e);
                                    break;
                                }
                            } else {
                                println!("---\nTry to send inappropriate packet:\n{:02x?}", data);
                            };
                        }
                        Err(e) => {
                            eprintln!("Read from client error: {}", e);
                            break;
                        }
                    }
                }
                res = server_read.read(&mut server_buf) => {
                    match res {
                        Ok(0) => break,
                        Ok(n) => {
                            let data = &server_buf[..n];
                            if let Ok(slmp_received_packet) = SlmpReturnPacket::try_from(data) {
                                println!("---\nReceived From SLMP Server:\n{}", slmp_received_packet);

                                if let Err(e) = client_write.write_all(data).await {
                                    eprintln!("Failed to forward to client: {}", e);
                                    break;
                                }
                            } else {
                                println!("---\nReceived inappropriate packet:\n{:02x?}", data);
                            };
                        }
                        Err(e) => {
                            eprintln!("Read from server error: {}", e);
                            break;
                        }
                    }
                }
            }
        }
        println!("\nConnection closed");
    })
}


#[derive(Debug)]
pub struct SlmpParseError(String);

impl std::fmt::Display for SlmpParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SLMP parse error: {}", self.0)
    }
}

impl std::error::Error for SlmpParseError {}

pub struct SlmpCommandPacket {
    pub request_code: u16,
    pub serial_id: u16,
    pub network_id: u8,
    pub pc_id: u8,
    pub io_id: u16,
    pub area_id: u8,
    pub data_len: u16,
    pub cpu_timer: u16,
    pub command: u16,
    pub subcommand: u16,
    pub data: Vec<u8>,
}

impl TryFrom<&[u8]> for SlmpCommandPacket {

    type Error = SlmpParseError;

    fn try_from(data: &[u8]) -> Result<Self, SlmpParseError> {

        const FIXED_FRAME_LEN: usize = 13;
        const CMDFRAME_PREFIX_FIXED_LEN: usize = 19;

        let packet_len = data.len();

        if packet_len < FIXED_FRAME_LEN {
            return Err(SlmpParseError("Data too short for SLMP header".to_string()));
        }

        let data_len = u16::from_le_bytes([data[11], data[12]]);

        if data_len as usize != packet_len - FIXED_FRAME_LEN {
            return Err(SlmpParseError("Received Invalid Data Frame".to_string()));
        }

        let request_code = u16::from_le_bytes([data[0], data[1]]);
        let serial_id = u16::from_le_bytes([data[2], data[3]]);
        let network_id = data[6];
        let pc_id = data[7];
        let io_id = u16::from_le_bytes([data[8], data[9]]);
        let area_id = data[10];
        let cpu_timer = u16::from_le_bytes([data[13], data[14]]);
        let command: u16 = u16::from_le_bytes([data[15], data[16]]);
        let subcommand: u16 = u16::from_le_bytes([data[17], data[18]]);
        let data = data[CMDFRAME_PREFIX_FIXED_LEN..].to_vec();

        Ok(SlmpCommandPacket {
            request_code,
            serial_id,
            network_id,
            pc_id,
            io_id,
            area_id,
            data_len,
            cpu_timer,
            command,
            subcommand,
            data,
        })
    }
}

impl std::fmt::Display for SlmpCommandPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "\
                Request_code: 0x{:04X}\n\
                Serial ID: 0x{:04X}\n\
                Network ID: 0x{:02X}\n\
                PC ID: 0x{:X}\n\
                Area ID: 0x{:X}\n\
                IO ID: 0x{:04X}\n\
                Data Length: 0x{:04X}\n\
                Command: 0x{:04X}\n\
                Subcommand: 0x{:04X}\n\
                Data: {:02X?}\
            ",
            self.request_code,
            self.serial_id,
            self.network_id,
            self.pc_id,
            self.io_id,
            self.area_id,
            self.data_len,
            self.command,
            self.subcommand,
            self.data
        )
    }
}

pub struct SlmpReturnPacket {
    pub request_code: u16,
    pub serial_id: u16,
    pub network_id: u8,
    pub pc_id: u8,
    pub io_id: u16,
    pub area_id: u8,
    pub data_len: u16,
    pub error: u16,
    pub data: Vec<u8>,
}

impl TryFrom<&[u8]> for SlmpReturnPacket {

    type Error = SlmpParseError;

    fn try_from(data: &[u8]) -> Result<Self, SlmpParseError> {

        const FIXED_FRAME_LEN: usize = 13;
        const RECVFRAME_PREFIX_FIXED_LEN: usize = 15;

        let packet_len = data.len();

        if packet_len < FIXED_FRAME_LEN {
            return Err(SlmpParseError("Data too short for SLMP header".to_string()));
        }

        let data_len = u16::from_le_bytes([data[11], data[12]]);
        if data_len as usize != packet_len - FIXED_FRAME_LEN {
            return Err(SlmpParseError("Received Invalid Data Frame".to_string()));
        }

        let request_code = u16::from_le_bytes([data[0], data[1]]);
        let serial_id = u16::from_le_bytes([data[2], data[3]]);
        let network_id = data[6];
        let pc_id = data[7];
        let io_id = u16::from_le_bytes([data[8], data[9]]);
        let area_id = data[10];
        let error = u16::from_le_bytes([data[13], data[14]]);
        let data = data[RECVFRAME_PREFIX_FIXED_LEN..].to_vec();

        Ok(SlmpReturnPacket {
            request_code,
            serial_id,
            network_id,
            pc_id,
            io_id,
            area_id,
            data_len,
            error,
            data,
        })
    }
}

impl std::fmt::Display for SlmpReturnPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "\
                Request_code: 0x{:04X}\n\
                Serial ID: 0x{:04X}\n\
                Network ID: 0x{:02X}\n\
                PC ID: 0x{:X}\n\
                Area ID: 0x{:X}\n\
                IO ID: 0x{:04X}\n\
                Data Length: 0x{:04X}\n\
                Error: 0x{:02x}\n\
                Data: {:02X?}\
            ",
            self.request_code,
            self.serial_id,
            self.network_id,
            self.pc_id,
            self.io_id,
            self.area_id,
            self.data_len,
            self.error, self.data
        )
    }
}
