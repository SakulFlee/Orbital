//! Working-directory-backed backends used on desktop.

use std::{
    fs,
    io::{Read, Write},
    path::PathBuf,
};

use crate::{FsError, Storage};

#[cfg(not(target_os = "android"))]
use crate::AssetSource;
#[cfg(not(target_os = "android"))]
use std::path::Path;

/// Desktop asset source: resolves relative paths against `<cwd>/Assets`.
#[cfg(not(target_os = "android"))]
pub struct DesktopAssetSource {
    base_dir: PathBuf,
}

#[cfg(not(target_os = "android"))]
impl DesktopAssetSource {
    pub fn new() -> Self {
        Self::with_base_dir(
            std::env::current_dir()
                .map(|cwd| cwd.join("Assets"))
                .unwrap_or_else(|_| PathBuf::from("Assets")),
        )
    }

    /// Creates a desktop asset source rooted at the given base directory.
    pub fn with_base_dir(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    fn resolve(&self, path: &str) -> PathBuf {
        self.base_dir.join(path)
    }
}

#[cfg(not(target_os = "android"))]
impl Default for DesktopAssetSource {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(target_os = "android"))]
impl AssetSource for DesktopAssetSource {
    fn read_bytes(&self, path: &str) -> Result<Vec<u8>, FsError> {
        fs::read(self.resolve(path)).map_err(|e| FsError::from_io(path, e))
    }

    fn read_to_string(&self, path: &str) -> Result<String, FsError> {
        fs::read_to_string(self.resolve(path)).map_err(|e| FsError::from_io(path, e))
    }

    fn list_dir(&self, dir: &str) -> Result<Vec<String>, FsError> {
        let root = self.resolve(dir);
        let mut out = Vec::new();
        collect_files(&root, &root, &mut out);
        Ok(out)
    }

    fn path_exists(&self, path: &str) -> bool {
        self.resolve(path).exists()
    }
}

/// A `std::fs`-backed [`Storage`] rooted at a fixed directory.
pub struct DirStorage {
    base_dir: PathBuf,
    cache_dir: PathBuf,
}

impl DirStorage {
    /// Creates storage rooted at `base_dir`, with the cache sharing that root.
    pub fn new(base_dir: PathBuf) -> Self {
        Self::with_cache_dir(base_dir.clone(), base_dir)
    }

    /// Creates storage rooted at `base_dir`, with cached files rooted at
    /// `cache_dir`.
    pub fn with_cache_dir(base_dir: PathBuf, cache_dir: PathBuf) -> Self {
        Self { base_dir, cache_dir }
    }

    fn resolve(&self, path: &str) -> PathBuf {
        self.base_dir.join(path)
    }

    fn resolve_cache(&self, path: &str) -> PathBuf {
        self.cache_dir.join(path)
    }
}

impl Storage for DirStorage {
    fn read_bytes(&self, path: &str) -> Result<Vec<u8>, FsError> {
        fs::read(self.resolve(path)).map_err(|e| FsError::from_io(path, e))
    }

    fn read_to_string(&self, path: &str) -> Result<String, FsError> {
        fs::read_to_string(self.resolve(path)).map_err(|e| FsError::from_io(path, e))
    }

    fn write_bytes(&self, path: &str, data: &[u8]) -> Result<(), FsError> {
        let full = self.resolve(path);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).map_err(|e| FsError::from_io(path, e))?;
        }
        fs::write(&full, data).map_err(|e| FsError::from_io(path, e))
    }

    fn open_read(&self, path: &str) -> Result<Box<dyn Read + Send>, FsError> {
        let file = fs::File::open(self.resolve(path)).map_err(|e| FsError::from_io(path, e))?;
        Ok(Box::new(file))
    }

    fn open_write(&self, path: &str) -> Result<Box<dyn Write + Send>, FsError> {
        let full = self.resolve(path);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).map_err(|e| FsError::from_io(path, e))?;
        }
        let file = fs::File::create(&full).map_err(|e| FsError::from_io(path, e))?;
        Ok(Box::new(file))
    }

    fn create_dir_all(&self, path: &str) -> Result<(), FsError> {
        fs::create_dir_all(self.resolve(path)).map_err(|e| FsError::from_io(path, e))
    }

    fn path_exists(&self, path: &str) -> bool {
        self.resolve(path).exists()
    }

    fn read_cache_bytes(&self, path: &str) -> Result<Vec<u8>, FsError> {
        fs::read(self.resolve_cache(path)).map_err(|e| FsError::from_io(path, e))
    }

    fn write_cache_bytes(&self, path: &str, data: &[u8]) -> Result<(), FsError> {
        let full = self.resolve_cache(path);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).map_err(|e| FsError::from_io(path, e))?;
        }
        fs::write(&full, data).map_err(|e| FsError::from_io(path, e))
    }

    fn cache_path_exists(&self, path: &str) -> bool {
        self.resolve_cache(path).exists()
    }
}

/// Desktop storage: rooted at the process working directory.
#[cfg(not(target_os = "android"))]
pub struct DesktopStorage {
    inner: DirStorage,
}

#[cfg(not(target_os = "android"))]
impl DesktopStorage {
    pub fn new() -> Self {
        let cwd = std::env::current_dir().unwrap_or_default();
        let cache_dir = dirs::cache_dir().unwrap_or_else(|| cwd.clone());
        Self {
            inner: DirStorage::with_cache_dir(cwd, cache_dir),
        }
    }
}

#[cfg(not(target_os = "android"))]
impl Default for DesktopStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(target_os = "android"))]
impl Storage for DesktopStorage {
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

/// Recursively collects file paths under `dir`, relative to `root`.
#[cfg(not(target_os = "android"))]
fn collect_files(root: &Path, dir: &Path, out: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files(root, &path, out);
        } else if let Ok(relative) = path.strip_prefix(root) {
            out.push(relative.to_string_lossy().replace('\\', "/"));
        }
    }
}
