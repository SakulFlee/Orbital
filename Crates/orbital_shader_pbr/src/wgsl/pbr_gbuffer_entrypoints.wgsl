// PBR G-buffer fragment entrypoint.
// Bindings and the vertex entrypoint come from the node library.

struct GbufferOutput {
    @location(0) albedo_ao: vec4<f32>,
    @location(1) normal_roughness: vec4<f32>,
    @location(2) metallic_emissive: vec4<f32>,
}

@fragment
fn entrypoint_fragment_gbuffer(in: FragmentData) -> GbufferOutput {
    let surface_data = surface(in);

    if (surface_data.alpha < pbr_factors.alpha_cutoff) {
        discard;
    }

    var out: GbufferOutput;
    out.albedo_ao = vec4<f32>(surface_data.albedo, surface_data.occlusion);
    out.normal_roughness = vec4<f32>(surface_data.normal, surface_data.roughness);
    out.metallic_emissive = vec4<f32>(surface_data.metallic, surface_data.emissive);
    return out;
}
