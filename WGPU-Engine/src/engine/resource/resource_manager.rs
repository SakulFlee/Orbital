use std::{
    fs::read,
    path::{Path, PathBuf},
};

use easy_gltf::Scene;

use crate::engine::{EngineError, EngineResult};

pub struct ResourceManager;

impl ResourceManager {
    pub const RESOURCE_FOLDER_NAME: &'static str = "res";

    pub fn resource_folder_path() -> PathBuf {
        if cfg!(debug_assertions) {
            Path::new(env!("OUT_DIR")).join(Self::RESOURCE_FOLDER_NAME)
        } else {
            Path::new(".").join(Self::RESOURCE_FOLDER_NAME)
        }
    }

    pub fn resource_path<P>(file_name: P) -> EngineResult<PathBuf>
    where
        P: AsRef<Path>,
    {
        let path = Self::resource_folder_path().join(file_name);

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
        let path = Self::resource_path(file_name)?;

        read(path).map_err(EngineError::IOError)
    }

    pub fn read_resource_gltf<P>(file_name: P) -> EngineResult<Vec<Scene>>
    where
        P: AsRef<Path>,
    {
        let path = Self::resource_path(file_name)?;

        easy_gltf::load(path).map_err(EngineError::GltfBadMode)
    }
}
