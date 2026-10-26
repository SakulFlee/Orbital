// PBR forward fragment entrypoint.
// Bindings and the vertex entrypoint come from the node library.

@fragment
fn entrypoint_fragment(in: FragmentData) -> @location(0) vec4<f32> {
    let pbr = pbr_data(in);
    var output = vec3(0.0);

    let view_pos = camera.view_projection_matrix * vec4<f32>(in.world_position, 1.0);
    let view_depth = -view_pos.z;

    var ambient = calculate_ambient_ibl(pbr);
    ambient = max(ambient, vec3(0.005));
    output += ambient;

    let light_reflectance = calculate_light_contribution(pbr, in.world_position, view_depth);
    output += light_reflectance;
    output += pbr.emissive;

    if (pbr.alpha < pbr_factors.alpha_cutoff) {
        discard;
    }

    // Linear HDR. Tonemapping and gamma are applied by the post pass.
    return vec4<f32>(output, pbr.alpha);
}
