use easy_gltf::Light;

use crate::resources::descriptors::LightDescriptor;

impl From<&Light> for LightDescriptor {
    fn from(value: &Light) -> Self {
        match value {
            Light::Point {
                name,
                position,
                color,
                intensity,
            } => LightDescriptor::PointLight {
                label: name.clone().unwrap_or("Unlabelled".into()),
                position: *position,
                color: *color,
            },
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
