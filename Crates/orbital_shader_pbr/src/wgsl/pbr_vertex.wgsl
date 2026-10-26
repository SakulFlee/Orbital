// Shared PBR vertex entrypoint for forward and G-buffer variants.

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
