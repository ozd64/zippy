use std::env::args;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use zippy::pretty_printer::pretty_print_zip_files;
use zippy::zip::Zip;

const ZIPPY_VERSION: &str = "0.1.0";
const ZIP_FILE_PATH_MISSING_ERROR_RETURN_CODE: i32 = -1;
const ZIP_FILE_PARSING_ERROR_RETURN_CODE: i32 = -2;
const UNABLE_TO_OPEN_FILE_ERROR_RETURN_CODE: i32 = -3;


fn main() {
    let zip_file_path = args()
        .nth(1)
        .map(|path_str| PathBuf::from(path_str))
        .unwrap_or_else(|| {
            print_help();
            std::process::exit(ZIP_FILE_PATH_MISSING_ERROR_RETURN_CODE);
        });

    let zip_file = match File::open(zip_file_path) {
        Ok(file) => Box::new(BufReader::new(file)),
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

fn print_help() {
    println!("zippy version: {}", ZIPPY_VERSION);
    println!("USAGE: zippy <ZIP_FILE_PATH>");
}
