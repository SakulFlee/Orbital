// PBR shader entrypoints + bindings.
// The reusable math/struct/orchestration functions come from the node library.

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

@vertex
fn entrypoint_vertex(
    vertex: VertexData,
    instance: InstanceData
) -> FragmentData {
    let model_space_matrix = mat4x4<f32>(
        instance.model_space_matrix_0,
        instance.model_space_matrix_1,
        instance.model_space_matrix_2,
        instance.model_space_matrix_3,
    );

    let world_position = model_space_matrix * vec4<f32>(vertex.position, 1.0);

    var out: FragmentData;
    out.position = camera.perspective_view_projection_matrix * world_position;
    out.world_position = world_position.xyz;
    out.uv = vertex.uv;
    out.tangent = normalize((model_space_matrix * vec4<f32>(vertex.tangent, 0.0)).xyz);
    out.bitangent = normalize((model_space_matrix * vec4<f32>(vertex.bitangent, 0.0)).xyz);
    out.normal = normalize((model_space_matrix * vec4<f32>(vertex.normal, 0.0)).xyz);
    return out;
}

@fragment
fn entrypoint_fragment(in: FragmentData) -> @location(0) vec4<f32> {
    let pbr = pbr_data(in);
    var output = vec3(0.0);

    let view_pos = camera.view_projection_matrix * vec4<f32>(in.world_position, 1.0);
    let view_depth = -view_pos.z;

    var ambient = calculate_ambient_ibl(pbr);
    ambient = max(ambient, vec3(0.005));
    output += ambient;

    let light_reflectance = calculate_light_contribution(pbr, in.world_position, view_depth);
    output += light_reflectance;
    output += pbr.emissive;

    let tone_mapped_color = aces_tone_map(output);
    return vec4<f32>(tone_mapped_color, 1.0);
}
