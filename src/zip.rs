use byteorder::{ByteOrder, LittleEndian};
use std::error::Error;
use std::fmt::Display;
use std::io::{Read, Seek, SeekFrom};

const MIN_EOF_CENTRAL_DIR_SIZE: u64 = 0x16;
const EOF_CENTRAL_DIR_SIGN: u32 = 0x06054b50;

#[derive(Debug, PartialEq, Eq)]
enum EndOfCentralDirectoryError {
    InvalidZipFile(u64),
    InvalidSignature(u32),
    EmptyZipFile,
    IOError(String),
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

struct EndOfCentralDirectory {
    central_dir_size: u8,
    central_dir_start_offset: u32,
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

        readable
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
            central_dir_size,
            central_dir_start_offset,
        })
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

        assert_eq!(eof_central_dir.central_dir_size, 1);
        assert_eq!(eof_central_dir.central_dir_start_offset, 0x00000120);
    }
}
