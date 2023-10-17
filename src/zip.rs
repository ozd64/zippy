use byteorder::{ByteOrder, LittleEndian};
use std::error::Error;
use std::fmt::Display;
use std::io::{Read, Seek, SeekFrom};

use crate::date_time::ZipDateTime;

const MIN_EOF_CENTRAL_DIR_SIZE: u64 = 0x16;
const MIN_CENTRAL_DIR_SIZE: u64 = 0x2E;
const EOF_CENTRAL_DIR_SIGN: u32 = 0x06054b50;
const CENTRAL_DIR_SIGN: u32 = 0x02014b50;

#[derive(Debug, PartialEq, Eq)]
enum EndOfCentralDirectoryError {
    InvalidZipFile(u64),
    InvalidSignature(u32),
    EmptyZipFile,
    IOError(String),
}

#[derive(Debug, PartialEq, Eq)]
enum ZipFileError {
    InvalidSignature(u32),
    UnsupportedZipVersion(u8),
    UnsupportedCompression(u16),
    FileEnvironmentError(FileEnvironmentError),
    IOError(String),
}

#[derive(Debug)]
enum FileEnvironment {
    MsDos = 0,
    Macintosh = 7,
    OSX = 19,
    WindowsNTFS = 10,
    FAT = 14,
    OS2 = 6,
    Unix = 3,
}

#[derive(Debug, PartialEq, Eq)]
enum FileEnvironmentError {
    InvalidFileEnvironment(u8),
}

#[derive(Debug, PartialEq, Eq)]
enum DeflateCompressionMode {
    Normal,
    Maximum,
    Fast,
    SuperFast,
}

#[derive(Debug, PartialEq, Eq)]
enum CompressionMethod {
    NoCompression,
    Deflate(DeflateCompressionMode),
}

impl Display for EndOfCentralDirectoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidZipFile(file_size) => write!(f, "Invalid ZIP file. The file size cannot be less than {} bytes. Given file size in bytes: {}", MIN_EOF_CENTRAL_DIR_SIZE, file_size),
            Self::InvalidSignature(sign) => write!(
                f,
                "Invalid end of central directory signature. Read signature: {:X}",
                sign
            ),
            Self::EmptyZipFile => write!(f, "A zip file must contain at least 1 file"),
            Self::IOError(error_msg) => write!(
                f,
                "An I/O error occured while parsing end of central directory. Message: {}",
                error_msg
            ),
        }
    }
}

impl Error for EndOfCentralDirectoryError {}

impl Display for ZipFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZipFileError::InvalidSignature(sign) => write!(
                f,
                "Invalid end of central directory signature. Read signature: {:X}",
                sign
            ),
            ZipFileError::UnsupportedZipVersion(zip_version) => {
                let major: i32 = (zip_version / 10) as i32;
                let minor: i32 = (zip_version % 10) as i32;

                write!(
                    f,
                    "Version specified in this ZIP file is not supported. Read Version: {}.{}",
                    major, minor
                )
            }
            ZipFileError::UnsupportedCompression(comp) => write!(
                f,
                "Unsupported compression method. Read compression method: {}",
                comp
            ),
            ZipFileError::FileEnvironmentError(err) => write!(f, "{}", err),
            Self::IOError(error_msg) => write!(
                f,
                "An I/O error occured while parsing central directory. Message: {}",
                error_msg
            ),
        }
    }
}

impl Error for ZipFileError {}

impl Display for FileEnvironment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileEnvironment::MsDos => write!(f, "MS-DOS"),
            FileEnvironment::Macintosh => write!(f, "Macintosh"),
            FileEnvironment::OSX => write!(f, "OS/X Darwin"),
            FileEnvironment::WindowsNTFS => write!(f, "Windows NTFS"),
            FileEnvironment::FAT => write!(f, "VFAT"),
            FileEnvironment::OS2 => write!(f, "OS/2"),
            FileEnvironment::Unix => write!(f, "UNIX"),
        }
    }
}

impl Display for FileEnvironmentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileEnvironmentError::InvalidFileEnvironment(byte) => {
                write!(f, "Invalid File environment read. Read value: {}", byte)
            }
        }
    }
}

impl Error for FileEnvironmentError {}

struct EndOfCentralDirectory {
    offset: u64,
    central_dir_size: u8,
    central_dir_start_offset: u32,
}

struct ZipFile {
    offset: u32,
    environment: FileEnvironment,
    is_encrypted: bool,
    compression_method: CompressionMethod,
    //The following flag will be used for determining whether CRC-32, Compressed size, uncompressed
    //size are written in the local file header if the below flag is set to false then the
    //information is kept in data descriptor follewed after local file header
    data_descriptor_used: bool,
    date_time: ZipDateTime,
}

impl EndOfCentralDirectory {
    pub fn from_readable<T>(
        readable: &mut T,
    ) -> Result<EndOfCentralDirectory, EndOfCentralDirectoryError>
    where
        T: Read + Seek,
    {
        let size = readable
            .seek(SeekFrom::End(0))
            .map_err(|err| EndOfCentralDirectoryError::IOError(err.to_string()))?;

        if size < MIN_EOF_CENTRAL_DIR_SIZE {
            return Err(EndOfCentralDirectoryError::InvalidZipFile(size));
        }

        let mut eof_central_dir_bytes = vec![0; MIN_EOF_CENTRAL_DIR_SIZE as usize];

        let eof_central_dir_offset = readable
            .seek(SeekFrom::End(-0x16))
            .map_err(|err| EndOfCentralDirectoryError::IOError(err.to_string()))?;

        readable
            .read_exact(&mut eof_central_dir_bytes)
            .map_err(|err| EndOfCentralDirectoryError::IOError(err.to_string()))?;

        let sign = LittleEndian::read_u32(&eof_central_dir_bytes[0..4]);

        if sign != EOF_CENTRAL_DIR_SIGN {
            return Err(EndOfCentralDirectoryError::InvalidSignature(sign));
        }

        let central_dir_size = eof_central_dir_bytes[10];

        if central_dir_size == 0 {
            return Err(EndOfCentralDirectoryError::EmptyZipFile);
        }

        let central_dir_start_offset = LittleEndian::read_u32(&eof_central_dir_bytes[16..20]);

        Ok(Self {
            offset: eof_central_dir_offset,
            central_dir_size,
            central_dir_start_offset,
        })
    }
}

impl ZipFile {
    pub fn from_readable<T>(readable: &mut T) -> Result<ZipFile, ZipFileError>
    where
        T: Read + Seek,
    {
        let mut central_dir_bytes = vec![0; MIN_CENTRAL_DIR_SIZE as usize];
        let offset = (readable
            .seek(SeekFrom::Current(0))
            .map_err(|err| ZipFileError::IOError(err.to_string()))?) as u32;

        readable
            .read_exact(&mut central_dir_bytes)
            .map_err(|err| ZipFileError::IOError(err.to_string()))?;

        let sign = LittleEndian::read_u32(&central_dir_bytes[0..4]);

        if sign != CENTRAL_DIR_SIGN {
            return Err(ZipFileError::InvalidSignature(sign));
        }

        let zip_version = central_dir_bytes[0x04];

        // We currently only support ZIP 2.0
        if zip_version != 0x14 {
            return Err(ZipFileError::UnsupportedZipVersion(zip_version));
        }

        let environment = FileEnvironment::from_byte(central_dir_bytes[0x05])
            .map_err(|err| ZipFileError::FileEnvironmentError(err))?;

        let compression_method_bytes = LittleEndian::read_u16(&central_dir_bytes[10..12]);
        let general_purpose_bit_flag = LittleEndian::read_u16(&central_dir_bytes[8..10]);

        let is_encrypted = (general_purpose_bit_flag & 0x0001) == 1;

        let compression_method = match compression_method_bytes {
            0x00 => CompressionMethod::NoCompression,
            0x08 => {
                // DEFLATE compression
                let deflate_mode = (general_purpose_bit_flag >> 1) & 0x0003;

                match deflate_mode {
                    0b00 => CompressionMethod::Deflate(DeflateCompressionMode::Normal),
                    0b01 => CompressionMethod::Deflate(DeflateCompressionMode::Maximum),
                    0b10 => CompressionMethod::Deflate(DeflateCompressionMode::Fast),
                    0b11 => CompressionMethod::Deflate(DeflateCompressionMode::SuperFast),
                    _ => CompressionMethod::Deflate(DeflateCompressionMode::Normal),
                }
            }
            _ => {
                return Err(ZipFileError::UnsupportedCompression(
                    compression_method_bytes,
                ))
            }
        };

        let data_descriptor_used = ((general_purpose_bit_flag >> 3) & 0x0001) == 1;
        let date = LittleEndian::read_u16(&central_dir_bytes[14..16]);
        let time = LittleEndian::read_u16(&central_dir_bytes[12..14]);

        let zip_date_time = ZipDateTime::from_bytes(date, time);

        Ok(Self {
            offset,
            environment,
            is_encrypted,
            compression_method,
            data_descriptor_used,
            date_time: zip_date_time,
        })
    }
}

impl FileEnvironment {
    pub fn from_byte(byte: u8) -> Result<FileEnvironment, FileEnvironmentError> {
        match byte {
            0 => Ok(FileEnvironment::MsDos),
            3 => Ok(FileEnvironment::Unix),
            6 => Ok(FileEnvironment::OS2),
            7 => Ok(FileEnvironment::Macintosh),
            10 => Ok(FileEnvironment::WindowsNTFS),
            14 => Ok(FileEnvironment::FAT),
            19 => Ok(FileEnvironment::OSX),
            _ => Err(FileEnvironmentError::InvalidFileEnvironment(byte)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_invalid_zip_file_error() {
        let mut cursor = Cursor::new(Vec::new());
        let eof_central_dir_result = EndOfCentralDirectory::from_readable(&mut cursor);

        assert!(eof_central_dir_result.is_err());
        assert_eq!(
            eof_central_dir_result.err().unwrap(),
            EndOfCentralDirectoryError::InvalidZipFile(0)
        );
    }

    #[test]
    fn test_eof_central_directory_signature_error() {
        let mut cursor = Cursor::new(vec![
            0x50, 0x4B, 0x05, 0x07, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x5A, 0x00,
            0x00, 0x00, 0x20, 0x01, 0x00, 0x00, 0x00, 0x00,
        ]);
        let eof_central_dir_result = EndOfCentralDirectory::from_readable(&mut cursor);

        assert!(eof_central_dir_result.is_err());
        assert_eq!(
            eof_central_dir_result.err().unwrap(),
            EndOfCentralDirectoryError::InvalidSignature(0x07054B50)
        );
    }

    #[test]
    fn test_eof_central_dir_empty_zip_file_error() {
        let mut cursor = Cursor::new(vec![
            0x50, 0x4B, 0x05, 0x06, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x5A, 0x00,
            0x00, 0x00, 0x20, 0x01, 0x00, 0x00, 0x00, 0x00,
        ]);
        let eof_central_dir_result = EndOfCentralDirectory::from_readable(&mut cursor);

        assert!(eof_central_dir_result.is_err());
        assert_eq!(
            eof_central_dir_result.err().unwrap(),
            EndOfCentralDirectoryError::EmptyZipFile
        );
    }

    #[test]
    fn test_successful_eof_central_dir() {
        let mut cursor = Cursor::new(vec![
            0x50, 0x4B, 0x05, 0x06, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x5A, 0x00,
            0x00, 0x00, 0x20, 0x01, 0x00, 0x00, 0x00, 0x00,
        ]);
        let eof_central_dir_result = EndOfCentralDirectory::from_readable(&mut cursor);

        assert!(eof_central_dir_result.is_ok());

        let eof_central_dir = eof_central_dir_result.unwrap();

        assert_eq!(eof_central_dir.offset, 0x00);
        assert_eq!(eof_central_dir.central_dir_size, 1);
        assert_eq!(eof_central_dir.central_dir_start_offset, 0x00000120);
    }

    #[test]
    fn test_zip_file_invalid_signature_error() {
        let mut cursor = Cursor::new(vec![
            0x50, 0x4B, 0x01, 0x03, 0x14, 0x03, 0x14, 0x00, 0x08, 0x00, 0x08, 0x00, 0x6F, 0xA7,
            0x39, 0x57, 0x7D, 0x99, 0xD7, 0xB2, 0xC6, 0x00, 0x00, 0x00, 0x30, 0x01, 0x00, 0x00,
            0x0C, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xA4, 0x81,
            0x00, 0x00, 0x00, 0x00, 0x63, 0x76, 0x5F, 0x64, 0x65, 0x62, 0x75, 0x67, 0x2E, 0x6C,
            0x6F, 0x67,
        ]);
        let zip_file_result = ZipFile::from_readable(&mut cursor);

        assert!(zip_file_result.is_err());
        assert_eq!(
            zip_file_result.err().unwrap(),
            ZipFileError::InvalidSignature(0x03014B50)
        );
    }

    #[test]
    fn test_zip_file_unsupported_zip_version() {
        let mut cursor = Cursor::new(vec![
            0x50, 0x4B, 0x01, 0x02, 0x15, 0x03, 0x14, 0x00, 0x08, 0x00, 0x08, 0x00, 0x6F, 0xA7,
            0x39, 0x57, 0x7D, 0x99, 0xD7, 0xB2, 0xC6, 0x00, 0x00, 0x00, 0x30, 0x01, 0x00, 0x00,
            0x0C, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xA4, 0x81,
            0x00, 0x00, 0x00, 0x00, 0x63, 0x76, 0x5F, 0x64, 0x65, 0x62, 0x75, 0x67, 0x2E, 0x6C,
            0x6F, 0x67,
        ]);
        let zip_file_result = ZipFile::from_readable(&mut cursor);

        assert!(zip_file_result.is_err());
        assert_eq!(
            zip_file_result.err().unwrap(),
            ZipFileError::UnsupportedZipVersion(0x15)
        );
    }

    #[test]
    fn test_unsupported_file_environment() {
        let mut cursor = Cursor::new(vec![
            0x50, 0x4B, 0x01, 0x02, 0x14, 0xFF, 0x14, 0x00, 0x08, 0x00, 0x08, 0x00, 0x6F, 0xA7,
            0x39, 0x57, 0x7D, 0x99, 0xD7, 0xB2, 0xC6, 0x00, 0x00, 0x00, 0x30, 0x01, 0x00, 0x00,
            0x0C, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xA4, 0x81,
            0x00, 0x00, 0x00, 0x00, 0x63, 0x76, 0x5F, 0x64, 0x65, 0x62, 0x75, 0x67, 0x2E, 0x6C,
            0x6F, 0x67,
        ]);
        let zip_file_result = ZipFile::from_readable(&mut cursor);

        assert!(zip_file_result.is_err());
        assert_eq!(
            zip_file_result.err().unwrap(),
            ZipFileError::FileEnvironmentError(FileEnvironmentError::InvalidFileEnvironment(0xFF))
        );
    }

    #[test]
    fn test_unsupported_compression_method() {
        let mut cursor = Cursor::new(vec![
            0x50, 0x4B, 0x01, 0x02, 0x14, 0x03, 0x14, 0x00, 0x08, 0x00, 0x10, 0x00, 0x6F, 0xA7,
            0x39, 0x57, 0x7D, 0x99, 0xD7, 0xB2, 0xC6, 0x00, 0x00, 0x00, 0x30, 0x01, 0x00, 0x00,
            0x0C, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xA4, 0x81,
            0x00, 0x00, 0x00, 0x00, 0x63, 0x76, 0x5F, 0x64, 0x65, 0x62, 0x75, 0x67, 0x2E, 0x6C,
            0x6F, 0x67,
        ]);
        let zip_file_result = ZipFile::from_readable(&mut cursor);

        assert!(zip_file_result.is_err());
        assert_eq!(
            zip_file_result.err().unwrap(),
            ZipFileError::UnsupportedCompression(0x10)
        );
    }

    #[test]
    fn test_file_encryption() {
        let mut cursor = Cursor::new(vec![
            0x50, 0x4B, 0x01, 0x02, 0x14, 0x03, 0x14, 0x00, 0x08, 0x00, 0x08, 0x00, 0x6F, 0xA7,
            0x39, 0x57, 0x7D, 0x99, 0xD7, 0xB2, 0xC6, 0x00, 0x00, 0x00, 0x30, 0x01, 0x00, 0x00,
            0x0C, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xA4, 0x81,
            0x00, 0x00, 0x00, 0x00, 0x63, 0x76, 0x5F, 0x64, 0x65, 0x62, 0x75, 0x67, 0x2E, 0x6C,
            0x6F, 0x67,
        ]);
        let zip_file_result = ZipFile::from_readable(&mut cursor);

        assert!(zip_file_result.is_ok());
        assert!(!zip_file_result.unwrap().is_encrypted);

        cursor = Cursor::new(vec![
            0x50, 0x4B, 0x01, 0x02, 0x14, 0x03, 0x14, 0x00, 0x09, 0x00, 0x08, 0x00, 0x6F, 0xA7,
            0x39, 0x57, 0x7D, 0x99, 0xD7, 0xB2, 0xC6, 0x00, 0x00, 0x00, 0x30, 0x01, 0x00, 0x00,
            0x0C, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xA4, 0x81,
            0x00, 0x00, 0x00, 0x00, 0x63, 0x76, 0x5F, 0x64, 0x65, 0x62, 0x75, 0x67, 0x2E, 0x6C,
            0x6F, 0x67,
        ]);
        let zip_file_result = ZipFile::from_readable(&mut cursor);

        assert!(zip_file_result.is_ok());
        assert!(zip_file_result.unwrap().is_encrypted)
    }

    #[test]
    fn test_data_descriptor_used() {
        let mut cursor = Cursor::new(vec![
            0x50, 0x4B, 0x01, 0x02, 0x14, 0x03, 0x14, 0x00, 0x08, 0x00, 0x08, 0x00, 0x6F, 0xA7,
            0x39, 0x57, 0x7D, 0x99, 0xD7, 0xB2, 0xC6, 0x00, 0x00, 0x00, 0x30, 0x01, 0x00, 0x00,
            0x0C, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xA4, 0x81,
            0x00, 0x00, 0x00, 0x00, 0x63, 0x76, 0x5F, 0x64, 0x65, 0x62, 0x75, 0x67, 0x2E, 0x6C,
            0x6F, 0x67,
        ]);
        let zip_file_result = ZipFile::from_readable(&mut cursor);

        assert!(zip_file_result.is_ok());
        assert!(zip_file_result.unwrap().data_descriptor_used);

        cursor = Cursor::new(vec![
            0x50, 0x4B, 0x01, 0x02, 0x14, 0x03, 0x14, 0x00, 0x00, 0x00, 0x08, 0x00, 0x6F, 0xA7,
            0x39, 0x57, 0x7D, 0x99, 0xD7, 0xB2, 0xC6, 0x00, 0x00, 0x00, 0x30, 0x01, 0x00, 0x00,
            0x0C, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xA4, 0x81,
            0x00, 0x00, 0x00, 0x00, 0x63, 0x76, 0x5F, 0x64, 0x65, 0x62, 0x75, 0x67, 0x2E, 0x6C,
            0x6F, 0x67,
        ]);
        let zip_file_result = ZipFile::from_readable(&mut cursor);

        assert!(zip_file_result.is_ok());
        assert!(!zip_file_result.unwrap().data_descriptor_used);
    }
}
