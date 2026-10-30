// Temporal accumulation for the view-aligned volumetric froxel grid.
//
// Each invocation reprojects the current froxel centre into the previous
// camera and blends the current froxel with the previous history. Invalid
// history (outside the previous view, resized grid, camera cut) falls back to
// the current sample.

struct AccumParams {
    prev_view: mat4x4<f32>,
    prev_view_projection: mat4x4<f32>,
    grid: vec4<u32>,
    settings: vec4<f32>, // current max distance, previous max distance, blend, valid
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var current_froxels: texture_3d<f32>;
@group(1) @binding(1) var history_froxels: texture_3d<f32>;
@group(1) @binding(2) var history_sampler: sampler;
@group(1) @binding(3) var output_froxels: texture_storage_3d<rgba16float, write>;
@group(1) @binding(4) var<uniform> params: AccumParams;

fn volumetric_view_position(uv: vec2<f32>, ndc_z: f32) -> vec3<f32> {
    let position_h = camera.perspective_projection_invert
        * vec4<f32>(uv * 2.0 - 1.0, ndc_z, 1.0);
    return position_h.xyz / position_h.w;
}

@compute @workgroup_size(4, 4, 4)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let grid = params.grid.xyz;
    if (any(id >= grid)) {
        return;
    }

    let current = textureLoad(current_froxels, vec3<i32>(id), 0);
    let current_max = max(params.settings.x, 0.1);
    let previous_max = max(params.settings.y, 0.1);
    let blend = clamp(params.settings.z, 0.0, 0.99);
    let history_valid = params.settings.w > 0.5;

    let uv = (vec2<f32>(id.xy) + 0.5) / vec2<f32>(grid.xy);
    let slice = (f32(id.z) + 0.5) / f32(grid.z);
    let view_depth = slice * current_max;

    let near_position = volumetric_view_position(uv, 0.0);
    let far_position = volumetric_view_position(uv, 1.0);
    let is_perspective = distance(near_position.xy, far_position.xy) > 1.0e-4;

    var view_position: vec3<f32>;
    if (is_perspective) {
        view_position = normalize(far_position) * view_depth;
    } else {
        view_position = vec3<f32>(near_position.xy, -view_depth);
    }

    let world_offset = (camera.view_projection_transposed * vec4<f32>(view_position, 0.0)).xyz;
    let world_position = camera.position.xyz + world_offset;

    let previous_clip = params.prev_view_projection * vec4<f32>(world_position, 1.0);
    let previous_depth = -(params.prev_view * vec4<f32>(world_position, 1.0)).z;
    let previous_ndc = previous_clip.xyz / max(previous_clip.w, 1.0e-6);
    let previous_uv = previous_ndc.xy * 0.5 + 0.5;
    let previous_slice = previous_depth / previous_max;

    let valid = history_valid
        && previous_clip.w > 0.0
        && all(previous_uv >= vec2<f32>(0.0))
        && all(previous_uv < vec2<f32>(1.0))
        && previous_slice >= 0.0
        && previous_slice <= 1.0;

    var result = current;
    if (valid) {
        let historical = textureSampleLevel(
            history_froxels,
            history_sampler,
            vec3<f32>(previous_uv, previous_slice),
            0.0,
        );
        result = mix(current, historical, blend);
    }

    textureStore(output_froxels, vec3<i32>(id), result);
}
