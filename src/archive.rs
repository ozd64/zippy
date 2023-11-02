use std::error::Error;
use std::fmt::Display;
use std::fs::File;
use std::path::Path;

use crate::headers::ZipFile;

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
    fn extract<P>(&self, extract_path: &P, extract_file: &File) -> Result<(), ExtractError> where P: AsRef<Path>;
}

pub trait Archive {
    fn extract_items<P>(&self, extract_path: P) -> Result<usize, ExtractError> where P: AsRef<Path>;
}

impl Extract for ZipFile {
    fn extract<P>(&self, extract_path: &P, extract_file: &File) -> Result<(), ExtractError> where P: AsRef<Path> {
        todo!()
    }
}
