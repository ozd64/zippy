use zippy::clap::{ArchiveCommand, Cli};
use zippy::commands::{self, ExtractOptions};

use clap::Parser;

fn main() {
    let cli = Cli::parse();

    match cli.archive_command {
        ArchiveCommand::Zip { zip_command } => {
            //EXTRACT COMMAND
            if let Some(path) = zip_command.extract {
                let extract_options =
                    ExtractOptions::new(path, zip_command.verbose, zip_command.destination);
                match commands::extract_files(extract_options) {
                    Ok(_) => (),
                    Err(err) => eprintln!("{}", err),
                }
            }

            //LIST COMMAND
            if let Some(path) = zip_command.list {
                commands::list_files(path);
            }
        }
    }
}
