struct VertexData {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) position_coordinates: vec3<f32>,
    @location(1) texture_coordinates: vec2<f32>,
}

struct InstanceData {
    @location(2) model_space_matrix_1: vec4<f32>,
    @location(3) model_space_matrix_2: vec4<f32>,
    @location(4) model_space_matrix_3: vec4<f32>,
    @location(5) model_space_matrix_4: vec4<f32>,
}

struct FragmentData {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) texture_coordinates: vec2<f32>,
}

struct CameraUniform {
    view_projection_matrix: mat4x4<f32>,
}

@group(0) @binding(0) var albedo_texture: texture_2d<f32>;
@group(0) @binding(1) var albedo_sampler: sampler;

@group(1) @binding(0) 
var<uniform> camera: CameraUniform;

@vertex
fn entrypoint_vertex(
    vertex: VertexData,
    instance: InstanceData
) -> FragmentData {
    let model_space_matrix = mat4x4<f32>(
        instance.model_space_matrix_1,
        instance.model_space_matrix_2,
        instance.model_space_matrix_3,
        instance.model_space_matrix_4,
    );

    var out: FragmentData;
    out.clip_position = camera.view_projection_matrix * model_space_matrix * vec4<f32>(vertex.position_coordinates, 1.0);
    out.texture_coordinates = vertex.texture_coordinates;
    return out;
}

@fragment
fn entrypoint_fragment(fragment: FragmentData) -> @location(0) vec4<f32> {
    return textureSample(
        albedo_texture,
        albedo_sampler,
        fragment.texture_coordinates
    );
}
