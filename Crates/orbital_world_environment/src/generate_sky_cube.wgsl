// Direct-to-cube analytic sky generation.
//
// Computes `sky_color` directly for each cubemap texel (no equirectangular
// intermediate), producing the specular IBL LoD 0 (the skybox).
//
// Face mapping matches `make_ibl_diffuse` / `make_ibl_specular` (WebGPU
// cubemap convention: +Y face looks up).

// ---------------------------------------------------------------------------
// Bindings
// ---------------------------------------------------------------------------
@group(0) @binding(0) var<uniform> params: SkyParams;
@group(0) @binding(1) var dst: texture_storage_2d_array<rgba16float, write>;

// Structure to define a cubemap face
struct Face {
    forward: vec3<f32>,
    up: vec3<f32>,
    right: vec3<f32>,
}

// Function to get face definition based on face index (Z-order)
fn gid_z_to_face(gid_z: u32) -> Face {
    switch gid_z {
        case 0u: { // +X
            return Face(vec3(1.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0), vec3(0.0, 0.0, -1.0));
        }
        case 1u: { // -X
            return Face(vec3(-1.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0), vec3(0.0, 0.0, 1.0));
        }
        case 2u: { // +Y
            return Face(vec3(0.0, 1.0, 0.0), vec3(0.0, 0.0, -1.0), vec3(1.0, 0.0, 0.0));
        }
        case 3u: { // -Y
            return Face(vec3(0.0, -1.0, 0.0), vec3(0.0, 0.0, 1.0), vec3(1.0, 0.0, 0.0));
        }
        case 4u: { // +Z
            return Face(vec3(0.0, 0.0, 1.0), vec3(0.0, 1.0, 0.0), vec3(1.0, 0.0, 0.0));
        }
        case 5u: { // -Z
            return Face(vec3(0.0, 0.0, -1.0), vec3(0.0, 1.0, 0.0), vec3(-1.0, 0.0, 0.0));
        }
        default {
            return Face(vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 0.0));
        }
    }
}

// ---------------------------------------------------------------------------
// Main compute kernel
// ---------------------------------------------------------------------------
@compute
@workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let dims = textureDimensions(dst).xy;
    if gid.x >= dims.x || gid.y >= dims.y {
        return;
    }

    // --- Cubemap texel → world-space direction -----------------------------
    let face = gid_z_to_face(gid.z);
    let cube_uv = (vec2<f32>(gid.xy) + 0.5) / vec2<f32>(dims) * 2.0 - 1.0;
    let D = normalize(face.forward + face.right * cube_uv.x - face.up * cube_uv.y);

    let colour = sky_color(
        D,
        params.sun_direction,
        params.sun_angular_radius,
        params.sun_intensity,
        params.moon_angular_radius,
        params.moon_intensity,
        params.star_intensity,
        params.star_density,
        params.exposure,
        params.ground_albedo,
        params.day_zenith,
        params.day_horizon,
        params.night_zenith,
        params.night_horizon,
        params.twilight,
        params.sun_color,
        params.moon_color,
    );

    textureStore(dst, gid.xy, gid.z, vec4<f32>(colour, 1.0));
}
