struct CameraUniform {
    position: vec3<f32>,
    view_projection_matrix: mat4x4<f32>,
    perspective_view_projection_matrix: mat4x4<f32>,
    view_projection_transposed: mat4x4<f32>,
    perspective_projection_invert: mat4x4<f32>,
    global_gamma: f32,
}

struct BoundingBox {
    min: vec3<f32>,
    max: vec3<f32>,
}

struct DrawIndexedIndirect {
    index_count: u32,
    instance_count: u32,
    first_index: u32,
    base_vertex: u32,
    first_instance: u32,
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(0) @binding(1) var<uniform> bounding_box: BoundingBox;
@group(0) @binding(2) var<storage, read_write> draw_indexed_indirect: DrawIndexedIndirect;

fn null_draw_indexed_indirect() {
    draw_indexed_indirect.index_count = 0u;
    draw_indexed_indirect.instance_count = 0u;
    draw_indexed_indirect.first_index = 0u;
    draw_indexed_indirect.base_vertex = 0u;
    draw_indexed_indirect.first_instance = 0u;
}

@compute
@workgroup_size(1, 1, 1)
fn main(
    @builtin(global_invocation_id) gid : vec3<u32>,
) {
    // Calculate frustum planes in clip space
    let vp = camera.view_projection_matrix;
    var frustum_planes = array<vec4<f32>, 6>(
        normalize(-vp[0]),          // Left plane
        normalize( vp[0]),          // Right plane
        normalize(-vp[1]),          // Bottom plane
        normalize( vp[1]),          // Top plane
        normalize( vp[2]),          // Near plane
        normalize(-vp[2] + vp[3])   // Far plane
    );

    // Frustum culling logic
    let min = bounding_box.min;
    let max = bounding_box.max;

    for (var i : u32 = 0; i < 6u; i++) {
        let plane = frustum_planes[i];
        let radius = abs(plane.x * min.x) + abs(plane.y * min.y) + abs(plane.z * min.z);
        if ((plane.x * max.x + plane.y * max.y + plane.z * max.z) < -radius) {
            // Bounding box is completely outside the frustum
            return;
        }
    }

    // Update indirect draw buffer to indicate drawing
    null_draw_indexed_indirect();
}
