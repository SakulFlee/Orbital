//! iOS backends: assets via `NSBundle`, storage in the app's Documents dir.

use std::{
    io::{Read, Write},
    path::PathBuf,
};

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{class, msg_send};
use objc2_foundation::{NSArray, NSString};

use crate::dir::DirStorage;
use crate::{AssetSource, FsError, Storage};

/// Returns the app bundle's resource directory path via
/// `[[NSBundle mainBundle] resourcePath]`.
fn bundle_resource_path() -> PathBuf {
    unsafe {
        let bundle: Retained<AnyObject> = msg_send![class!(NSBundle), mainBundle];
        let resource_path: Retained<NSString> = msg_send![&bundle, resourcePath];
        PathBuf::from(resource_path.to_string())
    }
}

/// Returns the app's Documents directory via
/// `NSSearchPathForDirectoriesInDomains(NSDocumentDirectory, ...)`.
fn documents_path() -> PathBuf {
    unsafe {
        let ns_document_directory: usize = 9; // NSDocumentDirectory
        let ns_user_domain_mask: usize = 1; // NSUserDomainMask
        let paths: Retained<NSArray> = msg_send![
            class!(NSSearchPathForDirectoriesInDomains),
            ns_document_directory,
            ns_user_domain_mask,
            true,
        ];
        let first: Option<Retained<NSString>> = msg_send![&paths, firstObject];
        match first {
            Some(p) => PathBuf::from(p.to_string()),
            None => PathBuf::new(),
        }
    }
}

/// Returns the app's Caches directory via
/// `NSSearchPathForDirectoriesInDomains(NSCachesDirectory, ...)`.
fn caches_path() -> PathBuf {
    unsafe {
        let ns_caches_directory: usize = 13; // NSCachesDirectory
        let ns_user_domain_mask: usize = 1; // NSUserDomainMask
        let paths: Retained<NSArray> = msg_send![
            class!(NSSearchPathForDirectoriesInDomains),
            ns_caches_directory,
            ns_user_domain_mask,
            true,
        ];
        let first: Option<Retained<NSString>> = msg_send![&paths, firstObject];
        match first {
            Some(p) => PathBuf::from(p.to_string()),
            None => PathBuf::new(),
        }
    }
}

/// iOS asset source backed by the app bundle's resource directory.
pub struct IosAssetSource {
    inner: crate::dir::DirStorage,
}

impl IosAssetSource {
    pub fn new() -> Self {
        Self {
            inner: DirStorage::new(bundle_resource_path()),
        }
    }
}

impl AssetSource for IosAssetSource {
    fn read_bytes(&self, path: &str) -> Result<Vec<u8>, FsError> {
        self.inner.read_bytes(path)
    }

    fn read_to_string(&self, path: &str) -> Result<String, FsError> {
        self.inner.read_to_string(path)
    }

    fn list_dir(&self, dir: &str) -> Result<Vec<String>, FsError> {
        // DirStorage doesn't have list_dir, so we use fs::read_dir directly.
        let root = self.inner.resolve(dir);
        let mut out = Vec::new();
        collect_files(&root, &root, &mut out);
        Ok(out)
    }

    fn path_exists(&self, path: &str) -> bool {
        self.inner.path_exists(path)
    }
}

/// iOS storage rooted at the app's Documents directory.
pub struct IosStorage {
    inner: DirStorage,
}

impl IosStorage {
    pub fn new() -> Self {
        let docs = documents_path();
        let cache = caches_path();
        Self {
            inner: DirStorage::with_cache_dir(docs.clone(), cache),
        }
    }
}

impl Storage for IosStorage {
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

    fn remove_file(&self, path: &str) -> Result<(), FsError> {
        self.inner.remove_file(path)
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
fn collect_files(root: &std::path::Path, dir: &std::path::Path, out: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
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
