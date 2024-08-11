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
    out.clip_position = vec4(uv * 4.0 - 1.0, 1.0, 1.0);
    out.frag_position = out.clip_position;
    return out;
}

@fragment
fn entrypoint_fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let view_pos_homogeneous = camera.perspective_projection_invert * in.clip_position;
    let view_ray_direction = view_pos_homogeneous.xyz / view_pos_homogeneous.w;
    var ray_direction = normalize((camera.view_projection_transposed * vec4(view_ray_direction, 0.0)).xyz);

    let sample = textureSample(env_map, env_sampler, ray_direction);

    var color = sample.xyz;
    color = color / (color + vec3<f32>(1.0));
    color = pow(color, vec3<f32>(1.0 / camera.skybox_gamma));

    return vec4<f32>(color, 1.0);
}
