use cgmath::Vector2;
use log::warn;

use crate::{SamplingType, WorldEnvironment, WorldEnvironmentDescriptor};

#[test]
fn test_realization() {
    logging::init();
    warn!("This test utilizes caching!");
    warn!("On unexpected results, make sure to delete the cache first!");
    warn!("The cache location should be printed in the log below somewhere.");

    const SIZE: u32 = 512;

    let (_, device, queue) = wgpu_test_adapter::make_wgpu_connection();

    let descriptor = WorldEnvironmentDescriptor::FromData {
        // TODO: Add a specular level count
        cube_face_size: SIZE,
        data: (0..SIZE * SIZE * 6)
            .into_iter()
            .map(|_| [[0u8; 4]; 4])
            .flatten()
            .flatten()
            .collect(),
        size: Vector2 { x: SIZE, y: SIZE },
        sampling_type: SamplingType::BoxBlur,
    };

    let _realization = WorldEnvironment::from_descriptor(&descriptor, &device, &queue, "Test");
}
