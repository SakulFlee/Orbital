use std::{error::Error, fmt, io};

/// Errors reported by the [`FileManager`](crate::FileManager) backends.
#[derive(Debug)]
pub enum FsError {
    /// The file manager was never initialized (Android only, before
    /// `android_main` runs).
    NotInitialized,
    /// The requested path does not exist or could not be opened.
    NotFound(String),
    /// The path exists but its contents are not valid UTF-8.
    Utf8(String),
    /// An underlying I/O error.
    Io(String, io::Error),
}

impl FsError {
    /// Maps an [`io::Error`] to [`FsError`], capturing the path for context.
    pub(crate) fn from_io(path: &str, error: io::Error) -> Self {
        if error.kind() == io::ErrorKind::NotFound {
            FsError::NotFound(path.to_string())
        } else {
            FsError::Io(path.to_string(), error)
        }
    }
}

impl fmt::Display for FsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FsError::NotInitialized => {
                write!(f, "FileManager was not initialized on this platform")
            }
            FsError::NotFound(path) => write!(f, "Path not found: '{path}'"),
            FsError::Utf8(path) => write!(f, "Contents of '{path}' are not valid UTF-8"),
            FsError::Io(path, error) => write!(f, "I/O error on '{path}': {error}"),
        }
    }
}

impl Error for FsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            FsError::Io(_, error) => Some(error),
            _ => None,
        }
    }
}
