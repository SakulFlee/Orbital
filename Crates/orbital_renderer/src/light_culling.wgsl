// Tiled light culling.
//
// One invocation owns one screen tile. It builds a conservative world-space
// AABB from the tile's depth range, then writes the indices of lights whose
// influence volume overlaps that AABB. Directional lights are always included.

struct TileParams {
    tiles_x: u32,
    tiles_y: u32,
    max_lights_per_tile: u32,
    tile_size: u32,
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(0) @binding(1) var<storage> light_store: array<Light>;
@group(0) @binding(13) var<uniform> light_count: u32;

@group(1) @binding(0) var depth_texture: texture_depth_2d;
@group(1) @binding(1) var<storage, read_write> tile_light_counts: array<u32>;
@group(1) @binding(2) var<storage, read_write> tile_light_indices: array<u32>;
@group(1) @binding(3) var<uniform> tile_params: TileParams;

fn world_from_ndc_depth(ndc: vec2<f32>, depth: f32) -> vec3<f32> {
    let view_position_h = camera.perspective_projection_invert * vec4<f32>(ndc, depth, 1.0);
    let view_position = view_position_h.xyz / view_position_h.w;
    let world_offset = (camera.view_projection_transposed * vec4<f32>(view_position, 0.0)).xyz;
    return camera.position.xyz + world_offset;
}

fn tile_aabb(ndc_min: vec2<f32>, ndc_max: vec2<f32>, min_depth: f32, max_depth: f32) -> array<vec3<f32>, 2> {
    var aabb_min = vec3<f32>(1e20);
    var aabb_max = vec3<f32>(-1e20);

    for (var i = 0u; i < 8u; i++) {
        let x = select(ndc_min.x, ndc_max.x, (i & 1u) != 0u);
        let y = select(ndc_min.y, ndc_max.y, (i & 2u) != 0u);
        let d = select(min_depth, max_depth, (i & 4u) != 0u);
        let position = world_from_ndc_depth(vec2<f32>(x, y), d);
        aabb_min = min(aabb_min, position);
        aabb_max = max(aabb_max, position);
    }

    return array<vec3<f32>, 2>(aabb_min, aabb_max);
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let tile = global_id.xy;
    if (tile.x >= tile_params.tiles_x || tile.y >= tile_params.tiles_y) {
        return;
    }

    let tile_index = tile.y * tile_params.tiles_x + tile.x;
    tile_light_counts[tile_index] = 0u;

    let dimensions = textureDimensions(depth_texture);
    let tile_size = max(tile_params.tile_size, 1u);
    let x0 = tile.x * tile_size;
    let y0 = tile.y * tile_size;
    let x1 = min(x0 + tile_size, dimensions.x);
    let y1 = min(y0 + tile_size, dimensions.y);

    var min_depth = 1.0;
    var max_depth = 0.0;
    var has_geometry = false;
    for (var y = y0; y < y1; y += 1u) {
        for (var x = x0; x < x1; x += 1u) {
            let depth = textureLoad(depth_texture, vec2<i32>(i32(x), i32(y)), 0);
            if (depth < 1.0) {
                has_geometry = true;
                min_depth = min(min_depth, depth);
                max_depth = max(max_depth, depth);
            }
        }
    }

    if (!has_geometry) {
        return;
    }

    let inv_width = 1.0 / f32(dimensions.x);
    let inv_height = 1.0 / f32(dimensions.y);
    let ndc_min = vec2<f32>(
        f32(x0) * inv_width * 2.0 - 1.0,
        1.0 - f32(y1) * inv_height * 2.0,
    );
    let ndc_max = vec2<f32>(
        f32(x1) * inv_width * 2.0 - 1.0,
        1.0 - f32(y0) * inv_height * 2.0,
    );
    let bounds = tile_aabb(ndc_min, ndc_max, min_depth, max_depth);
    let aabb_min = bounds[0];
    let aabb_max = bounds[1];

    var count = 0u;
    let index_base = tile_index * tile_params.max_lights_per_tile;
    for (var light_index = 0u; light_index < light_count; light_index += 1u) {
        let light = light_store[light_index];
        if (light.color.w == 0.0) {
            continue;
        }

        var intersects = light.direction.w == LIGHT_TYPE_DIRECTIONAL;
        if (!intersects && light.params.z > 0.0) {
            let closest = clamp(light.position.xyz, aabb_min, aabb_max);
            let offset = light.position.xyz - closest;
            intersects = dot(offset, offset) <= light.params.z;
        } else if (!intersects) {
            // Infinite range or an unknown type: keep it conservative.
            intersects = true;
        }

        if (!intersects) {
            continue;
        }
        if (count >= tile_params.max_lights_per_tile) {
            // More lights overlap this tile than fit in the fixed list. The
            // deferred pass falls back to the full light loop for this tile.
            tile_light_counts[tile_index] = 0xFFFFFFFFu;
            return;
        }

        tile_light_indices[index_base + count] = light_index;
        count += 1u;
    }

    tile_light_counts[tile_index] = count;
}
