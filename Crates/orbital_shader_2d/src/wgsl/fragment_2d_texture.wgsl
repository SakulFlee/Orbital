// Fragment shader for textured 2D shapes
@group(1) @binding(0) var base_texture: texture_2d<f32>;
@group(1) @binding(1) var base_sampler: sampler;

fn fragment_2d_texture(input: Vertex2DOutput) -> @location(0) vec4<f32> {
    let texture_color = textureSample(base_texture, base_sampler, input.texcoord);
    return texture_color * input.color;
}
