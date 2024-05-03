#import vertex_data
#import fragment_data

@vertex
fn entrypoint_vertex(
    in: VertexData
) -> FragmentData {
    var out: FragmentData;

    out.coordinate = vec4<f32>(in.position, 1.0);

    let m = in.vertex_index % 3;
    if m == 0 {
        out.vertex_color = vec3<f32>(1.0, 0.0, 0.0);
    } else if m == 1 {
        out.vertex_color = vec3<f32>(0.0, 1.0, 0.0);
    } else {
        out.vertex_color = vec3<f32>(0.0, 0.0, 1.0);
    }

    return out;
}

@fragment
fn entrypoint_fragment(in: FragmentData) -> @location(0) vec4<f32> {
    return vec4<f32>(in.vertex_color, 1.0);
}
