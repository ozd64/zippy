use byteorder::{ByteOrder, LittleEndian};
use std::error::Error;
use std::fmt::Display;
use std::io::{Read, Seek, SeekFrom};

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
    environment: FileEnvironment,
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
        T: Read,
    {
        let mut central_dir_bytes = vec![0; MIN_CENTRAL_DIR_SIZE as usize];

        readable
            .read_exact(&mut central_dir_bytes)
            .map_err(|err| ZipFileError::IOError(err.to_string()))?;

        let sign = LittleEndian::read_u32(&central_dir_bytes[0..4]);

        if sign != CENTRAL_DIR_SIGN {
            return Err(ZipFileError::InvalidSignature(sign));
        }

        let zip_version = central_dir_bytes[0x04];

        if zip_version != 0x14 {
            return Err(ZipFileError::UnsupportedZipVersion(zip_version));
        }

        let environment = FileEnvironment::from_byte(central_dir_bytes[0x05])
            .map_err(|err| ZipFileError::FileEnvironmentError(err))?;

        Ok(Self { environment })
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
}
