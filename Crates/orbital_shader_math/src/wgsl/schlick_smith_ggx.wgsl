fn schlick_smith_ggx(NdotL: f32, NdotV: f32, roughness: f32) -> f32 {
    let k = (roughness * roughness) / 2.0;
    let GL = NdotL / (NdotL * (1.0 - k) + k);
    let GV = NdotV / (NdotV * (1.0 - k) + k);
    return GL * GV;
}
