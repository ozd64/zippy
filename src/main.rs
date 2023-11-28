use zippy::clap::{ArchiveCommand, Cli};
use zippy::commands::{self, ExtractOptions};
use zippy::util::get_file_path;

use clap::Parser;

const INVALID_PATH_ERROR_RETURN_CODE: i32 = -10;

fn main() {
    let cli = Cli::parse();

    match cli.archive_command {
        ArchiveCommand::Zip { zip_command } => {
            //EXTRACT COMMAND
            if let Some(path) = zip_command.extract {
                let path = match get_file_path(path) {
                    Ok(path) => path,
                    Err(err) => {
                        eprintln!("{}", err);
                        std::process::exit(INVALID_PATH_ERROR_RETURN_CODE);
                    }
                };

                let extract_options =
                    ExtractOptions::new(path, zip_command.verbose, zip_command.destination);
                match commands::extract_files(extract_options) {
                    Ok(_) => (),
                    Err(err) => eprintln!("{}", err),
                }
            }

            //LIST COMMAND
            if let Some(path) = zip_command.list {
                let path = match get_file_path(path) {
                    Ok(path) => path,
                    Err(err) => {
                        eprintln!("{}", err);
                        std::process::exit(INVALID_PATH_ERROR_RETURN_CODE);
                    }
                };
                commands::list_files(path);
            }
        }
    }
}
