use std::env::args;
use std::fs::File;
use std::path::PathBuf;

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
        Ok(file) => file,
        Err(err) => {
            eprintln!(
                "An error occurred while trying to open the input file.\n\"{}\"",
                err.to_string()
            );
            std::process::exit(UNABLE_TO_OPEN_FILE_ERROR_RETURN_CODE);
        }
    };

    let zip = match Zip::from_file(zip_file) {
        Ok(zip) => zip,
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(ZIP_FILE_PARSING_ERROR_RETURN_CODE);
        }
    };

    println!(
        "File Count: {}, Directory Count: {}\n",
        zip.file_count(),
        zip.dir_count()
    );
    println!("File Name\tDate Time\tCompression Method\tIs Directory");
    zip.zip_files().iter().for_each(|zip_file| {
        println!(
            "{}\t{}\t{}\t{}",
            zip_file.file_name(),
            zip_file.date_time(),
            zip_file.compression_method(),
            zip_file.is_dir()
        )
    });
}

fn print_help() {
    println!("zippy version: {}", ZIPPY_VERSION);
    println!("USAGE: zippy <ZIP_FILE_PATH>");
}
