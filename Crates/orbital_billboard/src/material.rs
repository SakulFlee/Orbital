//! Built-in lit/unlit billboard materials.

use std::sync::Arc;

use cgmath::Vector3;
use orbital_material_shader::{
    AlphaMode, MaterialShaderDescriptor, RenderQueue, VertexStageLayout,
};
use orbital_shader_core::VariableType;
use orbital_shader_pbr::PBRMaterialShaderDescriptor;
use orbital_texture::{BufferDescriptor, FilterMode, TextureDescriptor, TextureSize};
use wgpu::{
    BlendState, CompareFunction, SamplerBindingType, TextureDimension, TextureFormat,
    TextureSampleType, TextureUsages, TextureViewDimension,
};

/// `pbr_vertex` provides the standard camera-projected vertex entrypoint.
const UNLIT_NODES: &[&str] = &["pbr_vertex"];

const UNLIT_WGSL: &str = include_str!("billboard_unlit.wgsl");

pub(crate) fn unlit_material(
    texture: TextureDescriptor,
    color: [f32; 4],
    alpha_mode: AlphaMode,
) -> Arc<MaterialShaderDescriptor> {
    let mut params = [0u8; 32];
    for (index, value) in color.iter().enumerate() {
        params[index * 4..index * 4 + 4].copy_from_slice(&value.to_le_bytes());
    }
    let cutoff = alpha_mode.cutoff().unwrap_or(0.0);
    params[16..20].copy_from_slice(&cutoff.to_le_bytes());

    let (blend, depth_write, render_queue) = blend_state(alpha_mode);

    Arc::new(MaterialShaderDescriptor {
        name: Some("Billboard Unlit Material".into()),
        nodes: UNLIT_NODES,
        raw_source: Some(UNLIT_WGSL.into()),
        variables: vec![
            VariableType::Texture {
                descriptor: Box::new(texture),
                sample_type: TextureSampleType::Float { filterable: true },
                sampler_binding_type: SamplerBindingType::Filtering,
            },
            VariableType::Buffer(BufferDescriptor {
                data: params.to_vec(),
                ..Default::default()
            }),
        ],
        vertex_stage_layouts: Some(vec![
            VertexStageLayout::ComplexVertexData,
            VertexStageLayout::InstanceData,
        ]),
        cull_mode: None,
        alpha_mode,
        blend,
        depth_write,
        depth_compare: CompareFunction::Less,
        render_queue,
        cast_shadows: false,
        receive_shadows: false,
        ..MaterialShaderDescriptor::default()
    })
}

pub(crate) fn lit_material(
    texture: TextureDescriptor,
    color: [f32; 4],
    alpha_mode: AlphaMode,
) -> Arc<MaterialShaderDescriptor> {
    let pbr = PBRMaterialShaderDescriptor {
        name: Some("Billboard Lit Material".into()),
        normal: flat_normal_texture(),
        albedo: texture,
        albedo_factor: Vector3::new(color[0], color[1], color[2]),
        metallic: white_r8_texture(),
        metallic_factor: 0.0,
        roughness: white_r8_texture(),
        roughness_factor: 0.8,
        occlusion: white_r8_texture(),
        emissive: black_r8_texture(),
        alpha_mode,
        alpha_factor: color[3],
        ..PBRMaterialShaderDescriptor::default()
    };

    let mut descriptor: MaterialShaderDescriptor = pbr.into();
    descriptor.cull_mode = None;
    descriptor.cast_shadows = false;
    descriptor.receive_shadows = false;
    Arc::new(descriptor)
}

fn blend_state(alpha_mode: AlphaMode) -> (BlendState, bool, RenderQueue) {
    if alpha_mode == AlphaMode::Blend {
        (BlendState::ALPHA_BLENDING, false, RenderQueue::Transparent)
    } else {
        (BlendState::REPLACE, true, RenderQueue::Opaque)
    }
}

fn flat_normal_texture() -> TextureDescriptor {
    TextureDescriptor::Data {
        pixels: vec![128, 128, 255, 255],
        size: TextureSize {
            width: 1,
            height: 1,
            ..Default::default()
        },
        usages: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        format: TextureFormat::Rgba8Unorm,
        texture_dimension: TextureDimension::D2,
        texture_view_dimension: TextureViewDimension::D2,
        filter_mode: FilterMode::linear(),
    }
}

fn white_r8_texture() -> TextureDescriptor {
    TextureDescriptor::Data {
        pixels: vec![255],
        size: TextureSize {
            width: 1,
            height: 1,
            ..Default::default()
        },
        usages: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        format: TextureFormat::R8Unorm,
        texture_dimension: TextureDimension::D2,
        texture_view_dimension: TextureViewDimension::D2,
        filter_mode: FilterMode::linear(),
    }
}

fn black_r8_texture() -> TextureDescriptor {
    TextureDescriptor::Data {
        pixels: vec![0],
        size: TextureSize {
            width: 1,
            height: 1,
            ..Default::default()
        },
        usages: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        format: TextureFormat::R8Unorm,
        texture_dimension: TextureDimension::D2,
        texture_view_dimension: TextureViewDimension::D2,
        filter_mode: FilterMode::linear(),
    }
}
