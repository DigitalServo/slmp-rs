use crate::CPU;

pub(crate) const fn remote_run() -> [u8; 8] {
    const COMMAND: [u8; 2] = 0x1001u16.to_le_bytes();
    const SUBCOMMAND: [u8; 2] = [0x00, 0x00];

    let operation_mode: [u8; 2] = 0x0003u16.to_le_bytes();
    let clear_mode: u8 = 0x02;
    const SURPLUS_CONSTANT: u8 = 0x00;

    [
        COMMAND[0], COMMAND[1],
        SUBCOMMAND[0], SUBCOMMAND[1],
        operation_mode[0], operation_mode[1],
        clear_mode,
        SURPLUS_CONSTANT
    ]
}

pub(crate) const fn remote_stop() -> [u8; 6] {
    const COMMAND: [u8; 2] = 0x1002u16.to_le_bytes();
    const SUBCOMMAND: [u8; 2] = [0x00, 0x00];
    const SURPLUS_CONSTANT: [u8; 2] = 0x0001u16.to_le_bytes();

    [
        COMMAND[0], COMMAND[1],
        SUBCOMMAND[0], SUBCOMMAND[1],
        SURPLUS_CONSTANT[0], SURPLUS_CONSTANT[1]
    ]
}

pub(crate) const fn remote_pause() -> [u8; 6] {
    const COMMAND: [u8; 2] = 0x1003u16.to_le_bytes();
    const SUBCOMMAND: [u8; 2] = [0x00, 0x00];
    let operation_mode: [u8; 2] = 0x0003u16.to_le_bytes();

    [
        COMMAND[0], COMMAND[1],
        SUBCOMMAND[0], SUBCOMMAND[1],
        operation_mode[0], operation_mode[1]
    ]
}

pub(crate) const fn remote_latch_clear() -> [u8; 6] {
    const COMMAND: [u8; 2] = 0x1005u16.to_le_bytes();
    const SUBCOMMAND: [u8; 2] = [0x00, 0x00];
    const SURPLUS_CONSTANT: [u8; 2] = 0x0001u16.to_le_bytes();

    [
        COMMAND[0], COMMAND[1],
        SUBCOMMAND[0], SUBCOMMAND[1],
        SURPLUS_CONSTANT[0], SURPLUS_CONSTANT[1]
    ]
}

pub(crate) const fn remote_reset() -> [u8; 6] {
    const COMMAND: [u8; 2] = 0x1006u16.to_le_bytes();
    const SUBCOMMAND: [u8; 2] = [0x00, 0x00];
    const SURPLUS_CONSTANT: [u8; 2] = 0x0001u16.to_le_bytes();

    [
        COMMAND[0], COMMAND[1],
        SUBCOMMAND[0], SUBCOMMAND[1],
        SURPLUS_CONSTANT[0], SURPLUS_CONSTANT[1]
    ]
}

pub(crate) const fn get_cpu_type() -> [u8; 4] {
    const COMMAND: [u8; 2] = 0x0101u16.to_le_bytes();
    const SUBCOMMAND: [u8; 2] = [0x00, 0x00];

    [
        COMMAND[0], COMMAND[1],
        SUBCOMMAND[0], SUBCOMMAND[1],
    ]
}

fn validate_password(cpu: &CPU, password: &str) -> std::io::Result<()> {
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

pub(crate) fn unlock_cpu(cpu: &CPU, password: &str) -> std::io::Result<Vec<u8>> {
    const COMMAND: [u8; 2] = 0x1630u16.to_le_bytes();
    const SUBCOMMAND: [u8; 2] = [0x00, 0x00];

    validate_password(cpu, &password)?;

    let password = password.as_bytes();
    let password_len = password.len();
    let password_len_code: [u8; 2] = (password_len as u16).to_le_bytes();

    let data_packet = [password_len_code.as_slice(), password].concat();

    let mut packet: Vec<u8> = Vec::with_capacity(6 + password_len);
    packet.extend_from_slice(&COMMAND);
    packet.extend_from_slice(&SUBCOMMAND);
    packet.extend_from_slice(&data_packet);

    Ok(packet)
}

pub(crate) fn lock_cpu(cpu: &CPU, password: &str) -> std::io::Result<Vec<u8>> {
    const COMMAND: [u8; 2] = 0x1631u16.to_le_bytes();
    const SUBCOMMAND: [u8; 2] = [0x00, 0x00];

    validate_password(cpu, &password)?;

    let password = password.as_bytes();
    let password_len = password.len();
    let password_len_code: [u8; 2] = (password_len as u16).to_le_bytes();

    let data_packet = [password_len_code.as_slice(), password].concat();

    let mut packet: Vec<u8> = Vec::with_capacity(6 + password_len);
    packet.extend_from_slice(&COMMAND);
    packet.extend_from_slice(&SUBCOMMAND);
    packet.extend_from_slice(&data_packet);

    Ok(packet)
}

pub(crate) const ECHO_MESSAGE: [u8; 4] = [0x41, 0x31, 0x47, 0x35];

pub(crate) const fn echo() -> [u8; 10] {
    const COMMAND: [u8; 2] = 0x0619u16.to_le_bytes();
    const SUBCOMMAND: [u8; 2] = [0x00, 0x00];
    const MESSAGE_LEN: [u8; 2] = (4 as u16).to_le_bytes();

    [
        COMMAND[0], COMMAND[1],
        SUBCOMMAND[0], SUBCOMMAND[1],
        MESSAGE_LEN[0], MESSAGE_LEN[1],
        ECHO_MESSAGE[0], ECHO_MESSAGE[1], ECHO_MESSAGE[2], ECHO_MESSAGE[3],
    ]
}
