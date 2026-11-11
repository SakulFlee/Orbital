// Fullscreen volumetric composite.
//
// The froxel column has already been integrated into cumulative in-scattering
// and transmittance. Sample that prefix at the scene depth, multiply the scene
// colour by the remaining transmittance, and add the in-scattering.

struct CompositeParams {
    range: vec4<f32>,       // x = max integration distance
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var scene_color: texture_2d<f32>;
@group(1) @binding(1) var scene_depth: texture_depth_2d;
@group(1) @binding(2) var integrated: texture_3d<f32>;
@group(1) @binding(3) var integrated_sampler: sampler;
@group(1) @binding(4) var<uniform> params: CompositeParams;

fn volumetric_view_position(uv: vec2<f32>, ndc_z: f32) -> vec3<f32> {
    let position_h = camera.perspective_projection_invert
        * vec4<f32>(uv * 2.0 - 1.0, ndc_z, 1.0);
    return position_h.xyz / position_h.w;
}

@vertex
fn vs_main(@builtin(vertex_index) id: u32) -> VertexOutput {
    let uv = vec2<f32>(
        f32((id << 1u) & 2u),
        f32(id & 2u),
    );

    var out: VertexOutput;
    out.position = vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
    out.uv = uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let pixel = vec2<i32>(in.position.xy);
    let scene = textureLoad(scene_color, pixel, 0);
    let depth = textureLoad(scene_depth, pixel, 0);

    let scene_view_position = volumetric_view_position(in.uv, depth);
    let scene_view_depth = max(-scene_view_position.z, 0.0);
    let t_max = clamp(scene_view_depth / params.range.x, 0.0, 1.0);
    let prefix = textureSampleLevel(
        integrated,
        integrated_sampler,
        vec3<f32>(in.uv, t_max),
        0.0,
    );

    return vec4<f32>(scene.rgb * prefix.a + prefix.rgb, 1.0);
}
