use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use crate::archive::{Archive, ExtractError};
use crate::pretty_printer::pretty_print_zip_files;
use crate::zip::Zip;

const UNABLE_TO_OPEN_FILE_ERROR_RETURN_CODE: i32 = -3;
const ZIP_FILE_PARSING_ERROR_RETURN_CODE: i32 = -2;

pub struct ExtractOptions {
    pub path: PathBuf,
    pub verbose: bool,
    pub destination_path: Option<PathBuf>,
}

impl ExtractOptions {
    pub fn new(path: PathBuf, verbose: bool, destination_path: Option<PathBuf>) -> Self {
        Self {
            path,
            verbose,
            destination_path,
        }
    }
}

pub fn extract_files(extract_options: ExtractOptions) -> Result<(), ExtractError> {
    let zip_file = match File::open(extract_options.path.clone()) {
        Ok(file) => BufReader::new(file),
        Err(err) => {
            eprintln!(
                "An error occurred while trying to open the input file.\n\"{}\"",
                err.to_string()
            );
            std::process::exit(UNABLE_TO_OPEN_FILE_ERROR_RETURN_CODE);
        }
    };

    let mut zip = match Zip::from_readable(zip_file) {
        Ok(zip) => zip,
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(ZIP_FILE_PARSING_ERROR_RETURN_CODE);
        }
    };

    let password = if zip.files_encrypted() {
        rpassword::prompt_password("Password: ").ok()
    } else {
        None
    };

    zip.extract_items(extract_options, password).map(|_| ())
}

pub fn list_files<P>(zip_file_path: P)
where
    P: AsRef<Path>,
{
    let zip_file = match File::open(zip_file_path) {
        Ok(file) => BufReader::new(file),
        Err(err) => {
            eprintln!(
                "An error occurred while trying to open the input file.\n\"{}\"",
                err.to_string()
            );
            std::process::exit(UNABLE_TO_OPEN_FILE_ERROR_RETURN_CODE);
        }
    };

    let zip = match Zip::from_readable(zip_file) {
        Ok(zip) => zip,
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(ZIP_FILE_PARSING_ERROR_RETURN_CODE);
        }
    };

    pretty_print_zip_files(&zip);
}
