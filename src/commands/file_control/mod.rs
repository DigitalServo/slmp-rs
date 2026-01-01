mod file_drive;

use crate::{CPU, SLMP4EConnectionProps, commands::COMMAND_PREFIX_BYTELEN};
use file_drive::FileDriveForQL;

pub(crate) const fn read_file_and_folder_props_for_ql(
    connection_props: &SLMP4EConnectionProps,
    drive: FileDriveForQL,
    start_file_no: u16,
    request_file_len: u16,
    request_folder_len: u16
) -> [u8; 31] {

    const DATA_BYTELEN: u16 = 12;
    const COMMAND_LEN: u16 = COMMAND_PREFIX_BYTELEN as u16 + DATA_BYTELEN;

    const COMMAND: [u8; 2] = 0x1810u16.to_le_bytes();
    let subcommand: [u8; 2] = match connection_props.cpu {
        CPU::Q | CPU::L => [0x00, 0x00],
        CPU::R => [0x40, 0x00],
    };

    const CONSTANT: [u8; 4] = [0x30, 0x30, 0x30, 0x30];

    let drive: [u8; 2] = drive.to_drive_code();
    let start_file: [u8; 2] = start_file_no.to_le_bytes();
    let request_file_len: [u8; 2] = request_file_len.to_le_bytes();
    let request_folder_len: [u8; 2] = request_folder_len.to_le_bytes();

    let header: [u8; 15] = connection_props.generate_header(COMMAND_LEN);

    [
        header[0], header[1], header[2], header[3],
        header[4], header[5], header[6], header[7],
        header[8], header[9], header[10], header[11],
        header[12], header[13], header[14],
        COMMAND[0], COMMAND[1],
        subcommand[0], subcommand[1],
        CONSTANT[0], CONSTANT[1], CONSTANT[2], CONSTANT[3],
        drive[0], drive[1],
        start_file[0], start_file[1],
        request_file_len[0], request_file_len[1],
        request_folder_len[0], request_folder_len[1]
    ]
}


// pub(crate) fn search_file(connection_props: &SLMP4EConnectionProps) -> Vec<u8> {

//     const COMMAND: [u8; 2] = 0x1811u16.to_le_bytes();
//     let subcommand: [u8; 2] = match connection_props.cpu {
//         CPU::Q | CPU::L => [0x00, 0x00],
//         CPU::R => [0x40, 0x00],
//     };

//     vec![
//         COMMAND[0], COMMAND[1],
//         subcommand[0], subcommand[1]
//     ]
// }


// pub(crate) fn create_file(connection_props: &SLMP4EConnectionProps) -> Vec<u8> {

//     const COMMAND: [u8; 2] = 0x1820u16.to_le_bytes();
//     let subcommand: [u8; 2] = match connection_props.cpu {
//         CPU::Q | CPU::L => [0x00, 0x00],
//         CPU::R => [0x40, 0x00],
//     };

//     vec![
//         COMMAND[0], COMMAND[1],
//         subcommand[0], subcommand[1]
//     ]
// }


// pub(crate) fn delete_file(connection_props: &SLMP4EConnectionProps) -> Vec<u8> {

//     const COMMAND: [u8; 2] = 0x1822u16.to_le_bytes();
//     let subcommand: [u8; 2] = match connection_props.cpu {
//         CPU::Q => [0x00, 0x00],
//         CPU::L => [0x04, 0x00],
//         CPU::R => [0x40, 0x00],
//     };

//     vec![
//         COMMAND[0], COMMAND[1],
//         subcommand[0], subcommand[1]
//     ]
// }


// pub(crate) fn copy_file(connection_props: &SLMP4EConnectionProps) -> Vec<u8> {

//     const COMMAND: [u8; 2] = 0x1824u16.to_le_bytes();
//     let subcommand: [u8; 2] = match connection_props.cpu {
//         CPU::Q => [0x00, 0x00],
//         CPU::L => [0x04, 0x00],
//         CPU::R => [0x40, 0x00],
//     };

//     vec![
//         COMMAND[0], COMMAND[1],
//         subcommand[0], subcommand[1]
//     ]
// }


// pub(crate) fn edit_file_attribute(connection_props: &SLMP4EConnectionProps) -> Vec<u8> {

//     const COMMAND: [u8; 2] = 0x1825u16.to_le_bytes();
//     let subcommand: [u8; 2] = match connection_props.cpu {
//         CPU::Q => [0x00, 0x00],
//         CPU::L => [0x04, 0x00],
//         CPU::R => [0x40, 0x00],
//     };

//     vec![
//         COMMAND[0], COMMAND[1],
//         subcommand[0], subcommand[1]
//     ]
// }


// pub(crate) fn edit_file_motified_data(connection_props: &SLMP4EConnectionProps) -> Vec<u8> {

//     const COMMAND: [u8; 2] = 0x1826u16.to_le_bytes();
//     let subcommand: [u8; 2] = match connection_props.cpu {
//         CPU::Q | CPU::L=> [0x00, 0x00],
//         CPU::R => [0x40, 0x00],
//     };

//     vec![
//         COMMAND[0], COMMAND[1],
//         subcommand[0], subcommand[1]
//     ]
// }


// pub(crate) fn open_file(connection_props: &SLMP4EConnectionProps) -> Vec<u8> {

//     const COMMAND: [u8; 2] = 0x1827u16.to_le_bytes();
//     let subcommand: [u8; 2] = match connection_props.cpu {
//         CPU::Q => [0x00, 0x00],
//         CPU::L => [0x04, 0x00],
//         CPU::R => [0x40, 0x00],
//     };

//     vec![
//         COMMAND[0], COMMAND[1],
//         subcommand[0], subcommand[1]
//     ]
// }


// pub(crate) fn read_file(connection_props: &SLMP4EConnectionProps) -> Vec<u8> {

//     const COMMAND: [u8; 2] = 0x1828u16.to_le_bytes();
//     const SUBCOMMAND: [u8; 2] = [0x00, 0x00];

//     vec![
//         COMMAND[0], COMMAND[1],
//         SUBCOMMAND[0], SUBCOMMAND[1]
//     ]
// }


// pub(crate) fn write_file(connection_props: &SLMP4EConnectionProps) -> Vec<u8> {

//     const COMMAND: [u8; 2] = 0x1829u16.to_le_bytes();
//     const SUBCOMMAND: [u8; 2] = [0x00, 0x00];

//     vec![
//         COMMAND[0], COMMAND[1],
//         SUBCOMMAND[0], SUBCOMMAND[1]
//     ]
// }


// pub(crate) fn close_file(connection_props: &SLMP4EConnectionProps) -> Vec<u8> {

//     const COMMAND: [u8; 2] = 0x182Au16.to_le_bytes();
//     const SUBCOMMAND: [u8; 2] = [0x00, 0x00];

//     vec![
//         COMMAND[0], COMMAND[1],
//         SUBCOMMAND[0], SUBCOMMAND[1]
//     ]
// }
