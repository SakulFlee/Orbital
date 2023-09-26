// --- Structures ---

struct VertexPoint {
    @location(0) position_coordinates: vec3<f32>,
    @location(1) texture_coordinates: vec2<f32>,
    @location(2) normal_coordinates: vec3<f32>,
}

struct InstanceUniform {
    @location(5) model_space_matrix_0: vec4<f32>,
    @location(6) model_space_matrix_1: vec4<f32>,
    @location(7) model_space_matrix_2: vec4<f32>,
    @location(8) model_space_matrix_3: vec4<f32>,
}

struct CameraUniform {
    view_projection_matrix: mat4x4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) texture_coordinates: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec4<f32>,
};

struct AmbientLight {
    color: vec3<f32>,
    strength: f32,
};

// --- Bindings ---

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;

@group(0) @binding(1)
var s_diffuse: sampler;

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@group(2) @binding(0) 
var<uniform> ambient_light: AmbientLight;

// --- Vertex ---

@vertex
fn vs_main(
    vertex_point: VertexPoint,
    instance: InstanceUniform,
) -> VertexOutput {
    let model_space_matrix = mat4x4<f32>(
        instance.model_space_matrix_0,
        instance.model_space_matrix_1,
        instance.model_space_matrix_2,
        instance.model_space_matrix_3,
    );

    var out: VertexOutput;

    // Pass Texture Coordinates along
    out.texture_coordinates = vertex_point.texture_coordinates;

    // Pass Normals along
    out.world_normal = vertex_point.normal_coordinates;

    // Calculate world position
    var world_position: vec4<f32> = model_space_matrix * vec4<f32>(vertex_point.position_coordinates, 1.0);
    out.world_position = world_position;

    // Calculate clip position
    var clip_position = camera.view_projection_matrix * world_position;
    out.clip_position = clip_position;

    return out;
}

// --- Fragment ---

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Get texel from texture
    let texture = textureSample(t_diffuse, s_diffuse, in.texture_coordinates);

    // Ambient Light
    let ambient_color = ambient_light.color * ambient_light.strength;

    // Combine light and colors
    let result = ambient_color * texture.xyz;

    return vec4<f32>(result, texture.a);
}
