use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub archive_command: ArchiveCommand,
}

#[derive(Subcommand)]
pub enum ArchiveCommand {
    #[command(arg_required_else_help = true)]
    Zip {
        #[clap(flatten)]
        zip_command: ZipCommand,
    },
}

#[derive(Debug, clap::Args)]
#[group(required = true)]
pub struct ZipCommand {
    #[arg(
        short = 'x',
        long,
        help = "Extracts the given ZIP file.",
        value_name = "ZIP_FILE_PATH"
    )]
    pub extract: Option<PathBuf>,

    #[arg(
        short,
        long,
        help = "Extract files in verbose mode. This flag enables which files are being extracted"
    )]
    pub verbose: bool,

    #[arg(
        short,
        long,
        help = "Choose the destination path of the extracted files",
        value_name = "DESTINATION_FOLDER"
    )]
    pub destination: Option<PathBuf>,

    #[arg(
        short,
        long,
        help = "List all files listed in a given zip file",
        value_name = "ZIP_FILE_PATH"
    )]
    pub list: Option<PathBuf>,
}
