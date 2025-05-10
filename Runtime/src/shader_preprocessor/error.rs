use std::{ffi::OsString, io};

#[derive(Debug)]
pub enum ShaderPreprocessorError {
    UnknownDirective { directive: String },
    NonUTF8FileName { file_name: OsString },
    IOError(io::Error),
    PatternError(glob::PatternError),
}
