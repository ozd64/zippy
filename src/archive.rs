use std::error::Error;
use std::fmt::Display;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use byteorder::{ByteOrder, LittleEndian};
use crc::{Crc, CRC_32_ISO_HDLC};
use flate2::read::DeflateDecoder;

use crate::headers::{CompressionMethod, ZipFile};

const MIN_LOCAL_FILE_HEADER_SIZE: usize = 30;
const FILE_READ_WRITE_BUFFER_SIZE: usize = 4096;

pub trait ReadableArchive: Read + Seek {}

impl<T: Read + Seek> ReadableArchive for BufReader<T> {}

pub type RefReadableArchive = Box<dyn ReadableArchive>;
type Crc32 = u32;

#[derive(Debug, PartialEq, Eq)]
pub enum ExtractError {
    IOError(String),
    InvalidZipFileParent(PathBuf),
    UnableToCreateExtractedFile(String, String),
    DeflateDecodingError(String),
    InvalidExtractedFile(u32, u32),
}

impl Display for ExtractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtractError::IOError(err_msg) => {
                write!(f, "An I/O error occurred while extracting file {}", err_msg)
            }
            ExtractError::InvalidZipFileParent(parent_path) => write!(f, "Invalid parent path for the zip file. Invalid parent path: {}", parent_path.as_path().display().to_string()),
            ExtractError::UnableToCreateExtractedFile(file_name, error_msg) => write!(f, "Unable to create the extracted file \"{}\".\nError: {}", file_name, error_msg),
            ExtractError::DeflateDecodingError(error_msg) => write!(f, "Unable to decode the deflated stream. {}", error_msg),
            ExtractError::InvalidExtractedFile(crc32, extracted_file_crc32) => write!(f, "Extracted file corruption. CRC-32 checksums are not matching. File CRC-32: 0x{:X}, Extracted file CRC-32: 0x{:X}", crc32, extracted_file_crc32)
        }
    }
}

impl Error for ExtractError {}

pub trait Extract {
    //TODO: Consider making ExtractError as trait type
    fn extract<P>(
        &self,
        extract_path: &P,
        extract_file: &mut RefReadableArchive,
    ) -> Result<(), ExtractError>
    where
        P: AsRef<Path>;
}

pub trait Archive {
    fn extract_items<P>(&mut self, extract_path: P) -> Result<usize, ExtractError>
    where
        P: AsRef<Path>;
}

impl Extract for ZipFile {
    fn extract<P>(
        &self,
        extract_path: &P,
        extract_file: &mut RefReadableArchive,
    ) -> Result<(), ExtractError>
    where
        P: AsRef<Path>,
    {
        let mut extracted_file_path = PathBuf::new();

        extracted_file_path.push(extract_path);
        extracted_file_path.push(self.file_name());

        //If the file is just a directory then just create the directory.
        if self.is_dir() {
            return std::fs::create_dir_all(extracted_file_path)
                .map_err(|err| ExtractError::IOError(err.to_string()));
        }

        // If the parent folder for the file is not created then create the parent folder before
        // creating the file.
        if let Some(parent_path) = extracted_file_path.parent() {
            if !parent_path.exists() {
                std::fs::create_dir_all(parent_path)
                    .map_err(|err| ExtractError::IOError(err.to_string()))?;
            }
        } else {
            return Err(ExtractError::InvalidZipFileParent(extracted_file_path));
        }

        let mut file = File::create(extracted_file_path.clone()).map_err(|err| {
            ExtractError::UnableToCreateExtractedFile(self.file_name().clone(), err.to_string())
        })?;
        let mut local_file_header_bytes = vec![0u8; MIN_LOCAL_FILE_HEADER_SIZE];

        extract_file
            .seek(std::io::SeekFrom::Start(self.offset() as u64))
            .map_err(|err| ExtractError::IOError(err.to_string()))?;
        extract_file
            .read_exact(&mut local_file_header_bytes)
            .map_err(|err| ExtractError::IOError(err.to_string()))?;

        let file_name_len = self.file_name().len();
        let extra_field_len = LittleEndian::read_u16(&local_file_header_bytes[28..]) as usize;
        let file_bytes_start_offset = file_name_len + extra_field_len;

        extract_file
            .seek(SeekFrom::Current(file_bytes_start_offset as i64))
            .map_err(|err| ExtractError::IOError(err.to_string()))?;

        let mut file_data_reader = if self.compression_method() == &CompressionMethod::NoCompression
        {
            extract_file.take(self.uncompressed_size().get() as u64)
        } else {
            extract_file.take(self.compressed_size().get() as u64)
        };

        //Decode the file
        let created_file_crc32 = match self.compression_method() {
            CompressionMethod::NoCompression => {
                //If no compression is set then just copy the file bytes into destination and
                //calculate CRC-32
                std::io::copy(&mut file_data_reader, &mut file)
                    .map_err(|err| ExtractError::IOError(err.to_string()))?;
                calculate_crc32(extracted_file_path)
                    .map_err(|err| ExtractError::IOError(err.to_string()))?
            }
            CompressionMethod::Deflate(_) => {
                decode_and_write_deflated_compressed_data(&mut file_data_reader, &mut file)?
            }
        };

        //If we extract a file then make sure that CRC-32 checksums are matching
        if !self.is_dir() {
            let crc32 = self.crc32().get();

            // If checksums are not matching then quit extracting the file.
            if crc32 != created_file_crc32 {
                return Err(ExtractError::InvalidExtractedFile(
                    crc32,
                    created_file_crc32,
                ));
            }
        }

        Ok(())
    }
}

fn decode_and_write_deflated_compressed_data<R, W>(
    reader: &mut R,
    writer: &mut W,
) -> Result<Crc32, ExtractError>
where
    R: Read,
    W: Write,
{
    let mut deflate_decoder = DeflateDecoder::new(reader);
    let mut buf = vec![0u8; FILE_READ_WRITE_BUFFER_SIZE];
    let crc = Crc::<u32>::new(&CRC_32_ISO_HDLC);
    let mut digest = crc.digest();

    loop {
        let read_bytes = deflate_decoder
            .read(&mut buf)
            .map_err(|err| ExtractError::DeflateDecodingError(err.to_string()))?;

        if read_bytes == 0 {
            break;
        }
        let read_bytes_buf = &buf[..read_bytes];

        writer
            .write_all(read_bytes_buf)
            .map_err(|err| ExtractError::IOError(err.to_string()))?;
        digest.update(read_bytes_buf);
    }

    Ok(digest.finalize())
}

fn calculate_crc32<P>(file_path: P) -> Result<Crc32, std::io::Error>
where
    P: AsRef<Path>,
{
    let mut extracted_file = File::open(file_path)?;
    let mut buf = vec![0u8; FILE_READ_WRITE_BUFFER_SIZE];
    let crc = Crc::<u32>::new(&CRC_32_ISO_HDLC);
    let mut digest = crc.digest();

    loop {
        let read_bytes = extracted_file.read(&mut buf)?;

        if read_bytes == 0 {
            break;
        }

        digest.update(&buf[..read_bytes]);
    }

    Ok(digest.finalize())
}
