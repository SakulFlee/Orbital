use std::path::Path;

use image::{DynamicImage, GenericImageView};
use wgpu::{
    AddressMode, CompareFunction, Device, Extent3d, FilterMode, ImageCopyTexture, ImageDataLayout,
    Origin3d, Queue, Sampler, SamplerDescriptor, SurfaceConfiguration, Texture as WTexture,
    TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
    TextureViewDescriptor,
};

// TODO: "from_path" should be ... from path, not string >_>

pub struct Texture {
    texture: WTexture,
    view: TextureView,
    sampler: Sampler,
}

impl Texture {
    pub const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;

    pub fn from_path(logical_device: &LogicalDevice, file_name: &str) -> Result<Self, String> {
        let resource_folder = if cfg!(debug_assertions) {
            Path::new(env!("OUT_DIR")).join("res")
        } else {
            Path::new(".").join("res")
        };
        // TODO: Resource Manager

        let file_path = resource_folder.join(file_name);
        let file_path_str = file_path
            .to_str()
            .ok_or("Couldn't convert file path to string")?;

        let bytes = std::fs::read(&file_path).map_err(|x| format!("Failed to read file: {}", x))?;

        Self::from_bytes(logical_device, &bytes, file_path_str)
    }

    pub fn from_bytes(
        logical_device: &LogicalDevice,
        bytes: &[u8],
        label: &str,
    ) -> Result<Self, String> {
        let image = image::load_from_memory(bytes).map_err(|e| e.to_string())?;
        Self::from_image(logical_device, &image, Some(label))
    }

    pub fn from_image(
        logical_device: &LogicalDevice,
        image: &DynamicImage,
        label: Option<&str>,
    ) -> Result<Self, String> {
        // Convert the image into something useable
        let rgba = image.to_rgba8();
        // Convert the image size into something useable
        let dimensions = image.dimensions();
        let size = Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        // Make texture
        let texture = device.create_texture(&TextureDescriptor {
            label: label,
            size: size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Fill texture
        queue.write_texture(
            ImageCopyTexture {
                aspect: TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
            },
            &rgba,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        // Create texture view
        let view = texture.create_view(&TextureViewDescriptor::default());

        // Create texture sampler
        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }

    pub fn make_depth_texture(
        device: &Device,
        config: &SurfaceConfiguration,
        label: Option<&str>,
    ) -> Self {
        let size = Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };

        let descriptor = TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&descriptor);

        let view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            compare: Some(CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }

    pub fn get_texture(&self) -> &WTexture {
        &self.texture
    }

    pub fn get_view(&self) -> &TextureView {
        &self.view
    }

    pub fn get_sampler(&self) -> &Sampler {
        &self.sampler
    }
}
