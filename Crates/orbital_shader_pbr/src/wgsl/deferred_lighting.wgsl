// Deferred lighting: reconstruct PBR data from the G-buffer, then run the
// same IBL and punctual-light functions used by the forward path.

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

@group(1) @binding(0) var gbuffer_albedo: texture_2d<f32>;
@group(1) @binding(1) var gbuffer_normal: texture_2d<f32>;
@group(1) @binding(2) var gbuffer_metallic: texture_2d<f32>;
@group(1) @binding(3) var depth_texture: texture_depth_2d;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) id: u32) -> VertexOutput {
    let uv = vec2<f32>(
        f32((id << 1u) & 2u),
        f32(id & 2u),
    );

    var out: VertexOutput;
    out.position = vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
    // Keep the unflipped coordinate: it maps directly to NDC for world-space
    // reconstruction, while the G-buffer is read with integer pixel coords.
    out.uv = uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let pixel = vec2<i32>(in.position.xy);
    let depth = textureLoad(depth_texture, pixel, 0);
    if (depth >= 1.0) {
        discard;
    }

    let albedo_ao = textureLoad(gbuffer_albedo, pixel, 0);
    let normal_roughness = textureLoad(gbuffer_normal, pixel, 0);
    let metallic_emissive = textureLoad(gbuffer_metallic, pixel, 0);

    let ndc = vec4<f32>(in.uv * 2.0 - 1.0, depth, 1.0);
    let view_position_h = camera.perspective_projection_invert * ndc;
    let view_position = view_position_h.xyz / view_position_h.w;
    let world_offset = (camera.view_projection_transposed * vec4<f32>(view_position, 0.0)).xyz;
    let world_position = camera.position.xyz + world_offset;

    var pbr: PBRData;
    pbr.albedo = albedo_ao.rgb;
    pbr.alpha = 1.0;
    pbr.N = normalize(normal_roughness.xyz);
    pbr.roughness = normal_roughness.w;
    pbr.metallic = metallic_emissive.x;
    pbr.emissive = metallic_emissive.yzw;
    pbr.occlusion = albedo_ao.a;
    pbr.V = normalize(camera.position.xyz - world_position);
    pbr.NdotV = clamp(dot(pbr.N, pbr.V), 0.0, 1.0);

    let reflection = normalize(reflect(-pbr.V, pbr.N));
    pbr.ibl_diffuse = textureSample(
        diffuse_env_map,
        diffuse_sampler,
        pbr.N
    ).rgb;

    let specular_mip_count = textureNumLevels(specular_env_map);
    let specular_mip_level = pbr.roughness * pbr.roughness * f32(specular_mip_count - 1u);
    pbr.ibl_specular = textureSampleLevel(
        specular_env_map,
        specular_sampler,
        reflection,
        specular_mip_level
    ).rgb;

    pbr.brdf_lut = textureSample(
        ibl_brdf_lut_texture,
        ibl_brdf_lut_sampler,
        vec2<f32>(
            max(pbr.NdotV, 0.0001),
            clamp(pbr.roughness, 0.0001, 1.0)
        )).rg;

    let view_pos = camera.view_projection_matrix * vec4<f32>(world_position, 1.0);
    let view_depth = -view_pos.z;

    var output = calculate_ambient_ibl(pbr);
    output = max(output, vec3<f32>(0.005));
    output += calculate_light_contribution(pbr, world_position, view_depth);
    output += pbr.emissive;

    return vec4<f32>(output, 1.0);
}
