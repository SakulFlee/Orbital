// ── GPU instance culling ────────────────────────────────────────────────
// Two-pass compute: (1) test each instance against the 6 frustum planes,
// (2) compact visible instances and write indirect draw args.

struct ModelParams {
    first_instance: u32,
    total_count: u32,
    index_count: u32,
    first_index: u32,
    base_vertex: i32,
    _pad: u32,
};

struct DrawIndexedIndirect {
    index_count: u32,
    instance_count: u32,
    first_index: u32,
    base_vertex: i32,
    first_instance: u32,
};

@group(0) @binding(0) var<uniform> frustum_planes: array<vec4<f32>, 6>;
@group(0) @binding(1) var<storage, read> params: array<ModelParams>;
@group(0) @binding(2) var<storage, read> in_instances: array<mat4x4<f32>>;
@group(0) @binding(3) var<storage, read> in_bounds: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read_write> out_instances: array<mat4x4<f32>>;
@group(0) @binding(5) var<storage, read_write> counters: array<atomic<u32>>;
@group(0) @binding(6) var<storage, read_write> indirect: array<DrawIndexedIndirect>;

// ── Pass 1: frustum test + compaction ───────────────────────────────────

@compute @workgroup_size(64)
fn cull(@builtin(global_invocation_id) id: vec3<u32>) {
    let model = id.y;
    let local_idx = id.x;
    let p = params[model];
    if local_idx >= p.total_count { return; }

    let global_idx = p.first_instance + local_idx;
    let bounds = in_bounds[global_idx];

    for (var i = 0u; i < 6u; i++) {
        let plane = frustum_planes[i];
        let dist = dot(plane.xyz, bounds.xyz) + plane.w;
        if dist <= -bounds.w { return; }  // fully outside
    }

    // Visible — compact into output with atomic counter.
    let slot = atomicAdd(&counters[model], 1u);
    out_instances[p.first_instance + slot] = in_instances[global_idx];
}

// ── Pass 2: write indirect draw args + reset counters ───────────────────

@compute @workgroup_size(1)
fn finalize(@builtin(global_invocation_id) id: vec3<u32>) {
    let model = id.x;
    let p = params[model];
    let count = atomicExchange(&counters[model], 0u);

    indirect[model] = DrawIndexedIndirect(
        p.index_count,
        count,
        p.first_index,
        p.base_vertex,
        0u,  // first_instance — compacted output starts at model's offset
    );
}
