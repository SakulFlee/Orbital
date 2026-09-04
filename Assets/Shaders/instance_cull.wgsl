// ── GPU instance culling ────────────────────────────────────────────────
// Two-pass compute: (1) test each instance against the 6 frustum planes,
// (2) compact visible instances and write indirect draw args.
//
// NOTE: The Adreno Vulkan driver miscompiles user code that materializes
// whole struct values (`let p = params[model]`) or writes struct-valued
// stores (`indirect[model] = DrawIndexedIndirect(..)`) — the same bug class
// that miscompiled `sky_color(D, params)` and the PBR shader. All buffer
// accesses here therefore use raw `vec4<u32>` / `u32` / per-component
// `vec4<f32>` loads and stores; no struct value ever crosses a register.
//
// Buffer layouts (byte-identical to the former struct versions):
//   params    : 6 × u32 per model → 2 × vec4<u32> per model
//               [0] = (first_instance, total_count, index_count, first_index)
//               [1] = (base_vertex as u32, pad, pad, pad)
//   indirect  : 5 × u32 per model = 20 bytes (DrawIndexedIndirect ABI)
//               (index_count, instance_count, first_index, base_vertex,
//                first_instance)
//   instances : array<vec4<f32>> — 4 per instance (mat4 rows/columns)
//   bounds    : array<vec4<f32>> — 1 per instance (center.xyz, radius)

@group(0) @binding(0) var<uniform> frustum_planes: array<vec4<f32>, 6>;
@group(0) @binding(1) var<storage, read> params: array<vec4<u32>>;
@group(0) @binding(2) var<storage, read> in_instances: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read> in_bounds: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read_write> out_instances: array<vec4<f32>>;
@group(0) @binding(5) var<storage, read_write> counters: array<atomic<u32>>;
@group(0) @binding(6) var<storage, read_write> indirect: array<u32>;

// ── Pass 1: frustum test + compaction ───────────────────────────────────

@compute @workgroup_size(64)
fn cull(@builtin(global_invocation_id) id: vec3<u32>) {
    let model = id.y;
    let local_idx = id.x;
    let p0 = params[model * 2u];
    if local_idx >= p0.y { return; }

    let global_idx = p0.x + local_idx;
    let bounds = in_bounds[global_idx];

    for (var i = 0u; i < 6u; i++) {
        let plane = frustum_planes[i];
        let dist = plane.x * bounds.x + plane.y * bounds.y + plane.z * bounds.z + plane.w;
        if dist <= -bounds.w { return; }  // fully outside
    }

    // Visible — compact into output with atomic counter.
    let slot = atomicAdd(&counters[model], 1u);
    let out_base = (p0.x + slot) * 4u;
    let in_base = global_idx * 4u;
    out_instances[out_base + 0u] = vec4<f32>(
        in_instances[in_base].x, in_instances[in_base].y,
        in_instances[in_base].z, in_instances[in_base].w);
    out_instances[out_base + 1u] = vec4<f32>(
        in_instances[in_base + 1u].x, in_instances[in_base + 1u].y,
        in_instances[in_base + 1u].z, in_instances[in_base + 1u].w);
    out_instances[out_base + 2u] = vec4<f32>(
        in_instances[in_base + 2u].x, in_instances[in_base + 2u].y,
        in_instances[in_base + 2u].z, in_instances[in_base + 2u].w);
    out_instances[out_base + 3u] = vec4<f32>(
        in_instances[in_base + 3u].x, in_instances[in_base + 3u].y,
        in_instances[in_base + 3u].z, in_instances[in_base + 3u].w);
}

// ── Pass 2: write indirect draw args + reset counters ───────────────────

@compute @workgroup_size(1)
fn finalize(@builtin(global_invocation_id) id: vec3<u32>) {
    let model = id.x;
    let p0 = params[model * 2u];
    let p1 = params[model * 2u + 1u];
    let count = atomicExchange(&counters[model], 0u);

    let base = model * 5u;
    // DrawIndexedIndirect ABI: (index_count, instance_count, first_index,
    // base_vertex(i32), first_instance). Compacted output starts at the
    // model's offset, so first_instance = 0.
    indirect[base + 0u] = p0.z;
    indirect[base + 1u] = count;
    indirect[base + 2u] = p0.w;
    indirect[base + 3u] = bitcast<u32>(p1.x);
    indirect[base + 4u] = 0u;
}

// ── Debug: unconditionally admit every instance (ORBITAL_CULL_DEBUG=cull_all)
// Same compaction/indirect path as `cull`, but skips the plane tests —
// discriminates "frustum/bounds math culls everything" from "compaction or
// indirect args are broken".

@compute @workgroup_size(64)
fn cull_all(@builtin(global_invocation_id) id: vec3<u32>) {
    let model = id.y;
    let local_idx = id.x;
    let p0 = params[model * 2u];
    if local_idx >= p0.y { return; }

    let global_idx = p0.x + local_idx;
    let slot = atomicAdd(&counters[model], 1u);
    let out_base = (p0.x + slot) * 4u;
    let in_base = global_idx * 4u;
    out_instances[out_base + 0u] = vec4<f32>(
        in_instances[in_base].x, in_instances[in_base].y,
        in_instances[in_base].z, in_instances[in_base].w);
    out_instances[out_base + 1u] = vec4<f32>(
        in_instances[in_base + 1u].x, in_instances[in_base + 1u].y,
        in_instances[in_base + 1u].z, in_instances[in_base + 1u].w);
    out_instances[out_base + 2u] = vec4<f32>(
        in_instances[in_base + 2u].x, in_instances[in_base + 2u].y,
        in_instances[in_base + 2u].z, in_instances[in_base + 2u].w);
    out_instances[out_base + 3u] = vec4<f32>(
        in_instances[in_base + 3u].x, in_instances[in_base + 3u].y,
        in_instances[in_base + 3u].z, in_instances[in_base + 3u].w);
}
