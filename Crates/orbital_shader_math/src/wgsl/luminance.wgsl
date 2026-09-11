const LUMINANCE_CONVERSION: vec3<f32> = vec3<f32>(0.2126, 0.7152, 0.0722);

fn luminance(color: vec3<f32>) -> f32 {
    return dot(color, LUMINANCE_CONVERSION);
}
