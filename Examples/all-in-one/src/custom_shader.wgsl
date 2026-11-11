// Custom material shader for the R8 exhibit.
//
// Engine nodes provide CameraUniform, VertexData, InstanceData and
// FragmentData. The cube is coloured by its local vertex position: the
// -X/-Y/-Z corner is dark, while +X, +Y and +Z tend towards red, green and
// blue. Every face therefore shows the full RGB distribution. Per-face
// shading and a view-dependent rim keep the unlit exhibit readable without
// going through the PBR lighting path.

const HALF_EXTENT: f32 = 1.2;

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
    // Local position (-1..1), reused as the per-corner colour coordinate.
    out.tangent = vertex.position / HALF_EXTENT;
    out.bitangent = vec3<f32>(0.0, 1.0, 0.0);
    out.normal = vec3<f32>(0.0, 1.0, 0.0);
    return out;
}

@fragment
fn entrypoint_fragment(in: FragmentData) -> @location(0) vec4<f32> {
    // Recover the face normal from the interpolated world position.
    var normal = normalize(cross(dpdx(in.world_position), dpdy(in.world_position)));
    let to_camera = normalize(camera.position - in.world_position);
    if (dot(normal, to_camera) < 0.0) {
        normal = -normal;
    }

    let local = clamp(in.tangent, vec3<f32>(-1.0), vec3<f32>(1.0));
    let corner_rgb = local * 0.45 + vec3<f32>(0.5);

    // Fixed key light so the unlit cube still reads as a solid object.
    let key = normalize(vec3<f32>(0.35, 0.8, 0.45));
    let shade = 0.55 + 0.45 * max(dot(normal, key), 0.0);

    // Hologram bands react to the view direction, so moving the camera makes
    // the pattern flow without needing a time uniform in the shader.
    let bands = 0.5 + 0.5 * sin(in.world_position.y * 9.0 + dot(in.world_position, to_camera) * 2.0);
    let rim = pow(1.0 - clamp(dot(normal, to_camera), 0.0, 1.0), 3.0);

    var color = corner_rgb * shade;
    color += vec3<f32>(0.08, 0.22, 0.35) * rim * (0.6 + 0.4 * bands);
    return vec4<f32>(color, 1.0);
}
