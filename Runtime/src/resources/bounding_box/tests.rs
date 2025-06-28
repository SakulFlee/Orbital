use cgmath::Point3;

use crate::wgpu_test_adapter;

use super::{BoundingBox, BoundingBoxDescriptor};

#[test]
fn realization() {
    let (_, device, _) = wgpu_test_adapter::make_wgpu_connection();

    let descriptor = BoundingBoxDescriptor {
        min: Point3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
        max: Point3 {
            x: 4.0,
            y: 5.0,
            z: 6.0,
        },
    };

    let _realization = BoundingBox::new(&descriptor, &device);
}
