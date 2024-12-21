use cgmath::{EuclideanSpace, Point3};
use easy_gltf::{Camera, Projection};

use crate::resources::descriptors::CameraDescriptor;

impl From<&Camera> for CameraDescriptor {
    fn from(value: &Camera) -> Self {
        let identifier = if let Some(name) = &value.name {
            name.clone()
        } else {
            String::from("Unnamed glTF Camera")
        };

        // TODO: Validate this calculation!
        let forward = value.forward();
        let yaw = forward.y.atan2(forward.x);
        let pitch = -forward.z.asin();

        match value.projection {
            Projection::Perspective {
                yfov: fovy,
                aspect_ratio,
            } => CameraDescriptor {
                label: identifier,
                position: Point3::from_vec(value.position()),
                yaw,
                pitch,
                fovy: fovy.0,
                aspect: aspect_ratio.unwrap_or(16.0 / 9.0),
                near: value.znear,
                far: value.zfar,
                ..Default::default()
            },
            Projection::Orthographic { scale } => unimplemented!(),
        }
    }
}
