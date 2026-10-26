//! Persistent, descriptor-keyed transient texture pool.
//!
//! Render targets are reused across frames instead of being recreated every
//! frame. Within a single frame a descriptor is not aliased; repeated requests
//! create separate textures so passes cannot accidentally read and write the
//! same target.

use wgpu::{
    Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureView, TextureViewDescriptor,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct TextureKey {
    width: u32,
    height: u32,
    depth_or_array_layers: u32,
    mip_level_count: u32,
    sample_count: u32,
    dimension: TextureDimension,
    format: TextureFormat,
    usage: u32,
    view_formats: Vec<TextureFormat>,
}

impl TextureKey {
    fn from_descriptor(descriptor: &TextureDescriptor<'_>) -> Self {
        Self {
            width: descriptor.size.width,
            height: descriptor.size.height,
            depth_or_array_layers: descriptor.size.depth_or_array_layers,
            mip_level_count: descriptor.mip_level_count,
            sample_count: descriptor.sample_count,
            dimension: descriptor.dimension,
            format: descriptor.format,
            usage: descriptor.usage.bits(),
            view_formats: descriptor.view_formats.to_vec(),
        }
    }
}

struct PoolEntry {
    key: TextureKey,
    texture: Texture,
    view: TextureView,
    last_used: u64,
}

/// A texture borrowed from the pool for the current frame.
#[derive(Clone, Debug)]
pub struct PooledTexture {
    pub texture: Texture,
    pub view: TextureView,
}

/// Descriptor-keyed texture pool.
pub struct TexturePool {
    entries: Vec<PoolEntry>,
    frame: u64,
    max_age: u64,
}

impl Default for TexturePool {
    fn default() -> Self {
        Self::new(3)
    }
}

impl TexturePool {
    /// Create a pool that keeps unused textures for `max_age` frames.
    pub fn new(max_age: u64) -> Self {
        Self {
            entries: Vec::new(),
            frame: 0,
            max_age,
        }
    }

    /// Start a new frame. Textures used in this frame will not be aliased to
    /// another request in the same frame.
    pub fn begin_frame(&mut self, frame: u64) {
        self.frame = frame;
    }

    /// Acquire a texture matching `descriptor`, creating it if necessary.
    ///
    /// The returned view uses the default view descriptor, which is correct for
    /// the 2D render targets the renderer currently requests.
    pub fn acquire(
        &mut self,
        device: &wgpu::Device,
        descriptor: &TextureDescriptor<'_>,
    ) -> PooledTexture {
        let key = TextureKey::from_descriptor(descriptor);

        if let Some(entry) = self
            .entries
            .iter_mut()
            .find(|entry| entry.key == key && entry.last_used != self.frame)
        {
            entry.last_used = self.frame;
            return PooledTexture {
                texture: entry.texture.clone(),
                view: entry.view.clone(),
            };
        }

        let texture = device.create_texture(descriptor);
        let view = texture.create_view(&TextureViewDescriptor::default());
        self.entries.push(PoolEntry {
            key,
            texture: texture.clone(),
            view: view.clone(),
            last_used: self.frame,
        });

        PooledTexture { texture, view }
    }

    /// Drop textures that have not been used for longer than `max_age` frames.
    pub fn end_frame(&mut self) {
        let current = self.frame;
        let max_age = self.max_age;
        self.entries
            .retain(|entry| current.saturating_sub(entry.last_used) <= max_age);
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn descriptor(width: u32) -> TextureDescriptor<'static> {
        TextureDescriptor {
            label: Some("test"),
            size: wgpu::Extent3d {
                width,
                height: 720,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        }
    }

    #[test]
    fn key_tracks_descriptor_differences() {
        let a = TextureKey::from_descriptor(&descriptor(1280));
        let b = TextureKey::from_descriptor(&descriptor(1920));
        let c = TextureKey::from_descriptor(&descriptor(1280));

        assert_ne!(a, b);
        assert_eq!(a, c);
    }
}
