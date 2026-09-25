fn distribution_ggx(NdotH: f32, roughness: f32) -> f32 {
    let alpha = roughness * roughness;
    let alpha_squared = alpha * alpha;
    let denom = (NdotH * NdotH) * (alpha_squared - 1.0) + 1.0;
    return alpha_squared / (PI * denom);
}
