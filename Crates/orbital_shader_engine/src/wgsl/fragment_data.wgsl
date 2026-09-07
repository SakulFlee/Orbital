struct FragmentData {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) tangent: vec3<f32>,
    @location(3) bitangent: vec3<f32>,
    @location(4) normal: vec3<f32>,
}
