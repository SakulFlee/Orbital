struct CameraUniform {
    position: vec3<f32>,
    view_projection_matrix: mat4x4<f32>,
    perspective_view_projection_matrix: mat4x4<f32>,
    view_projection_transposed: mat4x4<f32>,
    perspective_projection_invert: mat4x4<f32>,
    global_gamma: f32,
    skybox_gamma: f32,
}

struct VertexOutput {
    @builtin(position) frag_position: vec4<f32>,
    // idk why this split is needed but these are actually two 
    // different variables!! DO NOT REMOVE!!!
    @location(0) clip_position: vec4<f32>,
}

struct Info {
    lod: i32,
}

@group(0) @binding(0) var diffuse_env_map: texture_cube<f32>;
@group(0) @binding(1) var diffuse_env_sampler: sampler;

@group(0) @binding(2) var specular_env_map: texture_cube<f32>;
@group(0) @binding(3) var specular_env_sampler: sampler;

// @group(0) @binding(4) var ibl_brdf_env_map: texture_cube<f32>;
// @group(0) @binding(5) var ibl_brdf_env_sampler: sampler;

@group(0) @binding(6) var<uniform> info: Info;

@group(1) @binding(0) var<uniform> camera: CameraUniform;

@vertex
fn entrypoint_vertex(
    @builtin(vertex_index) id: u32,
) -> VertexOutput {
    let uv = vec2<f32>(vec2<u32>(
        id & 1u,
        (id >> 1u) & 1u,
    ));

    var out: VertexOutput;
    out.clip_position = vec4(uv * 4.0 - 1.0, 1.0, 1.0);
    out.frag_position = out.clip_position;
    return out;
}

@fragment
fn entrypoint_fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Precalculations
    let view_position = camera.perspective_projection_invert * in.clip_position;
    let view_ray_direction = view_position.xyz / view_position.w;
    var ray_direction = normalize((camera.view_projection_transposed * vec4(view_ray_direction, 0.0)).xyz);

    // HDRI SkyBox
    var sample: vec4<f32>;
    if info.lod < 0 {
        sample = textureSample(diffuse_env_map, diffuse_env_sampler, ray_direction);
    } else {
        sample = textureSampleLevel(specular_env_map, specular_env_sampler, ray_direction, f32(info.lod));
    }

    var color = sample.xyz;
    color = color / (color + vec3<f32>(1.0));
    color = pow(color, vec3<f32>(1.0 / camera.skybox_gamma));

    // Generated SkyBox:
    // let sky_color = vec3<f32>(0.0, 0.75, 1.0);
    // let horizon_color = vec3<f32>(0.5, 0.5, 0.5);
    // let color = mix(horizon_color, sky_color, ray_direction.y);

    return vec4<f32>(color, 1.0);
}
