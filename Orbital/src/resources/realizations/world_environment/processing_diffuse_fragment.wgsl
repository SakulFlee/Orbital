struct FragmentData {
    @builtin(position) position: vec4<f32>,
}

@group(0) @binding(0) var src_env_map: texture_cube<f32>;
@group(0) @binding(1) var src_sampler: sampler;

@fragment
fn entrypoint_fragment(in: FragmentData) -> @location(0) vec4<f32> {
    return vec4<f32>(0.7, 0.8, 0.9, 1.0);
}
