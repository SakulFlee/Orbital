// Analytic diffuse (irradiance) IBL for the generated sky.
//
// Instead of an 8192-sample Monte Carlo convolution, the irradiance is
// computed deterministically:
//
//   1. A fixed Fibonacci-sphere quadrature integrates the smooth parts of the
//      sky (day/night gradient, twilight, sun/moon halos, stars, ground).
//   2. Closed-form sun/moon disk terms (tiny solid angle that the fixed
//      quadrature cannot resolve) are added explicitly.
//
// The result is noise-free, cheap and consistent with `sky_color`.
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

// Deterministic Fibonacci-sphere direction. Uniform over the unit sphere.
fn fib_direction(i: u32, n: u32) -> vec3<f32> {
    const GOLDEN_ANGLE: f32 = 2.399963229728653;
    let phi = f32(i) * GOLDEN_ANGLE;
    let y = 1.0 - (f32(i) + 0.5) / f32(n) * 2.0;
    let r = sqrt(max(0.0, 1.0 - y * y));
    return vec3<f32>(cos(phi) * r, y, sin(phi) * r);
}

// Closed-form irradiance of a small uniform disk of angular radius `radius`
// (steradians) centred on `disk_dir`, as seen from a surface with normal `N`.
// The disk is assumed small enough that `dot(ω, N)` is roughly constant.
fn disk_irradiance(
    disk_dir: vec3<f32>,
    radius: f32,
    radiance: vec3<f32>,
    N: vec3<f32>,
) -> vec3<f32> {
    let cos_n = max(dot(N, disk_dir), 0.0);
    if cos_n <= 0.0 {
        return vec3<f32>(0.0);
    }
    let solid_angle = TWO_PI * (1.0 - cos(radius));
    return radiance * solid_angle * cos_n;
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

    // --- Cubemap texel → world-space direction (the normal) ----------------
    let face = gid_z_to_face(gid.z);
    let cube_uv = (vec2<f32>(gid.xy) + 0.5) / vec2<f32>(dims) * 2.0 - 1.0;
    let N = normalize(face.forward + face.right * cube_uv.x - face.up * cube_uv.y);

    // --- Fibonacci-sphere quadrature for the smooth parts -------------------
    // The sky is smooth (gradients + halos) and the sun/moon are added in
    // closed form below, so 64 samples is visually indistinguishable from
    // more while costing half as much.
    const N_SAMPLES: u32 = 64u;
    let weight = 4.0 * PI / f32(N_SAMPLES); // solid angle per sample

    var irradiance = vec3<f32>(0.0);
    for (var i = 0u; i < N_SAMPLES; i++) {
        let dir = fib_direction(i, N_SAMPLES);
        let ndotl = max(dot(dir, N), 0.0);
        if ndotl > 0.0 {
            irradiance += sky_color(dir, params) * ndotl;
        }
    }
    irradiance *= weight;

    // --- Closed-form sun / moon disks ---------------------------------------
    let sun_dir = params.sun_direction;
    let sun_elev = sun_dir.y;
    let sun_visible = smoothstep(-0.05, 0.05, sun_elev);
    // `smoothstep` requires `edge0 < edge1`, so night = `1.0 - day`.
    let moon_visible = 1.0 - smoothstep(-0.05, 0.05, sun_elev);

    if sun_visible > 0.0 {
        irradiance += disk_irradiance(
            sun_dir,
            params.sun_angular_radius,
            params.sun_color * params.sun_intensity * sun_visible,
            N,
        );
    }
    if moon_visible > 0.0 {
        irradiance += disk_irradiance(
            -sun_dir,
            params.moon_angular_radius,
            params.moon_color * params.moon_intensity * moon_visible,
            N,
        );
    }

    textureStore(dst, gid.xy, gid.z, vec4<f32>(irradiance, 1.0));
}
