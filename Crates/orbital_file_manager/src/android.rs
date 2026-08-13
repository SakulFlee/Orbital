//! Android backends: assets via `AAssetManager`, storage in the app's
//! internal data directory.

use std::{
    ffi::CString,
    io::{Read, Write},
    path::PathBuf,
};

use ndk::asset::{Asset, AssetManager};

use crate::dir::DirStorage;
use crate::{AssetSource, FsError, Storage};

/// Android asset source backed by the APK's `AAssetManager`.
pub struct AndroidAssetSource {
    asset_manager: AssetManager,
}

impl AndroidAssetSource {
    pub fn new(asset_manager: AssetManager) -> Self {
        Self { asset_manager }
    }

    fn open(&self, path: &str) -> Result<Asset, FsError> {
        let cstr = CString::new(path)
            .map_err(|_| FsError::Utf8(path.to_string()))?;
        self.asset_manager
            .open(&cstr)
            .ok_or_else(|| FsError::NotFound(path.to_string()))
    }

    fn open_dir(&self, dir: &str) -> Option<ndk::asset::AssetDir> {
        let cstr = CString::new(dir).ok()?;
        self.asset_manager.open_dir(&cstr)
    }

    fn collect(&self, dir: &str, out: &mut Vec<String>) {
        let Some(mut asset_dir) = self.open_dir(dir) else {
            return;
        };
        while let Some(entry) = asset_dir.next() {
            let name = entry.to_str().unwrap_or_default();
            let sub = if dir.is_empty() {
                name.to_string()
            } else {
                format!("{dir}/{name}")
            };
            // Directories cannot be opened as assets; recurse into them.
            if self.open(&sub).is_ok() {
                out.push(sub);
            } else {
                self.collect(&sub, out);
            }
        }
    }
}

impl AssetSource for AndroidAssetSource {
    fn read_bytes(&self, path: &str) -> Result<Vec<u8>, FsError> {
        let mut asset = self.open(path)?;
        let mut bytes = Vec::with_capacity(asset.length());
        asset
            .read_to_end(&mut bytes)
            .map_err(|e| FsError::Io(path.to_string(), e))?;
        Ok(bytes)
    }

    fn read_to_string(&self, path: &str) -> Result<String, FsError> {
        let bytes = self.read_bytes(path)?;
        String::from_utf8(bytes).map_err(|_| FsError::Utf8(path.to_string()))
    }

    fn list_dir(&self, dir: &str) -> Result<Vec<String>, FsError> {
        let mut out = Vec::new();
        self.collect(dir, &mut out);
        Ok(out)
    }

    fn path_exists(&self, path: &str) -> bool {
        match CString::new(path) {
            Ok(cstr) => self.asset_manager.open(&cstr).is_some(),
            Err(_) => false,
        }
    }
}

/// Android storage rooted at the app's internal data directory.
pub struct AndroidStorage {
    inner: DirStorage,
}

impl AndroidStorage {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            // Cached files live in a `cache/` subdirectory of the app's
            // internal storage.
            inner: DirStorage::with_cache_dir(data_dir.clone(), data_dir.join("cache")),
        }
    }
}

impl Storage for AndroidStorage {
    fn read_bytes(&self, path: &str) -> Result<Vec<u8>, FsError> {
        self.inner.read_bytes(path)
    }

    fn read_to_string(&self, path: &str) -> Result<String, FsError> {
        self.inner.read_to_string(path)
    }

    fn write_bytes(&self, path: &str, data: &[u8]) -> Result<(), FsError> {
        self.inner.write_bytes(path, data)
    }

    fn open_read(&self, path: &str) -> Result<Box<dyn Read + Send>, FsError> {
        self.inner.open_read(path)
    }

    fn open_write(&self, path: &str) -> Result<Box<dyn Write + Send>, FsError> {
        self.inner.open_write(path)
    }

    fn create_dir_all(&self, path: &str) -> Result<(), FsError> {
        self.inner.create_dir_all(path)
    }

    fn path_exists(&self, path: &str) -> bool {
        self.inner.path_exists(path)
    }

    fn read_cache_bytes(&self, path: &str) -> Result<Vec<u8>, FsError> {
        self.inner.read_cache_bytes(path)
    }

    fn write_cache_bytes(&self, path: &str, data: &[u8]) -> Result<(), FsError> {
        self.inner.write_cache_bytes(path, data)
    }

    fn cache_path_exists(&self, path: &str) -> bool {
        self.inner.cache_path_exists(path)
    }
}
