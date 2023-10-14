use std::env::args;
use std::fs::File;
use std::path::PathBuf;

const ZIPPY_VERSION: &str = "0.1.0";
const ZIP_FILE_PATH_MISSING_ERROR_RETURN_CODE: i32 = -1;

fn main() {
    let zip_file_path = args()
        .nth(1)
        .map(|path_str| PathBuf::from(path_str))
        .unwrap_or_else(|| {
            print_help();
            std::process::exit(ZIP_FILE_PATH_MISSING_ERROR_RETURN_CODE);
        });

    let mut zip_file = File::open(zip_file_path);
}

fn print_help() {
    println!("zippy version: {}", ZIPPY_VERSION);
    println!("USAGE: zippy <ZIP_FILE_PATH>");
}
