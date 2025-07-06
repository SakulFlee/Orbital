use cgmath::Vector2;
use log::{debug, warn};

use crate::{
    logging,
    resources::{SamplingType, WorldEnvironment, WorldEnvironmentDescriptor},
    wgpu_test_adapter,
};

fn check_if_ci() -> bool {
    if std::env::var("CI").is_ok() {
        warn!("CI environment detected!");
        warn!("Will skip this test and pass it due to it being too heavy on software rendering required by CI.");
        warn!("This does NOT mean it passed.");

        return true;
    }

    false
}

#[test]
fn test_realization_no_mip_level_count_set() {
    logging::test_init();
    if check_if_ci() {
        return;
    }

    warn!("This test utilizes caching!");
    warn!("On unexpected results, make sure to delete the cache first!");
    warn!("The cache location should be printed in the log below somewhere.");

    const SIZE: u32 = 512;

    let (_, device, queue) = wgpu_test_adapter::make_wgpu_connection();

    let descriptor = WorldEnvironmentDescriptor::FromData {
        cube_face_size: SIZE,
        data: (0..SIZE * SIZE * 6)
            .flat_map(|_| [[0u8; 4]; 4])
            .flatten()
            .collect(),
        size: Vector2 { x: SIZE, y: SIZE },
        sampling_type: SamplingType::BoxBlur,
        specular_mip_level_count: None,
    };

    let _realization = WorldEnvironment::from_descriptor(&descriptor, None, &device, &queue)
        .expect("Failed to create realization");
}

#[test]
fn test_realization_some_mip_level_count_set() {
    logging::test_init();
    if check_if_ci() {
        return;
    }

    warn!("This test utilizes caching!");
    warn!("On unexpected results, make sure to delete the cache first!");
    warn!("The cache location should be printed in the log below somewhere.");

    const SIZE: u32 = 512;

    let (_, device, queue) = wgpu_test_adapter::make_wgpu_connection();

    for i in 1..10 {
        debug!("Testing with mip level count: {i}");

        let descriptor = WorldEnvironmentDescriptor::FromData {
            cube_face_size: SIZE,
            data: (0..SIZE * SIZE * 6)
                .flat_map(|_| [[0u8; 4]; 4])
                .flatten()
                .collect(),
            size: Vector2 { x: SIZE, y: SIZE },
            sampling_type: SamplingType::BoxBlur,
            specular_mip_level_count: Some(i),
        };

        let _realization = WorldEnvironment::from_descriptor(&descriptor, None, &device, &queue)
            .expect("Failed to create realization");
    }
}

#[test]
fn test_caching() {
    logging::test_init();
    if check_if_ci() {
        return;
    }

    warn!("This test utilizes caching!");
    warn!("On unexpected results, make sure to delete the cache first!");
    warn!("The cache location should be printed in the log below somewhere.");

    const SIZE: u32 = 512;

    let (_, device, queue) = wgpu_test_adapter::make_wgpu_connection();

    let descriptor = WorldEnvironmentDescriptor::FromData {
        cube_face_size: SIZE,
        data: (0..SIZE * SIZE * 6)
            .flat_map(|_| [[0u8; 4]; 4])
            .flatten()
            .collect(),
        size: Vector2 { x: SIZE, y: SIZE },
        sampling_type: SamplingType::BoxBlur,
        specular_mip_level_count: None,
    };
    let _realization = WorldEnvironment::from_descriptor(&descriptor, None, &device, &queue)
        .expect("Failed to create realization");

    let cache_file =
        WorldEnvironment::find_cache_file(&descriptor).expect("Cache file not resolved!");

    assert!(&cache_file.exists());
    assert!(std::fs::metadata(&cache_file)
        .expect("Cache file metadata missing!")
        .is_file());
    assert!(
        std::fs::metadata(&cache_file)
            .expect("Cache file metadata missing!")
            .len()
            > 0
    );
}

#[test]
fn test_cache_dir() {
    logging::test_init();
    if check_if_ci() {
        return;
    }

    warn!("This test utilizes caching!");
    warn!("On unexpected results, make sure to delete the cache first!");
    warn!("The cache location should be printed in the log below somewhere.");

    WorldEnvironment::find_cache_dir().expect("Cache dir not resolved! NOTE: Make sure this test is running on a platform that supports caching!");
}

#[test]
fn test_cache_file() {
    logging::test_init();
    if check_if_ci() {
        return;
    }

    warn!("This test utilizes caching!");
    warn!("On unexpected results, make sure to delete the cache first!");
    warn!("The cache location should be printed in the log below somewhere.");

    const SIZE: u32 = 512;

    let (_, device, queue) = wgpu_test_adapter::make_wgpu_connection();

    let descriptor = WorldEnvironmentDescriptor::FromData {
        cube_face_size: SIZE,
        data: (0..SIZE * SIZE * 6)
            .flat_map(|_| [[0u8; 4]; 4])
            .flatten()
            .collect(),
        size: Vector2 { x: SIZE, y: SIZE },
        sampling_type: SamplingType::BoxBlur,
        specular_mip_level_count: None,
    };
    let _realization = WorldEnvironment::from_descriptor(&descriptor, None, &device, &queue)
        .expect("Failed to create realization");

    let _cache_file =
        WorldEnvironment::find_cache_file(&descriptor).expect("Cache file not resolved!");
}
