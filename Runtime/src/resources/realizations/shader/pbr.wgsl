#import vertex_data
#import fragment_data

@vertex
fn entrypoint_vertex(
    in: VertexData
) -> FragmentData {
    var out: FragmentData;

    out.clip_position = vec4<f32>(in.position, 1.0);

    return out;
}

@fragment
fn entrypoint_fragment(in: FragmentData) -> @location(0) vec4<f32> {
    //>! UNLIT
    //>! LIT
    //>! SHADOW

    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
