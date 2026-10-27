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
    let t_max = clamp(depth, 0.0, 1.0);

    var transmittance = 1.0;
    var scattering = vec3<f32>(0.0);
    var previous_distance = 0.0;
    let step_size = t_max / f32(steps);

    for (var i = 0u; i < steps; i += 1u) {
        let t = (f32(i) + 0.5) * step_size;
        let ndc = vec4<f32>(in.uv * 2.0 - 1.0, t, 1.0);
        let view_position_h = camera.perspective_projection_invert * ndc;
        let view_position = view_position_h.xyz / view_position_h.w;
        let sample_distance = -view_position.z;
        let step_distance = max(sample_distance - previous_distance, 0.0);
        previous_distance = sample_distance;

        let sample = textureSampleLevel(froxels, froxel_sampler, vec3<f32>(in.uv, t), 0.0);
        scattering += transmittance * sample.rgb * step_distance;
        transmittance *= exp(-sample.a * step_distance);
    }

    return vec4<f32>(scene.rgb * transmittance + scattering, 1.0);
}
