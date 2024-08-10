struct CameraUniform {
    view_projection_matrix: mat4x4<f32>,
    position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) frag_position: vec4<f32>,
    @location(0) clip_position: vec4<f32>,
}

@group(0) @binding(0)
var env_map: texture_cube<f32>;
@group(0) @binding(1)
var env_sampler: sampler;

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@vertex
fn entrypoint_vertex(
    @builtin(vertex_index) id: u32,
) -> VertexOutput {
    let uv = vec2<f32>(vec2<u32>(
        id & 1u,
        (id >> 1u) & 1u,
    ));
    var out: VertexOutput;
    // out.clip_position = vec4(uv * vec2(4.0, -4.0) + vec2(-1.0, 1.0), 0.0, 1.0);
    out.clip_position = vec4(uv * 4.0 - 1.0, 1.0, 1.0);
    out.frag_position = vec4(uv * 4.0 - 1.0, 1.0, 1.0);
    return out;
}

@fragment
fn entrypoint_fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // let view_pos_homogeneous = camera.inv_proj * in.clip_position;
    // let view_ray_direction = view_pos_homogeneous.xyz / view_pos_homogeneous.w;
    // var ray_direction = normalize((camera.inv_view * vec4(view_ray_direction, 0.0)).xyz);

    // let sample = textureSample(env_map, env_sampler, ray_direction);
    // return sample;

    return vec4<f32>(0.5, 0.5, 0.5, 1.0);
}
