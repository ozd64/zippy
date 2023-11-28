use std::error::Error;
use std::fmt::Display;
use std::io::SeekFrom;
use std::path::PathBuf;

use crate::archive::{Archive, Extract, ExtractError, ReadableArchive};
use crate::commands::ExtractOptions;
use crate::headers::{
    EncryptionMethod, EndOfCentralDirectory, EndOfCentralDirectoryError, ZipFile, ZipFileError,
};

#[derive(Debug)]
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

pub struct Zip<R: ReadableArchive> {
    readable: R,
    zip_file_count: usize,
    file_count: usize,
    dir_count: usize,
    files_encrypted: bool,
    zip_files: Vec<ZipFile>,
}

impl<R: ReadableArchive> Zip<R> {
    pub fn from_readable(mut readable: R) -> Result<Self, ZipError> {
        let end_of_central_dir = EndOfCentralDirectory::from_readable(&mut readable)
            .map_err(|err| ZipError::EndOfCentralDirectoryError(err))?;

        readable
            .seek(SeekFrom::Start(
                end_of_central_dir.central_dir_start_offset() as u64,
            ))
            .map_err(|err| ZipError::IOError(err.to_string()))?;

        let mut zip_files: Vec<ZipFile> =
            Vec::with_capacity(end_of_central_dir.central_dir_size() as usize);

        for _ in 0..end_of_central_dir.central_dir_size() {
            match ZipFile::from_readable(&mut readable) {
                Ok(zip_file) => zip_files.push(zip_file),
                Err(err) => return Err(ZipError::ZipFileError(err)),
            }
        }

        let dir_count = zip_files
            .iter()
            .filter(|zip_file| zip_file.is_dir())
            .count();

        let file_count = ((end_of_central_dir.central_dir_size()) as usize) - dir_count;

        // Update CRC-32, Uncompressed size as well as compressed size in case ZIP file is
        // configured with Data descriptor
        let zip_file_offsets: Vec<u32> =
            zip_files.iter().map(|zip_file| zip_file.offset()).collect();

        zip_files = zip_files
            .into_iter()
            .enumerate()
            .map(|(index, zip_file)| {
                if zip_file.data_descriptor_used() {
                    if index == (zip_file_offsets.len() - 1) {
                        zip_file.update_with_data_descriptor(
                            &mut readable,
                            end_of_central_dir.central_dir_start_offset(),
                        );
                    } else {
                        zip_file.update_with_data_descriptor(
                            &mut readable,
                            zip_file_offsets[index + 1],
                        );
                    }
                }

                zip_file
            })
            .collect();

        let files_encrypted = zip_files
            .iter()
            .any(|zip_file| zip_file.encryption_method() != &EncryptionMethod::NoEncryption);

        Ok(Self {
            readable,
            zip_file_count: end_of_central_dir.central_dir_size() as usize,
            zip_files,
            dir_count,
            files_encrypted,
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

    pub fn files_encrypted(&self) -> bool {
        self.files_encrypted
    }
}

impl<R: ReadableArchive> Archive for Zip<R> {
    fn extract_items(
        &mut self,
        extract_options: ExtractOptions,
        password: Option<String>,
    ) -> Result<usize, ExtractError> {
        let parent = extract_options
            .path
            .parent()
            .map(|parent_path| PathBuf::from(parent_path))
            .unwrap();

        self.zip_files
            .iter()
            .map(|zip_item| {
                zip_item.extract(
                    &parent,
                    &mut self.readable,
                    &password,
                    extract_options.verbose,
                )
            })
            .try_fold(0, |count, zip_extract_result| {
                zip_extract_result.map(|_| count + 1)
            })
    }
}
