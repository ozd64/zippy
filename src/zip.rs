use std::error::Error;
use std::fmt::Display;
use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::path::Path;

use crate::archive::{Archive, Extract, ExtractError};
use crate::headers::{EndOfCentralDirectory, EndOfCentralDirectoryError, ZipFile, ZipFileError};

#[derive(Debug, PartialEq, Eq)]
pub enum ZipError {
    EndOfCentralDirectoryError(EndOfCentralDirectoryError),
    ZipFileError(ZipFileError),
    IOError(String),
}

impl Display for ZipError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EndOfCentralDirectoryError(err) => {
                write!(f, "An error occurred while reading ZIP file.\n{}", err)
            }
            Self::ZipFileError(err) => {
                write!(f, "An error occurred while reading ZIP file.\n{}", err)
            }
            Self::IOError(error_msg) => write!(
                f,
                "An I/O error occured while parsing ZIP file. Message: {}",
                error_msg
            ),
        }
    }
}

impl Error for ZipError {}

pub struct Zip {
    file: File,
    zip_file_count: usize,
    file_count: usize,
    dir_count: usize,
    zip_files: Vec<ZipFile>,
}

impl Zip {
    pub fn from_file(mut file: File) -> Result<Self, ZipError> {
        let end_of_central_dir = EndOfCentralDirectory::from_readable(&mut file)
            .map_err(|err| ZipError::EndOfCentralDirectoryError(err))?;

        file.seek(SeekFrom::Start(
            end_of_central_dir.central_dir_start_offset() as u64,
        ))
        .map_err(|err| ZipError::IOError(err.to_string()))?;

        let mut zip_files: Vec<ZipFile> =
            Vec::with_capacity(end_of_central_dir.central_dir_size() as usize);

        for _ in 0..end_of_central_dir.central_dir_size() {
            match ZipFile::from_readable(&mut file) {
                Ok(zip_file) => zip_files.push(zip_file),
                Err(err) => return Err(ZipError::ZipFileError(err)),
            }
        }

        let dir_count = zip_files
            .iter()
            .filter(|zip_file| zip_file.is_dir())
            .count();

        let file_count = ((end_of_central_dir.central_dir_size()) as usize) - dir_count;

        Ok(Self {
            file,
            zip_file_count: end_of_central_dir.central_dir_size() as usize,
            zip_files,
            dir_count,
            file_count,
        })
    }

    pub fn zip_file_couunt(&self) -> usize {
        self.zip_file_count
    }

    pub fn zip_files(&self) -> &Vec<ZipFile> {
        &self.zip_files
    }

    pub fn dir_count(&self) -> usize {
        self.dir_count
    }

    pub fn file_count(&self) -> usize {
        self.file_count
    }
}

impl Archive for Zip {
    fn extract_items<P>(&self, extract_path: P) -> Result<usize, ExtractError>
    where
        P: AsRef<Path>,
    {
        self.zip_files
            .iter()
            .map(|zip_item| zip_item.extract(&extract_path, &self.file))
            .try_fold(0, |count, zip_extract_result| {
                zip_extract_result.map(|_| count + 1)
            })
    }
}
