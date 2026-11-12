// Minimal custom material shader.
//
// The engine nodes provide CameraUniform, VertexData, InstanceData and
// FragmentData. This file only supplies the entrypoints and the visual style.
//
// The cube is a standard RGB colour cube: local -X/-Y/-Z maps to black and
// +X/+Y/+Z maps to red/green/blue, with the other corners blending between
// them. Every face therefore shows a clean slice of the RGB distribution.

const HALF_EXTENT: f32 = 0.5;

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
    out.uv = vertex.uv;
    // Local position (-1..1), reused as the per-corner colour coordinate.
    out.tangent = vertex.position / HALF_EXTENT;
    out.bitangent = vertex.bitangent;
    out.normal = normalize(vertex.normal);
    return out;
}

@fragment
fn entrypoint_fragment(in: FragmentData) -> @location(0) vec4<f32> {
    let local = clamp(in.tangent, vec3<f32>(-1.0), vec3<f32>(1.0));
    let corner_rgb = local * 0.5 + vec3<f32>(0.5);

    // Keep the corner colours pure; this small face term only separates the
    // three visible faces so the cube still reads as a solid object.
    let normal = normalize(in.normal);
    let key = normalize(vec3<f32>(0.35, 0.8, 0.45));
    let shade = 0.85 + 0.15 * max(dot(normal, key), 0.0);

    return vec4<f32>(corner_rgb * shade, 1.0);
}
