// PBR shadow-caster entrypoints.
//
// Group 0 is the material bind group and group 1 is the per-light view
// projection matrix. The vertex stage only needs position/UV, but the buffer
// layout is shared with the forward and G-buffer variants.

struct VertexData {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tangent: vec3<f32>,
    @location(3) bitangent: vec3<f32>,
    @location(4) uv: vec2<f32>,
}

struct InstanceData {
    @location(5) model_space_matrix_0: vec4<f32>,
    @location(6) model_space_matrix_1: vec4<f32>,
    @location(7) model_space_matrix_2: vec4<f32>,
    @location(8) model_space_matrix_3: vec4<f32>,
}

struct PBRFactors {
    albedo_factor: vec3<f32>,
    metallic_factor: f32,
    roughness_factor: f32,
    alpha_factor: f32,
    alpha_cutoff: f32,
}

struct ShadowVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@group(0) @binding(2) var albedo_texture: texture_2d<f32>;
@group(0) @binding(3) var albedo_sampler: sampler;
@group(0) @binding(12) var<uniform> pbr_factors: PBRFactors;
@group(1) @binding(0) var<uniform> shadow_view_projection: mat4x4<f32>;

@vertex
fn entrypoint_vertex_shadow(
    vertex: VertexData,
    instance: InstanceData,
) -> ShadowVertexOutput {
    let model_space_matrix = mat4x4<f32>(
        instance.model_space_matrix_0,
        instance.model_space_matrix_1,
        instance.model_space_matrix_2,
        instance.model_space_matrix_3,
    );
    let world_position = model_space_matrix * vec4<f32>(vertex.position, 1.0);

    var out: ShadowVertexOutput;
    out.position = shadow_view_projection * world_position;
    out.uv = vertex.uv;
    return out;
}

@fragment
fn entrypoint_fragment_shadow(in: ShadowVertexOutput) {
    // Opaque materials have a zero cutoff and skip the texture fetch.
    if (pbr_factors.alpha_cutoff > 0.0) {
        let alpha = textureSampleLevel(
            albedo_texture,
            albedo_sampler,
            in.uv,
            0.0,
        ).a * pbr_factors.alpha_factor;
        if (alpha < pbr_factors.alpha_cutoff) {
            discard;
        }
    }
}
