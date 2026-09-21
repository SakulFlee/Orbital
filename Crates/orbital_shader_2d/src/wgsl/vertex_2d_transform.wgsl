// 2D vertex transform with camera uniform
struct CameraUniform {
    view_proj: mat4x4<f32>,
    screen_size: vec2<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;

// Transform 2D vertex position to clip space using camera matrix
fn vertex_2d_transform(
    position: vec2<f32>,
    color: vec4<f32>,
    texcoord: vec2<f32>,
    shape_params: vec2<f32>,
) -> Vertex2DOutput {
    var output: Vertex2DOutput;
    output.clip_position = camera.view_proj * vec4<f32>(position, 0.0, 1.0);
    output.color = color;
    output.texcoord = texcoord;
    output.shape_params = shape_params;
    return output;
}
