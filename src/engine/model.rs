use std::{mem::size_of, ops::Range, path::Path};

use bytemuck::{Pod, Zeroable};
use tobj::LoadOptions;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupLayout, BufferAddress, BufferUsages, Device, Queue, VertexAttribute,
    VertexBufferLayout, VertexFormat, VertexStepMode,
};

use crate::{texture::Texture, Vertex};

// TODO: "from_path" should be ... from path, not string >_>

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}

impl Vertex for ModelVertex {
    fn descriptor() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<ModelVertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: size_of::<[f32; 3]>() as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
                VertexAttribute {
                    offset: size_of::<[f32; 5]>() as BufferAddress,
                    shader_location: 2,
                    format: VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}

impl Model {
    pub fn from_path(
        file_name: &str,
        device: &Device,
        queue: &Queue,
        bind_group_layout: &BindGroupLayout,
    ) -> Result<Self, String> {
        let resource_folder = if cfg!(debug_assertions) {
            Path::new(env!("OUT_DIR")).join("res")
        } else {
            Path::new(".").join("res")
        };
        // TODO: Resource Manager

        let (obj_models, obj_materials) = tobj::load_obj(
            resource_folder.join(file_name),
            &LoadOptions {
                // Single index mesh (optimized for real-time GPU rendering)
                single_index: true,
                // Triangulate polygons
                triangulate: true,
                // Whether to skip points that aren't used
                ignore_points: false,
                // Whether to skip lines that aren't used
                ignore_lines: false,
            },
        )
        .map_err(|e| e.to_string())?;
        let obj_materials = obj_materials.map_err(|x| x.to_string())?;

        let meshes = obj_models
            .iter()
            .map(|x| Mesh::from_obj_model(x, &device))
            .collect::<Vec<Mesh>>();

        let mut materials: Vec<Material> = Vec::new();
        for obj_material in obj_materials {
            if obj_material.diffuse_texture.is_none() {
                // No texture == skip entry
                continue;
            }

            let mut texture_path = String::new();
            if let Some(parent) = Path::new(&file_name).parent() {
                texture_path += parent.to_str().unwrap(); // TODO: Probably unsafe and may break in some scenarios
            }
            texture_path += "/";
            texture_path += &obj_material.diffuse_texture.unwrap();

            let diffuse_material =
                Material::from_path(&texture_path, device, queue, bind_group_layout);

            if diffuse_material.is_err() {
                return Err(format!(
                    "Failed to load at least one material: {}",
                    diffuse_material.err().unwrap()
                ));
            }

            materials.push(diffuse_material.unwrap());
        }

        Ok(Model { meshes, materials })
    }
}

pub struct Mesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub material: usize,
    pub instance_range: Range<u32>,
}

impl Mesh {
    pub fn from_obj_model(model: &tobj::Model, device: &wgpu::Device) -> Self {
        // Loop over the mesh and convert each point to our Vertex type
        let model_vertices = (0..model.mesh.positions.len() / 3)
            .map(|i| ModelVertex {
                // One triangle has three vertices, so we multiply by 3 to get the index of the data
                position: [
                    model.mesh.positions[i * 3],
                    model.mesh.positions[i * 3 + 1],
                    model.mesh.positions[i * 3 + 2],
                ],
                // Texture coordinates are 2D and not 3D like position & normals
                // tex_coords: [model.mesh.texcoords[i * 2], model.mesh.texcoords[i * 2 + 1]],  // TODO: Crashes if there are no texture
                tex_coords: [0.0, 1.0],
                // Similar to the position of each vertex we have a normal point at that location, which needs to be extracted in the same way
                normal: [
                    model.mesh.normals[i * 3],
                    model.mesh.normals[i * 3 + 1],
                    model.mesh.normals[i * 3 + 2],
                ],
            })
            .collect::<Vec<_>>();

        // Build Vertex Buffer
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some(&format!("{} Vertex Buffer", &model.name)),
            contents: bytemuck::cast_slice(&model_vertices),
            usage: BufferUsages::VERTEX,
        });

        // Build Index Buffer
        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some(&format!("{} Index Buffer", &model.name)),
            contents: bytemuck::cast_slice(&model.mesh.indices),
            usage: BufferUsages::INDEX,
        });

        Mesh {
            name: model.name.clone(),
            vertex_buffer,
            index_buffer,
            num_elements: model.mesh.indices.len() as u32,
            material: model.mesh.material_id.unwrap_or(0),
            instance_range: 0..3,
        }
    }

    pub fn set_instance_range(&mut self, instance_range: Range<u32>) {
        self.instance_range = instance_range;
    }
}

pub struct Material {
    pub name: String,
    pub diffuse_texture: Texture,
    pub bind_group: BindGroup,
}

impl Material {
    pub fn from_path(
        file_name: &str,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Result<Self, String> {
        let texture = Texture::from_path(device, queue, file_name)?;

        Ok(Self::from_texture(
            file_name,
            texture,
            device,
            bind_group_layout,
        ))
    }

    pub fn from_texture(
        name: &str,
        diffuse_texture: Texture,
        device: &Device,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(name),
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.get_view()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.get_sampler()),
                },
            ],
        });

        Self {
            name: name.to_string(),
            diffuse_texture,
            bind_group,
        }
    }
}
