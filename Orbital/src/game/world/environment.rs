use crate::resources::descriptors::{CubeTextureDescriptor, MaterialDescriptor};

#[derive(Debug)]
pub struct WorldEnvironment {
    pub sky_box_material_descriptor: MaterialDescriptor,
}

impl WorldEnvironment {
    pub fn new(sky_box_material_descriptor: MaterialDescriptor) -> Self {
        if let MaterialDescriptor::SkyBox { sky_texture: _ } = &sky_box_material_descriptor {
            Self {
                sky_box_material_descriptor,
            }
        } else {
            panic!("WorldEnvironment requires the SkyBox Material to be of type MaterialDescriptor::SkyBox!");
        }
    }

    pub fn sky_box_material_descriptor(&self) -> &MaterialDescriptor {
        &self.sky_box_material_descriptor
    }
}

impl Default for WorldEnvironment {
    fn default() -> Self {
        Self {
            sky_box_material_descriptor: MaterialDescriptor::SkyBox {
                sky_texture: CubeTextureDescriptor::RadianceHDRFile {
                    cube_face_size: 1024,
                    path: "Assets/HDRs/lonely_road_afternoon_puresky_4k.hdr",
                },
            },
        }
    }
}
