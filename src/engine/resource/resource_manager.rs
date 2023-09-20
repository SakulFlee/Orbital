use std::{
    fs::read,
    path::{Path, PathBuf},
};

use crate::engine::{EngineError, EngineResult};

pub struct ResourceManager;

impl ResourceManager {
    pub const RESOURCE_FOLDER_NAME: &'static str = "res";

    pub fn get_resource_folder_path() -> PathBuf {
        if cfg!(debug_assertions) {
            Path::new(env!("OUT_DIR")).join(Self::RESOURCE_FOLDER_NAME)
        } else {
            Path::new(".").join(Self::RESOURCE_FOLDER_NAME)
        }
    }

    pub fn get_resource_path<P>(file_name: P) -> EngineResult<PathBuf>
    where
        P: AsRef<Path>,
    {
        let path = Self::get_resource_folder_path().join(file_name);

        if !path.exists() {
            Err(EngineError::ResourceMissing)
        } else {
            Ok(path)
        }
    }

    pub fn read_resource_binary<P>(file_name: P) -> EngineResult<Vec<u8>>
    where
        P: AsRef<Path>,
    {
        let path = Self::get_resource_path(file_name)?;

        Ok(read(&path).map_err(|e| EngineError::IOError(e))?)
    }
}
