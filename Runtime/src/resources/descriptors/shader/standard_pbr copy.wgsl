struct VertexData {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) position_coordinates: vec3<f32>,
    @location(1) texture_coordinates: vec2<f32>,
}

struct FragmentData {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) vertex_color: vec3<f32>,
    @location(1) texture_coordinates: vec2<f32>,
}

@group(0) @binding(0) var albedo_texture: texture_2d<f32>;
@group(0) @binding(1) var albedo_sampler: sampler;

@vertex
fn entrypoint_vertex(in: VertexData) -> FragmentData {
    var out: FragmentData;

    out.clip_position = vec4<f32>(in.position_coordinates, 1.0);
    out.texture_coordinates = in.texture_coordinates;

    switch in.vertex_index % 3 {
        case 0u: {
            out.vertex_color = vec3<f32>(1.0, 0.0, 0.0);
        }
        case 1u: {
            out.vertex_color = vec3<f32>(0.0, 1.0, 0.0);
        }
        default: {
            out.vertex_color = vec3<f32>(0.0, 0.0, 1.0);
        }
    }

    return out;
}

@fragment
fn entrypoint_fragment(in: FragmentData) -> @location(0) vec4<f32> {
    // return vec4<f32>(in.vertex_color, 1.0);
    return textureSample(albedo_texture, albedo_sampler, in.texture_coordinates);
}
