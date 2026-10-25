use std::{
    error::Error,
    sync::{
        Arc, RwLock,
        atomic::{AtomicU64, Ordering},
    },
};

use cgmath::Vector4;
use hashbrown::HashMap;
use wgpu::{Buffer, BufferDescriptor, BufferUsages, Device, Queue, TextureFormat};

use orbital_core::cache::{Cache, CacheEntry};
use orbital_instance::Instance;
use orbital_material_shader::{MaterialShader, MaterialShaderDescriptor};
use orbital_math::Transform;
pub use orbital_mesh::{Mesh, MeshDescriptor};
use ulid::Ulid;

mod descriptor;
pub use descriptor::*;

static MATERIAL_CACHE_HITS: AtomicU64 = AtomicU64::new(0);
static MATERIAL_CACHE_MISSES: AtomicU64 = AtomicU64::new(0);

/// Returns cumulative material cache hits and misses.
pub fn material_cache_stats() -> (u64, u64) {
    (
        MATERIAL_CACHE_HITS.load(Ordering::Relaxed),
        MATERIAL_CACHE_MISSES.load(Ordering::Relaxed),
    )
}

#[derive(Debug, Clone)]
pub struct Model {
    mesh: Arc<Mesh>,
    materials: Vec<Arc<MaterialShader>>,
    instance_count: u32,
    instance_capacity: u32,
    instance_buffer: Buffer,
    /// CPU copy of the instance buffer, used to compact visible instances
    /// without re-encoding every transform each frame.
    instance_data: Vec<u8>,
    /// World-space bounding sphere per instance: `[center.x, center.y, center.z, radius]`.
    instance_bounds: Vec<[f32; 4]>,
    /// Hash of the mesh and materials. Instance transforms are excluded so an
    /// animation update can reuse the existing GPU buffer.
    source_hash: Ulid,
}

impl Model {
    pub fn from_descriptor<'cache>(
        descriptor: &ModelDescriptor,
        surface_format: &TextureFormat,
        device: &Device,
        queue: &Queue,
        mesh_cache: &'cache RwLock<Cache<Arc<MeshDescriptor>, Mesh>>,
        material_cache: &'cache RwLock<Cache<Arc<MaterialShaderDescriptor>, MaterialShader>>,
    ) -> Result<Self, Box<dyn Error + 'cache>> {
        // --- Mesh ---
        let mesh = match mesh_cache.write() {
            Ok(mut lock) => lock
                .entry(descriptor.mesh.clone())
                .or_insert(CacheEntry::new(Mesh::from_descriptor(
                    &descriptor.mesh,
                    device,
                    queue,
                )))
                .clone_inner(),
            Err(e) => return Err(Box::new(e)),
        };

        // --- Material ---
        let mut materials = Vec::new();
        for material_descriptor in &descriptor.materials {
            materials.push(match material_cache.write() {
                Ok(mut lock) => {
                    if lock.contains_key(material_descriptor) {
                        MATERIAL_CACHE_HITS.fetch_add(1, Ordering::Relaxed);
                    } else {
                        MATERIAL_CACHE_MISSES.fetch_add(1, Ordering::Relaxed);
                    }

                    lock.entry(material_descriptor.clone())
                        .or_insert(CacheEntry::new(MaterialShader::from_descriptor(
                            material_descriptor,
                            Some(*surface_format),
                            device,
                            queue,
                        )?))
                        .clone_inner()
                }
                Err(e) => return Err(Box::new(e)),
            });
        }

        let instance_capacity = (descriptor.transforms.len() as u32).max(1);
        let instance_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Model Instance Buffer"),
            size: instance_capacity as u64 * 64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut model = Self {
            mesh,
            materials,
            instance_count: 0,
            instance_capacity,
            instance_buffer,
            instance_data: Vec::with_capacity(instance_capacity as usize * 64),
            instance_bounds: Vec::with_capacity(instance_capacity as usize),
            source_hash: descriptor.instance_hash(),
        };
        model.update_instances(&descriptor.transforms, device, queue);
        Ok(model)
    }

    /// Update the instance buffer in place.
    ///
    /// The buffer is only reallocated when the new instance count exceeds the
    /// existing capacity. This is the path animation will use every frame.
    pub fn update_instances(
        &mut self,
        transforms: &HashMap<Ulid, Transform>,
        device: &Device,
        queue: &Queue,
    ) {
        let instance_count = transforms.len() as u32;
        if instance_count > self.instance_capacity {
            let new_capacity = instance_count.next_power_of_two().max(1);
            self.instance_buffer = device.create_buffer(&BufferDescriptor {
                label: Some("Model Instance Buffer"),
                size: new_capacity as u64 * 64,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.instance_capacity = new_capacity;
            self.instance_data.reserve(new_capacity as usize * 64);
            self.instance_bounds.reserve(new_capacity as usize);
        }

        self.instance_data.clear();
        self.instance_bounds.clear();
        let bounding_sphere = self.mesh.bounding_sphere();

        for transform in transforms.values() {
            let instance = Instance::from(transform);
            self.instance_data
                .extend(instance.to_buffer_data_flattened());

            if let Some(sphere) = &bounding_sphere {
                let matrix = transform.to_matrix();
                let center =
                    matrix * Vector4::new(sphere.center.x, sphere.center.y, sphere.center.z, 1.0);
                let max_scale = transform
                    .scale
                    .x
                    .max(transform.scale.y)
                    .max(transform.scale.z);
                self.instance_bounds.push([
                    center.x,
                    center.y,
                    center.z,
                    sphere.radius * max_scale,
                ]);
            } else {
                self.instance_bounds.push([0.0, 0.0, 0.0, 0.0]);
            }
        }

        if instance_count > 0 {
            queue.write_buffer(&self.instance_buffer, 0, &self.instance_data);
        }
        self.instance_count = instance_count;
    }

    pub fn mesh(&self) -> &Mesh {
        &self.mesh
    }

    pub fn materials(&self) -> &Vec<Arc<MaterialShader>> {
        &self.materials
    }

    pub fn instance_count(&self) -> u32 {
        self.instance_count
    }

    pub fn instance_buffer(&self) -> &Buffer {
        &self.instance_buffer
    }

    /// CPU copy of the instance buffer, matching [`Self::instance_buffer`].
    pub fn instance_data(&self) -> &[u8] {
        &self.instance_data
    }

    /// World-space bounding sphere per instance: `[center.x, center.y, center.z, radius]`.
    pub fn instance_bounds(&self) -> &[[f32; 4]] {
        &self.instance_bounds
    }

    /// Hash of the mesh and materials, used to decide whether a model can
    /// update its instances in place.
    pub fn source_hash(&self) -> Ulid {
        self.source_hash
    }
}
