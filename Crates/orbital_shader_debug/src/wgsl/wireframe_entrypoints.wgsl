// Wireframe shader entrypoints + bindings.
// Reusable structs/layouts come from the engine/math node libraries.

@group(0) @binding(0) var<uniform> camera: CameraUniform;

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
    return out;
}

@fragment
fn entrypoint_fragment(in: FragmentData) -> @location(0) vec4<f32> {
   return vec4<f32>(1.0);
}
