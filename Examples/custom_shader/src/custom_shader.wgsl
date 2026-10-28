// Minimal custom material shader.
//
// The engine nodes provide CameraUniform, VertexData, InstanceData and
// FragmentData. This file only supplies the entrypoints and the visual style.

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
    out.uv = vec2<f32>(0.0);
    out.tangent = vec3<f32>(1.0, 0.0, 0.0);
    out.bitangent = vec3<f32>(0.0, 1.0, 0.0);
    out.normal = vec3<f32>(0.0, 1.0, 0.0);
    return out;
}

@fragment
fn entrypoint_fragment(in: FragmentData) -> @location(0) vec4<f32> {
    let bands = fract(in.world_position.x * 0.35 + in.world_position.y * 0.2);
    let pulse = 0.5 + 0.5 * sin(in.world_position.z * 2.0);
    return vec4<f32>(bands, 1.0 - bands, pulse, 1.0);
}
