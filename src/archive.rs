use std::error::Error;
use std::fmt::Display;
use std::io::{Read, Seek, BufReader};
use std::path::Path;

use crate::headers::ZipFile;

pub trait ReadableArchive: Read + Seek {}

impl<T: Read + Seek> ReadableArchive for BufReader<T> {}

#[derive(Debug, PartialEq, Eq)]
pub enum ExtractError {
    IOError(String),
}

impl Display for ExtractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtractError::IOError(err_msg) => {
                write!(f, "An I/O error occurred while extracting file {}", err_msg)
            }
        }
    }
}

impl Error for ExtractError {}

pub trait Extract {
    //TODO: Consider making ExtractError as trait type
    fn extract<P>(&self, extract_path: &P, extract_file: &mut Box<dyn ReadableArchive>) -> Result<(), ExtractError>
    where
        P: AsRef<Path>;
}

pub trait Archive {
    fn extract_items<P>(&mut self, extract_path: P) -> Result<usize, ExtractError>
    where
        P: AsRef<Path>;
}

impl Extract for ZipFile {
    fn extract<P>(&self, extract_path: &P, extract_file: &mut Box<dyn ReadableArchive>) -> Result<(), ExtractError>
    where P: AsRef<Path>
    {
        todo!()
    }
}
