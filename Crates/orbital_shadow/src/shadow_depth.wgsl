struct VertexData {
    @location(0) position: vec3<f32>,
}

struct InstanceData {
    @location(5) model_space_matrix_0: vec4<f32>,
    @location(6) model_space_matrix_1: vec4<f32>,
    @location(7) model_space_matrix_2: vec4<f32>,
    @location(8) model_space_matrix_3: vec4<f32>,
}

@group(0) @binding(0) var<uniform> light_view_proj: mat4x4<f32>;

@vertex
fn entrypoint_vertex(
    vertex: VertexData,
    instance: InstanceData,
) -> @builtin(position) vec4<f32> {
    let model_matrix = mat4x4<f32>(
        instance.model_space_matrix_0,
        instance.model_space_matrix_1,
        instance.model_space_matrix_2,
        instance.model_space_matrix_3,
    );
    let world_position = model_matrix * vec4<f32>(vertex.position, 1.0);
    return light_view_proj * world_position;
}
