use std::error::Error;
use std::fmt::Display;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub enum PathError {
    EmptyPath,
    ParentPathGiven,
    CurrentPathGiven,
    EnvironmentError(String),
}

impl Display for PathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathError::ParentPathGiven => write!(
                f,
                "Parent path of the current dir cannot be a archive file path."
            ),
            PathError::CurrentPathGiven => write!(f, "Current path cannot be a archive file path"),
            PathError::EnvironmentError(error_msg) => write!(
                f,
                "An error occurred while finding the parent path of the current dir.\n{}",
                error_msg
            ),
            PathError::EmptyPath => write!(f, "Archive file path cannot be empty"),
        }
    }
}

impl Error for PathError {}

pub fn get_file_path(path: PathBuf) -> Result<PathBuf, PathError> {
    if let Some(file_name) = path.file_name() {
        let lossy_file_name = file_name.to_string_lossy();

        if lossy_file_name.is_empty() {
            return Err(PathError::EmptyPath);
        }

        if lossy_file_name == "." {
            return Err(PathError::CurrentPathGiven);
        } else if lossy_file_name == ".." {
            return Err(PathError::ParentPathGiven);
        }
    }

    if path.is_relative() {
        let current_dir =
            std::env::current_dir().map_err(|err| PathError::EnvironmentError(err.to_string()))?;
        let mut absolute_path = PathBuf::from(current_dir);

        absolute_path.push(path);

        Ok(absolute_path)
    } else {
        Ok(path)
    }
}
