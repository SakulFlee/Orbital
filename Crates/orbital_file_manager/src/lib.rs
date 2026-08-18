//! Cross-platform file access abstraction for Orbital.
//!
//! Splits I/O into two namespaces:
//! - **Assets**: read-only, bundled resources (e.g. `"Models/foo.glb"`,
//!   `"Shaders/pbr.wgsl"`). Paths are relative to the asset root and carry no
//!   `Assets/` prefix.
//! - **Storage**: read-write application data (IBL cache, saved scenes, logs).
//!
//! Platform behavior:
//! - **Desktop**: assets resolve against `<cwd>/Assets`, storage against `<cwd>`.
//! - **Android**: assets are read through `AAssetManager` from the APK; storage
//!   lands in the app's internal data directory.
//!
//! The process-wide instance is accessed via [`FileManager::global`]. On Android
//! it must be initialized first from `android_main` through
//! [`FileManager::init_android_global`] (wired up by the `orbital` entry-point
//! macros).

#[cfg(target_os = "android")]
mod android;
mod dir;
mod error;

#[cfg(target_os = "android")]
pub use android::{AndroidAssetSource, AndroidStorage};
#[cfg(not(target_os = "android"))]
pub use dir::{DesktopAssetSource, DesktopStorage};

pub use dir::DirStorage;

pub use error::FsError;

use std::{
    io::{Read, Write},
    sync::OnceLock,
};

/// Read-only, bundled asset access.
pub trait AssetSource: Send + Sync {
    /// Reads the full contents of an asset as bytes.
    fn read_bytes(&self, path: &str) -> Result<Vec<u8>, FsError>;
    /// Reads the full contents of an asset as UTF-8 text.
    fn read_to_string(&self, path: &str) -> Result<String, FsError>;
    /// Lists the files under `dir`, recursively, as forward-slash paths
    /// relative to `dir` itself (e.g. `"sub/foo.wgsl"`).
    fn list_dir(&self, dir: &str) -> Result<Vec<String>, FsError>;
    /// Whether the given asset path exists.
    fn path_exists(&self, path: &str) -> bool;
}

/// Read-write application storage.
pub trait Storage: Send + Sync {
    /// Reads the full contents of a stored file as bytes.
    fn read_bytes(&self, path: &str) -> Result<Vec<u8>, FsError>;
    /// Reads the full contents of a stored file as UTF-8 text.
    fn read_to_string(&self, path: &str) -> Result<String, FsError>;
    /// Writes bytes to a stored file, creating parent directories as needed.
    fn write_bytes(&self, path: &str, data: &[u8]) -> Result<(), FsError>;
    /// Opens a stored file for reading.
    fn open_read(&self, path: &str) -> Result<Box<dyn Read + Send>, FsError>;
    /// Opens a stored file for writing (truncating), creating parent
    /// directories as needed.
    fn open_write(&self, path: &str) -> Result<Box<dyn Write + Send>, FsError>;
    /// Recursively creates a directory.
    fn create_dir_all(&self, path: &str) -> Result<(), FsError>;
    /// Whether the given stored path exists.
    fn path_exists(&self, path: &str) -> bool;

    /// Deletes a stored file, if it exists.
    fn remove_file(&self, path: &str) -> Result<(), FsError>;

    /// Reads the full contents of a cached file as bytes.
    fn read_cache_bytes(&self, path: &str) -> Result<Vec<u8>, FsError>;
    /// Writes bytes to a cached file, creating parent directories as needed.
    fn write_cache_bytes(&self, path: &str, data: &[u8]) -> Result<(), FsError>;
    /// Whether the given cached path exists.
    fn cache_path_exists(&self, path: &str) -> bool;
}

/// The process-wide file manager.
pub struct FileManager {
    assets: Box<dyn AssetSource>,
    storage: Box<dyn Storage>,
}

static GLOBAL: OnceLock<FileManager> = OnceLock::new();

impl FileManager {
    /// Builds a [`FileManager`] from custom asset and storage backends.
    ///
    /// Useful for embedding with a custom asset root or for tests. The
    /// process-wide instance used by [`FileManager::global`] is unaffected.
    pub fn new(assets: Box<dyn AssetSource>, storage: Box<dyn Storage>) -> Self {
        Self { assets, storage }
    }

    /// Returns the process-wide [`FileManager`].
    ///
    /// On desktop this lazily initializes a working-directory-backed backend.
    /// On Android the backend must have been set up first via
    /// [`FileManager::init_android_global`], otherwise
    /// [`FsError::NotInitialized`] is returned.
    pub fn global() -> Result<&'static FileManager, FsError> {
        #[cfg(target_os = "android")]
        {
            GLOBAL.get().ok_or(FsError::NotInitialized)
        }

        #[cfg(not(target_os = "android"))]
        {
            Ok(GLOBAL.get_or_init(|| FileManager {
                assets: Box::new(DesktopAssetSource::new()),
                storage: Box::new(DesktopStorage::new()),
            }))
        }
    }

    /// Initializes the Android backend from the native activity handed to
    /// `android_main`. Called by the `make_main!` / `make_android_main!` macros
    /// before the event loop is created.
    #[cfg(target_os = "android")]
    pub fn init_android_global(
        asset_manager: ndk::asset::AssetManager,
        data_dir: Option<std::path::PathBuf>,
    ) -> Result<(), FsError> {
        let data_dir = data_dir.unwrap_or_default();
        let _ = GLOBAL.set(FileManager {
            assets: Box::new(AndroidAssetSource::new(asset_manager)),
            storage: Box::new(AndroidStorage::new(data_dir)),
        });
        Ok(())
    }

    /// Reads an asset as bytes.
    pub fn read_asset_bytes(&self, path: &str) -> Result<Vec<u8>, FsError> {
        self.assets.read_bytes(path)
    }

    /// Reads an asset as UTF-8 text.
    pub fn read_asset_to_string(&self, path: &str) -> Result<String, FsError> {
        self.assets.read_to_string(path)
    }

    /// Lists files under an asset directory (recursively, relative paths).
    pub fn list_asset_dir(&self, dir: &str) -> Result<Vec<String>, FsError> {
        self.assets.list_dir(dir)
    }

    /// Whether an asset exists.
    pub fn asset_exists(&self, path: &str) -> bool {
        self.assets.path_exists(path)
    }

    /// Reads a stored file as bytes.
    pub fn read_bytes(&self, path: &str) -> Result<Vec<u8>, FsError> {
        self.storage.read_bytes(path)
    }

    /// Reads a stored file as UTF-8 text.
    pub fn read_to_string(&self, path: &str) -> Result<String, FsError> {
        self.storage.read_to_string(path)
    }

    /// Writes bytes to a stored file.
    pub fn write_bytes(&self, path: &str, data: &[u8]) -> Result<(), FsError> {
        self.storage.write_bytes(path, data)
    }

    /// Opens a stored file for reading.
    pub fn open_read(&self, path: &str) -> Result<Box<dyn Read + Send>, FsError> {
        self.storage.open_read(path)
    }

    /// Opens a stored file for writing.
    pub fn open_write(&self, path: &str) -> Result<Box<dyn Write + Send>, FsError> {
        self.storage.open_write(path)
    }

    /// Recursively creates a directory in storage.
    pub fn create_dir_all(&self, path: &str) -> Result<(), FsError> {
        self.storage.create_dir_all(path)
    }

    /// Whether a stored path exists.
    pub fn storage_path_exists(&self, path: &str) -> bool {
        self.storage.path_exists(path)
    }

    /// Deletes a stored file, if it exists.
    pub fn remove_file(&self, path: &str) -> Result<(), FsError> {
        self.storage.remove_file(path)
    }

    /// Reads a cached file as bytes.
    pub fn read_cache_bytes(&self, path: &str) -> Result<Vec<u8>, FsError> {
        self.storage.read_cache_bytes(path)
    }

    /// Writes bytes to a cached file.
    pub fn write_cache_bytes(&self, path: &str, data: &[u8]) -> Result<(), FsError> {
        self.storage.write_cache_bytes(path, data)
    }

    /// Whether a cached path exists.
    pub fn cache_path_exists(&self, path: &str) -> bool {
        self.storage.cache_path_exists(path)
    }
}
