use easy_gltf::Light;

use crate::resources::{descriptors::LightDescriptor, realizations::PointLight};

impl From<&Light> for LightDescriptor {
    fn from(value: &Light) -> Self {
        match value {
            Light::Point {
                name,
                position,
                color,
                intensity,
            } => {
                let point_light = PointLight {
                    label: name.clone().unwrap_or("unlabelled".into()),
                    position: *position,
                    color: *color,
                };

                LightDescriptor::PointLight(point_light)
            }
            Light::Directional {
                name,
                direction,
                color,
                intensity,
            } => unimplemented!(),
            Light::Spot {
                name,
                position,
                direction,
                color,
                intensity,
                inner_cone_angle,
                outer_cone_angle,
            } => unimplemented!(),
        }
    }
}
