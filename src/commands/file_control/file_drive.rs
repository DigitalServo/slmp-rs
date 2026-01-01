pub enum FileDriveForR {
    Device,
    SDMemory,
    DataMemory
}

impl FileDriveForR {
    pub(crate) const fn to_drive_code(&self) -> [u8; 2] {
        match self {
            Self::DataMemory => 0x0001u16,
            Self::SDMemory => 0x0002u16,
            Self::Device => 0x0003u16,
        }.to_le_bytes()
    }
}

pub enum FileDriveForQL {
    ProgramMemory,
    SRAMCard,
    SDMemory,
    DefaultRAM,
    DefaultROM
}

impl FileDriveForQL {
    pub(crate) const fn to_drive_code(&self) -> [u8; 2] {
        match self {
            Self::ProgramMemory => 0x0000u16,
            Self::SRAMCard => 0x0001u16,
            Self::SDMemory => 0x0002u16,
            Self::DefaultRAM => 0x0003u16,
            Self::DefaultROM => 0x0003u16,
        }.to_le_bytes()
    }
}

pub enum FileDrive {
    R(FileDriveForR),
    QL(FileDriveForQL)
}

impl FileDrive {
    pub(crate) const fn to_drive_code(&self) -> [u8; 2] {
        match self {
            Self::R(drive) => drive.to_drive_code(),
            Self::QL(drive) => drive.to_drive_code()
        }
    }
}

pub enum FileExtension { DAT, PRG, QPG, PFB, QCD, DCM, QDI, DID }

pub enum FileAttribute {
    ReadOnly(bool),
    ReadWrite(bool),
}

impl FileAttribute {
    pub(crate) const fn to_attribute_code(&self) -> [u8; 2] {
        match self {
            Self::ReadOnly(false) => [0x01, 0x00],
            Self::ReadOnly(true) => [0x21, 0x00],
            Self::ReadWrite(false) => [0x00, 0x00],
            Self::ReadWrite(true) => [0x20, 0x00],
        }
    }
}

pub enum FolderAttribute {
    ReadOnly(bool),
    ReadWrite(bool),
}

impl FolderAttribute {
    pub(crate) const fn to_attribute_code(&self) -> [u8; 2] {
        match self {
            Self::ReadOnly(false) => [0x11, 0x00],
            Self::ReadOnly(true) => [0x31, 0x00],
            Self::ReadWrite(false) => [0x10, 0x00],
            Self::ReadWrite(true) => [0x30, 0x00],
        }
    }
}

pub enum FileOpenMode {Read, Write}

impl FileOpenMode {
    pub(crate) const fn to_mode_code(&self) -> [u8; 2] {
        match self {
            Self::Read => [0x00, 0x00],
            Self::Write => [0x00, 0x01],
        }
    }
}
