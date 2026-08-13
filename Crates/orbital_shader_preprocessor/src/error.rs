#[derive(Debug)]
pub enum ShaderPreprocessorError {
    UnknownDirective { directive: String },
    NonUTF8FileName { file_name: std::ffi::OsString },
    Fs(orbital_file_manager::FsError),
}
