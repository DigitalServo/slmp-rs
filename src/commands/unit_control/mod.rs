use crate::{CPU, SLMP4EConnectionProps, commands::{COMMAND_PREFIX_BYTELEN, HEADER_BYTELEN}};

pub(crate) const fn remote_run(connection_props: &SLMP4EConnectionProps) -> [u8; 23] {
    const DATA_BYTELEN: u16 = 4;
    const COMMAND_LEN: u16 = COMMAND_PREFIX_BYTELEN as u16 + DATA_BYTELEN;

    const COMMAND: [u8; 2] = 0x1001u16.to_le_bytes();
    const SUBCOMMAND: [u8; 2] = [0x00, 0x00];

    let operation_mode: [u8; 2] = 0x0003u16.to_le_bytes();
    let clear_mode: u8 = 0x02;
    const SURPLUS_CONSTANT: u8 = 0x00;

    let header: [u8; 15] = connection_props.generate_header(COMMAND_LEN);

    [
        header[0], header[1], header[2], header[3],
        header[4], header[5], header[6], header[7],
        header[8], header[9], header[10], header[11],
        header[12], header[13], header[14],
        COMMAND[0], COMMAND[1],
        SUBCOMMAND[0], SUBCOMMAND[1],
        operation_mode[0], operation_mode[1],
        clear_mode,
        SURPLUS_CONSTANT
    ]
}

pub(crate) const fn remote_stop(connection_props: &SLMP4EConnectionProps) -> [u8; 21] {
    const DATA_BYTELEN: u16 = 2;
    const COMMAND_LEN: u16 = COMMAND_PREFIX_BYTELEN as u16 + DATA_BYTELEN;

    const COMMAND: [u8; 2] = 0x1002u16.to_le_bytes();
    const SUBCOMMAND: [u8; 2] = [0x00, 0x00];
    const SURPLUS_CONSTANT: [u8; 2] = 0x0001u16.to_le_bytes();

    let header: [u8; 15] = connection_props.generate_header(COMMAND_LEN);

    [
        header[0], header[1], header[2], header[3],
        header[4], header[5], header[6], header[7],
        header[8], header[9], header[10], header[11],
        header[12], header[13], header[14],
        COMMAND[0], COMMAND[1],
        SUBCOMMAND[0], SUBCOMMAND[1],
        SURPLUS_CONSTANT[0], SURPLUS_CONSTANT[1]
    ]
}

pub(crate) const fn remote_pause(connection_props: &SLMP4EConnectionProps) -> [u8; 21] {
    const DATA_BYTELEN: u16 = 2;
    const COMMAND_LEN: u16 = COMMAND_PREFIX_BYTELEN as u16 + DATA_BYTELEN;

    const COMMAND: [u8; 2] = 0x1003u16.to_le_bytes();
    const SUBCOMMAND: [u8; 2] = [0x00, 0x00];
    let operation_mode: [u8; 2] = 0x0003u16.to_le_bytes();

    let header: [u8; 15] = connection_props.generate_header(COMMAND_LEN);

    [
        header[0], header[1], header[2], header[3],
        header[4], header[5], header[6], header[7],
        header[8], header[9], header[10], header[11],
        header[12], header[13], header[14],
        COMMAND[0], COMMAND[1],
        SUBCOMMAND[0], SUBCOMMAND[1],
        operation_mode[0], operation_mode[1]
    ]
}

pub(crate) const fn remote_latch_clear(connection_props: &SLMP4EConnectionProps) -> [u8; 21] {
    const DATA_BYTELEN: u16 = 2;
    const COMMAND_LEN: u16 = COMMAND_PREFIX_BYTELEN as u16 + DATA_BYTELEN;

    const COMMAND: [u8; 2] = 0x1005u16.to_le_bytes();
    const SUBCOMMAND: [u8; 2] = [0x00, 0x00];
    const SURPLUS_CONSTANT: [u8; 2] = 0x0001u16.to_le_bytes();

    let header: [u8; 15] = connection_props.generate_header(COMMAND_LEN);

    [
        header[0], header[1], header[2], header[3],
        header[4], header[5], header[6], header[7],
        header[8], header[9], header[10], header[11],
        header[12], header[13], header[14],
        COMMAND[0], COMMAND[1],
        SUBCOMMAND[0], SUBCOMMAND[1],
        SURPLUS_CONSTANT[0], SURPLUS_CONSTANT[1]
    ]
}

pub(crate) const fn remote_reset(connection_props: &SLMP4EConnectionProps) -> [u8; 21] {
    const DATA_BYTELEN: u16 = 2;
    const COMMAND_LEN: u16 = COMMAND_PREFIX_BYTELEN as u16 + DATA_BYTELEN;

    const COMMAND: [u8; 2] = 0x1006u16.to_le_bytes();
    const SUBCOMMAND: [u8; 2] = [0x00, 0x00];
    const SURPLUS_CONSTANT: [u8; 2] = 0x0001u16.to_le_bytes();

    let header: [u8; 15] = connection_props.generate_header(COMMAND_LEN);

    [
        header[0], header[1], header[2], header[3],
        header[4], header[5], header[6], header[7],
        header[8], header[9], header[10], header[11],
        header[12], header[13], header[14],
        COMMAND[0], COMMAND[1],
        SUBCOMMAND[0], SUBCOMMAND[1],
        SURPLUS_CONSTANT[0], SURPLUS_CONSTANT[1]
    ]
}

pub(crate) const fn get_cpu_type(connection_props: &SLMP4EConnectionProps) -> [u8; 19] {
    const DATA_BYTELEN: u16 = 0;
    const COMMAND_LEN: u16 = COMMAND_PREFIX_BYTELEN as u16 + DATA_BYTELEN;

    const COMMAND: [u8; 2] = 0x0101u16.to_le_bytes();
    const SUBCOMMAND: [u8; 2] = [0x00, 0x00];

    let header: [u8; 15] = connection_props.generate_header(COMMAND_LEN);

    [
        header[0], header[1], header[2], header[3],
        header[4], header[5], header[6], header[7],
        header[8], header[9], header[10], header[11],
        header[12], header[13], header[14],
        COMMAND[0], COMMAND[1],
        SUBCOMMAND[0], SUBCOMMAND[1],
    ]
}

fn validate_password(cpu: CPU, password: &str) -> std::io::Result<()> {
    let len = password.len();
    match cpu {
        CPU::Q | CPU::L => if len != 4 {
            return Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Q/L type CPU requires password length of 4"));
        } else { Ok(()) },
        CPU::R => if !(6..=32).contains(&len) {
            return Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "R type CPU requires password length of 6~32"));
        } else { Ok(()) }
    }
}

pub(crate) fn unlock_cpu(connection_props: &SLMP4EConnectionProps, password: &str) -> std::io::Result<Vec<u8>> {

    const COMMAND: [u8; 2] = 0x1630u16.to_le_bytes();
    const SUBCOMMAND: [u8; 2] = [0x00, 0x00];

    validate_password(connection_props.cpu, &password)?;

    let password = password.as_bytes();
    let password_len: [u8; 2] = (password.len() as u16).to_le_bytes();

    let data_packet = [password_len.as_slice(), password].concat();
    let command_len: u16 = COMMAND_PREFIX_BYTELEN as u16 + data_packet.len() as u16;
    let header: [u8; 15] = connection_props.generate_header(command_len);

    let mut packet: Vec<u8> = Vec::with_capacity(HEADER_BYTELEN + command_len as usize);
    packet.extend_from_slice(&header);
    packet.extend_from_slice(&COMMAND);
    packet.extend_from_slice(&SUBCOMMAND);
    packet.extend_from_slice(&data_packet);

    Ok(packet)
}

pub(crate) fn lock_cpu(connection_props: &SLMP4EConnectionProps, password: &str) -> std::io::Result<Vec<u8>> {

    const COMMAND: [u8; 2] = 0x1631u16.to_le_bytes();
    const SUBCOMMAND: [u8; 2] = [0x00, 0x00];

    validate_password(connection_props.cpu, &password)?;

    let password = password.as_bytes();
    let password_len: [u8; 2] = (password.len() as u16).to_le_bytes();

    let data_packet = [password_len.as_slice(), password].concat();
    let command_len: u16 = COMMAND_PREFIX_BYTELEN as u16 + data_packet.len() as u16;
    let header: [u8; 15] = connection_props.generate_header(command_len);

    let mut packet: Vec<u8> = Vec::with_capacity(HEADER_BYTELEN + command_len as usize);
    packet.extend_from_slice(&header);
    packet.extend_from_slice(&COMMAND);
    packet.extend_from_slice(&SUBCOMMAND);
    packet.extend_from_slice(&data_packet);

    Ok(packet)
}

pub(crate) const ECHO_MESSAGE: [u8; 4] = [0x41, 0x31, 0x47, 0x35];

pub(crate) const fn echo(connection_props: &SLMP4EConnectionProps) -> [u8; 25] {

    const COMMAND: [u8; 2] = 0x0619u16.to_le_bytes();
    const SUBCOMMAND: [u8; 2] = [0x00, 0x00];

    const MESSAGE_LEN: [u8; 2] = (4 as u16).to_le_bytes();

    const DATA_PACKET_LEN: usize = 6;
    const COMMAND_LEN: usize = COMMAND_PREFIX_BYTELEN + DATA_PACKET_LEN;
    let header: [u8; 15] = connection_props.generate_header(COMMAND_LEN as u16);

    [
        header[0], header[1], header[2], header[3],
        header[4], header[5], header[6], header[7],
        header[8], header[9], header[10], header[11],
        header[12], header[13], header[14],
        COMMAND[0], COMMAND[1],
        SUBCOMMAND[0], SUBCOMMAND[1],
        MESSAGE_LEN[0], MESSAGE_LEN[1],
        ECHO_MESSAGE[0], ECHO_MESSAGE[1], ECHO_MESSAGE[2], ECHO_MESSAGE[3],
    ]
}
