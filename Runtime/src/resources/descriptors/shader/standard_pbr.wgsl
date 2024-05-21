struct VertexData {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) position_coordinates: vec3<f32>,
    @location(1) texture_coordinates: vec2<f32>,
}

struct FragmentData {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) texture_coordinates: vec2<f32>,
}

@group(0) @binding(0) var albedo_texture: texture_2d<f32>;
@group(0) @binding(1) var albedo_sampler: sampler;

@vertex
fn entrypoint_vertex(in: VertexData) -> FragmentData {
    var out: FragmentData;

    out.clip_position = vec4<f32>(in.position_coordinates, 1.0);
    out.texture_coordinates = in.texture_coordinates;

    return out;
}

@fragment
fn entrypoint_fragment(in: FragmentData) -> @location(0) vec4<f32> {
    return textureSample(albedo_texture, albedo_sampler, in.texture_coordinates);
}
