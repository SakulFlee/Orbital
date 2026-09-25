fn slope_scaled_bias(base_bias: f32, n_dot_l: f32) -> f32 {
    return base_bias * (1.0 + 0.5 * (1.0 - clamp(n_dot_l, 0.0, 1.0)));
}
