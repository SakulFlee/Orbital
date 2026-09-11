//! Desktop-focused tests for the [`FileManager`](crate::FileManager) backends.
//!
//! The Android backends cannot be exercised on the host, so these only cover the
//! `std::fs`-backed storage and asset behavior.

use orbital_file_manager::{FileManager, Storage};

fn temp_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("orbital_fm_test_{tag}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

#[test]
fn storage_write_read_round_trip() {
    let base = temp_dir("round_trip");
    let storage = orbital_file_manager::DirStorage::new(base.clone());

    storage
        .write_bytes("nested/dir/file.bin", &[1, 2, 3, 4])
        .expect("write creates parent dirs");
    assert!(storage.path_exists("nested/dir/file.bin"));

    let bytes = storage
        .read_bytes("nested/dir/file.bin")
        .expect("read back");
    assert_eq!(bytes, vec![1, 2, 3, 4]);

    let text = "hello storage";
    storage
        .write_bytes("nested/note.txt", text.as_bytes())
        .expect("write text");
    assert_eq!(
        storage
            .read_to_string("nested/note.txt")
            .expect("read text"),
        text
    );

    let _ = std::fs::remove_dir_all(base);
}

#[test]
fn storage_missing_file_is_not_found() {
    let base = temp_dir("missing");
    let storage = orbital_file_manager::DirStorage::new(base.clone());

    assert!(matches!(
        storage.read_bytes("does/not/exist.bin"),
        Err(orbital_file_manager::FsError::NotFound(_))
    ));
    assert!(!storage.path_exists("does/not/exist.bin"));

    let _ = std::fs::remove_dir_all(base);
}

#[test]
fn cache_namespace_is_separate_from_storage() {
    let base = temp_dir("cache");
    let cache = temp_dir("cache_dir");
    let storage = orbital_file_manager::DirStorage::with_cache_dir(base.clone(), cache.clone());

    storage
        .write_cache_bytes("Orbital/IBLs/abc.bin", &[9, 8, 7])
        .expect("write cache creates parent dirs");
    assert!(storage.cache_path_exists("Orbital/IBLs/abc.bin"));
    assert_eq!(
        storage
            .read_cache_bytes("Orbital/IBLs/abc.bin")
            .expect("read cache"),
        vec![9, 8, 7]
    );

    // The cache write must not appear in the storage namespace.
    assert!(!storage.path_exists("Orbital/IBLs/abc.bin"));

    let _ = std::fs::remove_dir_all(base);
    let _ = std::fs::remove_dir_all(cache);
}

#[test]
fn desktop_cache_uses_platform_cache_dir() {
    let fm = FileManager::global().expect("global file manager");

    // Writing through the cache namespace must end up under the platform cache
    // dir, not the working directory.
    let probe = "orbital_fm_cache_probe.bin";
    fm.write_cache_bytes(probe, &[1, 2, 3])
        .expect("write cache");
    assert!(fm.cache_path_exists(probe));

    let cache_dir = dirs::cache_dir().expect("desktop cache dir exists");
    assert!(cache_dir.join(probe).exists());
    assert!(!std::env::current_dir().unwrap().join(probe).exists());

    let _ = std::fs::remove_file(cache_dir.join(probe));
}

#[test]
fn global_file_manager_resolves_from_working_directory() {
    // Runs from the package root; write a probe into the current working dir's
    // `Assets/` and confirm the global (desktop) backend reads it back.
    let fm = FileManager::global().expect("global file manager");
    let probe = "file_manager_global_probe.txt";
    let base = std::env::current_dir().expect("cwd");
    let assets = base.join("Assets");
    let _ = std::fs::create_dir_all(&assets);
    std::fs::write(assets.join(probe), "probe").expect("write probe");

    assert!(fm.asset_exists(probe));
    assert_eq!(fm.read_asset_to_string(probe).expect("read probe"), "probe");

    let _ = std::fs::remove_file(assets.join(probe));
}
