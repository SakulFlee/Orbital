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

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) texture_coordinates: vec2<f32>,
};

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;

@group(0) @binding(1)
var s_diffuse: sampler;

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

    out.clip_position = model_space_matrix * vec4<f32>(vertex_point.position_coordinates, 1.0);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.texture_coordinates);
}
