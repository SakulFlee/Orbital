use std::{ffi::OsString, io};

#[derive(Debug)]
pub enum Error {
    UnknownDirective { directive: String },
    NonUTF8FileName { file_name: OsString },
    IOError(io::Error),
    PatternError(glob::PatternError),
}
