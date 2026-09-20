// Vertex output for 2D rendering
struct Vertex2DOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) texcoord: vec2<f32>,
    @location(2) shape_params: vec2<f32>,
};
