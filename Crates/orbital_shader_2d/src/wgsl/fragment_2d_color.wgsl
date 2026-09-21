// Fragment shader for solid color 2D shapes
fn fragment_2d_color(input: Vertex2DOutput) -> @location(0) vec4<f32> {
    return input.color;
}
