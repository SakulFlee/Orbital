struct BoundingBox {
    min: vec3<f32>,
    max: vec3<f32>,
}

struct IndirectIndexedDraw {
    index_count: u32,
    instance_count: u32,
    first_index: u32,
    base_vertex: u32,
    first_instance: u32,
}

@group(0) @binding(0) var<uniform> frustum: array<vec4<f32>, 6>;
@group(0) @binding(1) var<storage, read> bounding_box: array<BoundingBox>;
@group(0) @binding(2) var<storage, read_write> indirect_indexed_draw: array<IndirectIndexedDraw>;

fn null_indirect_indexed_draw(index: u32) {
    indirect_indexed_draw[index].index_count = 0u;
    indirect_indexed_draw[index].instance_count = 0u;
}

@compute
@workgroup_size(16, 1, 1)
fn main(
    @builtin(global_invocation_id) gid : vec3<u32>,
) {
    for (var i: u32 = 0u; i < 6u; i++) {
        var plane = frustum[i];

        let dot_min = dot(bounding_box[gid.x].min, plane.xyz) + plane.w;
        let dot_max = dot(bounding_box[gid.x].max, plane.xyz) + plane.w;

        if (dot_min < 0.0 && dot_max < 0.0) {
            null_indirect_indexed_draw(gid.x);
            break;
        }
    }
}
