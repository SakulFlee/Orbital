// Shared PBR bindings for forward, G-buffer and shadow variants.

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(0) @binding(1) var<storage> light_store: array<Light>;
@group(0) @binding(2) var diffuse_env_map: texture_cube<f32>;
@group(0) @binding(3) var diffuse_sampler: sampler;
@group(0) @binding(4) var specular_env_map: texture_cube<f32>;
@group(0) @binding(5) var specular_sampler: sampler;
@group(0) @binding(6) var ibl_brdf_lut_texture: texture_2d<f32>;
@group(0) @binding(7) var ibl_brdf_lut_sampler: sampler;

@group(0) @binding(8) var<uniform> shadow_data: ShadowData;
@group(0) @binding(9) var shadow_map_array: texture_depth_2d_array;
@group(0) @binding(10) var shadow_sampler: sampler_comparison;
@group(0) @binding(11) var point_shadow_maps: texture_depth_cube_array;
@group(0) @binding(12) var point_shadow_sampler: sampler_comparison;
@group(0) @binding(13) var<uniform> light_count: u32;

@group(1) @binding(0) var normal_texture: texture_2d<f32>;
@group(1) @binding(1) var normal_sampler: sampler;
@group(1) @binding(2) var albedo_texture: texture_2d<f32>;
@group(1) @binding(3) var albedo_sampler: sampler;
@group(1) @binding(4) var metallic_texture: texture_2d<f32>;
@group(1) @binding(5) var metallic_sampler: sampler;
@group(1) @binding(6) var roughness_texture: texture_2d<f32>;
@group(1) @binding(7) var roughness_sampler: sampler;
@group(1) @binding(8) var occlusion_texture: texture_2d<f32>;
@group(1) @binding(9) var occlusion_sampler: sampler;
@group(1) @binding(10) var emissive_texture: texture_2d<f32>;
@group(1) @binding(11) var emissive_sampler: sampler;
@group(1) @binding(12) var<uniform> pbr_factors: PBRFactors;
