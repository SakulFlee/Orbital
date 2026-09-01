const INV_ATAN = vec2<f32>(0.1591, 0.3183);

@group(0) @binding(0)
var src: texture_2d<f32>;

@group(0) @binding(1)
var dst: texture_storage_2d_array<rgba16float, write>;

// Tile offset and face index passed from the CPU (gid.z is always 0 for
// per-face tiled dispatches, so we receive the actual face index here).
struct TileParams {
    tile_offset: vec2<u32>,
    face_index: u32,
    _pad: u32,
}
@group(0) @binding(2)
var<uniform> params: TileParams;

struct Face {
    forward: vec3<f32>,
    up: vec3<f32>,
    right: vec3<f32>,
}

fn gid_z_to_face(gid_z: u32) -> Face {
    switch gid_z {
        // FACE +X
        case 0u: {
            return Face(
                vec3(1.0, 0.0, 0.0),  // forward
                vec3(0.0, 1.0, 0.0),  // up
                vec3(0.0, 0.0, -1.0), // right
            );
        }
        // FACE -X
        case 1u: {
            return Face(
                vec3(-1.0, 0.0, 0.0),
                vec3(0.0, 1.0, 0.0),
                vec3(0.0, 0.0, 1.0),
            );
        }
        // FACE +Y
        case 2u: {
            return Face(
                vec3(0.0, 1.0, 0.0),
                vec3(0.0, 0.0, -1.0),
                vec3(1.0, 0.0, 0.0),
            );
        }
        // FACE -Y
        case 3u: {
            return Face(
                vec3(0.0, -1.0, 0.0),
                vec3(0.0, 0.0, 1.0),
                vec3(1.0, 0.0, 0.0),
            );
        }
        // FACE +Z
        case 4u: {
            return Face(
                vec3(0.0, 0.0, 1.0),
                vec3(0.0, 1.0, 0.0),
                vec3(1.0, 0.0, 0.0),
            );
        }
        // FACE -Z
        case 5u: {
            return Face(
                vec3(0.0, 0.0, -1.0),
                vec3(0.0, 1.0, 0.0),
                vec3(-1.0, 0.0, 0.0),
            );
        }
        // SHOULD NEVER TRIGGER!
        default: {
            return Face(
                vec3(0.0, 0.0, 0.0),
                vec3(0.0, 0.0, 0.0),
                vec3(0.0, 0.0, 0.0),
            );
        }
    }
}

@compute
@workgroup_size(8, 8, 1)
fn main(
    @builtin(global_invocation_id)
    gid: vec3<u32>,
) {
    let src_dimensions = vec2<f32>(textureDimensions(src));
    let dst_dimensions = vec2<f32>(textureDimensions(dst));
    let abs_xy = gid.xy + params.tile_offset;

    if abs_xy.x >= u32(dst_dimensions.x) {
        return;
    }

    let cube_uv = (vec2<f32>(abs_xy) + 0.5) / dst_dimensions * 2.0 - 1.0;

    let face = gid_z_to_face(params.face_index);
    let N = normalize(face.forward + face.right * cube_uv.x - face.up * cube_uv.y);

    let eq_uv = vec2(
        atan2(N.z, N.x), 
        asin(N.y)
    ) * INV_ATAN + 0.5;
    let eq_pixel = vec2<u32>(eq_uv * src_dimensions);

    let sample = textureLoad(src, eq_pixel, 0);
    textureStore(dst, abs_xy, gid.z, sample);
}
