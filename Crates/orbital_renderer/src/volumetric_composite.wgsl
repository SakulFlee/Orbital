// Fullscreen volumetric composite.
//
// Integrates the froxel grid along the view ray in world/view distance,
// multiplies the scene color by the accumulated transmittance, and adds the
// accumulated in-scattering.

struct CompositeParams {
    steps: u32,
    max_distance: f32,
    _padding: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var scene_color: texture_2d<f32>;
@group(1) @binding(1) var scene_depth: texture_depth_2d;
@group(1) @binding(2) var froxels: texture_3d<f32>;
@group(1) @binding(3) var froxel_sampler: sampler;
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
    let steps = max(params.steps, 1u);

    let scene_view_position = volumetric_view_position(in.uv, depth);
    let scene_view_depth = max(-scene_view_position.z, 0.0);
    let t_max = clamp(scene_view_depth / params.max_distance, 0.0, 1.0);
    let step_distance = t_max * params.max_distance / f32(steps);

    var transmittance = 1.0;
    var scattering = vec3<f32>(0.0);

    for (var i = 0u; i < steps; i += 1u) {
        let t = (f32(i) + 0.5) * (t_max / f32(steps));
        let sample = textureSampleLevel(froxels, froxel_sampler, vec3<f32>(in.uv, t), 0.0);
        scattering += transmittance * sample.rgb * step_distance;
        transmittance *= exp(-sample.a * step_distance);
    }

    return vec4<f32>(scene.rgb * transmittance + scattering, 1.0);
}
