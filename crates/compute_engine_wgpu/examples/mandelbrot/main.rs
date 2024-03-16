// Full credit of this goes to wgpu (https://github.com/gfx-rs/wgpu).
// This example was taken from their examples and mildly modified to work
// with our engine version.
// The original example is licensed under Apache-2.0 & MIT and
// can be found here:
// https://github.com/gfx-rs/wgpu/tree/trunk/examples/src/storage_texture

//! This example demonstrates the basic usage of storage textures for the purpose of
//! creating a digital image of the Mandelbrot set
//! (<https://en.wikipedia.org/wiki/Mandelbrot_set>).
//!
//! Storage textures work like normal textures but they operate similar to storage buffers
//! in that they can be written to. The issue is that as it stands, write-only is the
//! only valid access mode for storage textures in WGSL and although there is a WGPU feature
//! to allow for read-write access, this is unfortunately a native-only feature and thus
//! we won't be using it here. If we needed a reference texture, we would need to add a
//! second texture to act as a reference and attach that as well. Luckily, we don't need
//! to read anything in our shader except the dimensions of our texture, which we can
//! easily get via `textureDimensions`.
//!
//! A lot of things aren't explained here via comments. See hello-compute and
//! repeated-compute for code that is more thoroughly commented.

use std::io::Write;

use compute_engine_wgpu::{ComputeEngineTrait, ComputeEngineWGPU};
use logging::info;

const TEXTURE_DIMS: (usize, usize) = (512, 512);

#[tokio::main]
pub async fn main() {
    logging::log_init();

    info!("Full credit of this goes to wgpu (https://github.com/gfx-rs/wgpu).");
    info!("This example was taken from their examples and mildly modified to work with our engine version.");
    info!("The original example is licensed under Apache-2.0 & MIT and can be found here:");
    info!("https://github.com/gfx-rs/wgpu/tree/trunk/examples/src/storage_texture");
    info!("");

    let mut texture_data = vec![0u8; TEXTURE_DIMS.0 * TEXTURE_DIMS.1 * 4];

    let compute_engine = ComputeEngineWGPU::new()
        .await
        .expect("Compute Engine initialization failed!");

    let shader = compute_engine
        .device()
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "shader.wgsl"
            ))),
        });

    let storage_texture = compute_engine
        .device()
        .create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: TEXTURE_DIMS.0 as u32,
                height: TEXTURE_DIMS.1 as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
    let storage_texture_view = storage_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let output_staging_buffer = compute_engine
        .device()
        .create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of_val(&texture_data[..]) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

    let bind_group_layout =
        compute_engine
            .device()
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                }],
            });
    let bind_group = compute_engine
        .device()
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&storage_texture_view),
            }],
        });

    let pipeline_layout =
        compute_engine
            .device()
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });
    let pipeline =
        compute_engine
            .device()
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: "main",
            });

    info!("Wgpu context set up.");
    //----------------------------------------

    let mut command_encoder = compute_engine
        .device()
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut compute_pass = command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        compute_pass.set_bind_group(0, &bind_group, &[]);
        compute_pass.set_pipeline(&pipeline);
        compute_pass.dispatch_workgroups(TEXTURE_DIMS.0 as u32, TEXTURE_DIMS.1 as u32, 1);
    }
    command_encoder.copy_texture_to_buffer(
        wgpu::ImageCopyTexture {
            texture: &storage_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::ImageCopyBuffer {
            buffer: &output_staging_buffer,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                // This needs to be padded to 256.
                bytes_per_row: Some((TEXTURE_DIMS.0 * 4) as u32),
                rows_per_image: Some(TEXTURE_DIMS.1 as u32),
            },
        },
        wgpu::Extent3d {
            width: TEXTURE_DIMS.0 as u32,
            height: TEXTURE_DIMS.1 as u32,
            depth_or_array_layers: 1,
        },
    );
    compute_engine
        .queue()
        .submit(Some(command_encoder.finish()));

    let buffer_slice = output_staging_buffer.slice(..);
    let (sender, receiver) = flume::bounded(1);
    buffer_slice.map_async(wgpu::MapMode::Read, move |r| sender.send(r).unwrap());
    compute_engine
        .device()
        .poll(wgpu::Maintain::wait())
        .panic_on_timeout();
    receiver.recv_async().await.unwrap().unwrap();
    info!("Output buffer mapped");
    {
        let view = buffer_slice.get_mapped_range();
        texture_data.copy_from_slice(&view[..]);
    }
    info!("GPU data copied to local.");
    output_staging_buffer.unmap();

    output_image_native(
        texture_data.to_vec(),
        TEXTURE_DIMS,
        String::from("mandelbrot.png"),
    );
}

pub fn output_image_native(image_data: Vec<u8>, texture_dims: (usize, usize), path: String) {
    let mut png_data = Vec::<u8>::with_capacity(image_data.len());
    let mut encoder = png::Encoder::new(
        std::io::Cursor::new(&mut png_data),
        texture_dims.0 as u32,
        texture_dims.1 as u32,
    );
    encoder.set_color(png::ColorType::Rgba);
    let mut png_writer = encoder.write_header().unwrap();
    png_writer.write_image_data(&image_data[..]).unwrap();
    png_writer.finish().unwrap();
    info!("PNG file encoded in memory.");

    let mut file = std::fs::File::create(&path).unwrap();
    file.write_all(&png_data[..]).unwrap();
    info!("PNG file written to disc as \"{}\".", path);

    info!("Done! Check the 'mandelbrot.png' file!");
}
