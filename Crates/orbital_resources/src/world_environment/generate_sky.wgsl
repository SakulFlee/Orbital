// Analytic time-of-day sky generation — produces an equirectangular HDR
// texture without any ray marching.  The look is driven by a single
// `time_of_day_hours` parameter:
//
//   * Day  — blue zenith / pale horizon gradient, bright sun disk + halo.
//   * Dusk/dawn — warm orange band near the horizon.
//   * Night — dark blue-black sky, a procedural deterministic starfield,
//             and a moon opposite the sun.
//
// Each pixel is computed independently from its world-space direction, so
// the output is perfectly stable (no per-frame noise) and cheap.
//
// Parameters are uploaded via a uniform buffer matching the layout below.
// Total struct size: 3 rows × 16 bytes = 48 bytes.

const PI: f32 = 3.14159265359;
const TWO_PI: f32 = 6.28318530718;

// ---------------------------------------------------------------------------
// Uniform buffer — must stay in sync with GeneratedSkyParameters on the
// Rust side.  Every row is 16 bytes (std140 alignment).
// ---------------------------------------------------------------------------
struct SkyParams {
    // Row 0 (offset  0): 4 × f32
    time_of_day_hours: f32,
    sun_azimuth: f32,
    sun_angular_radius: f32,
    sun_intensity: f32,

    // Row 1 (offset 16): 4 × f32
    moon_angular_radius: f32,
    moon_intensity: f32,
    star_intensity: f32,
    exposure: f32,

    // Row 2 (offset 32): vec3<f32> + 4 bytes pad
    ground_albedo_r: f32,
    ground_albedo_g: f32,
    ground_albedo_b: f32,
    _pad: f32,
}

// ---------------------------------------------------------------------------
// Bindings
// ---------------------------------------------------------------------------
@group(0) @binding(0) var<uniform> params: SkyParams;
@group(0) @binding(1) var dst: texture_storage_2d<rgba32float, write>;

// ---------------------------------------------------------------------------
// Deterministic 3D integer hash (simple integer-noise), used for the stars.
// ---------------------------------------------------------------------------
fn star_hash(cx: u32, cy: u32, cz: u32) -> u32 {
    var h = cx * 374761393u + cy * 668265263u + cz * 1274126177u;
    h = (h ^ (h >> 13u)) * 1103515245u;
    h = h ^ (h >> 16u);
    return h;
}

// ---------------------------------------------------------------------------
// Main compute kernel
// ---------------------------------------------------------------------------
@compute
@workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let dims = vec2<f32>(textureDimensions(dst));
    if gid.x >= u32(dims.x) || gid.y >= u32(dims.y) {
        return;
    }

    // --- Equirectangular UV → world-space direction -----------------------
    //
    // Inverse of the mapping used by make_ibl_diffuse / make_ibl_specular:
    //   u = atan2(L.z, L.x) / (2π) + 0.5       →  phi
    //   v = asin(L.y)    / π     + 0.5          →  theta
    let uv = (vec2<f32>(gid.xy) + 0.5) / dims;
    let phi   = (uv.x - 0.5) * TWO_PI;
    let y_dir = sin((uv.y - 0.5) * PI);
    let r_dir = cos((uv.y - 0.5) * PI);
    let D = normalize(vec3<f32>(r_dir * cos(phi), y_dir, r_dir * sin(phi)));

    // --- Sun position from time of day ------------------------------------
    // `theta` is the sun's elevation angle over 24 h:
    //   6 h → 0 (dawn, on horizon), 12 h → π/2 (noon, overhead),
    //  18 h → π (dusk, opposite horizon), 0/24 h → −π/2 (midnight, below).
    let theta = (params.time_of_day_hours - 6.0) / 24.0 * TWO_PI;
    let cos_t = cos(theta);
    let sin_t = sin(theta);
    let sun_elev = sin_t; // in [-1, 1]

    let sun_dir = normalize(vec3<f32>(
        cos_t * cos(params.sun_azimuth),
        sin_t,
        cos_t * sin(params.sun_azimuth),
    ));

    let day_factor = smoothstep(-0.1, 0.25, sun_elev);
    let night_factor = 1.0 - day_factor;

    // --- Day / night sky gradient -----------------------------------------
    let day_zenith   = vec3<f32>(0.35, 0.55, 0.95);
    let day_horizon  = vec3<f32>(0.75, 0.85, 1.0);
    let night_zenith = vec3<f32>(0.005, 0.008, 0.03);
    let night_horizon = vec3<f32>(0.02, 0.02, 0.05);

    let zenith   = mix(night_zenith, day_zenith, day_factor);
    let horizon  = mix(night_horizon, day_horizon, day_factor);

    // Per-pixel vertical fade between horizon and zenith.
    let height = clamp(D.y, 0.0, 1.0);
    var colour = mix(horizon, zenith, pow(height, 0.5));

    // --- Twilight warm band -----------------------------------------------
    // Peaks when the sun sits on the horizon and hugs the horizon line.
    let twilight = exp(-abs(sun_elev) * 8.0);
    let horizon_term = pow(1.0 - height, 2.0);
    let warm = vec3<f32>(1.5, 0.7, 0.25);
    colour += warm * twilight * horizon_term;

    // --- Sun disk + halo --------------------------------------------------
    let cos_a = clamp(dot(D, sun_dir), -1.0, 1.0);
    let sun_ang = acos(cos_a);
    let sun_visible = smoothstep(-0.05, 0.05, sun_elev);

    // Soft HDR disk with a slight warm tint.
    let disk = smoothstep(
        params.sun_angular_radius * 1.5,
        params.sun_angular_radius * 0.5,
        sun_ang,
    );
    let sun_colour = vec3<f32>(1.0, 0.92, 0.78);
    colour += sun_colour * params.sun_intensity * disk * sun_visible;

    // Wide faint halo so the sun reads as a bright glowing spot.
    let halo = exp(-sun_ang * sun_ang / (2.0 * 0.12 * 0.12));
    colour += warm * params.sun_intensity * 0.15 * halo * sun_visible;

    // --- Moon (opposite the sun, visible at night) -------------------------
    let moon_dir = -sun_dir;
    let moon_ang = acos(clamp(dot(D, moon_dir), -1.0, 1.0));
    let moon_visible = smoothstep(0.05, -0.05, sun_elev); // night only

    let moon_disk = smoothstep(
        params.moon_angular_radius * 1.5,
        params.moon_angular_radius * 0.5,
        moon_ang,
    );
    let moon_colour = vec3<f32>(0.72, 0.76, 0.85);
    colour += moon_colour * params.moon_intensity * moon_disk * moon_visible;

    let moon_halo = exp(-moon_ang * moon_ang / (2.0 * 0.02 * 0.02));
    colour += moon_colour * params.moon_intensity * 0.3 * moon_halo * moon_visible;

    // --- Stars (deterministic, faded in at night) --------------------------
    const STAR_GRID: f32 = 90.0;
    let bias = i32(STAR_GRID);
    let cell = vec3<i32>(floor(D * STAR_GRID)) + vec3<i32>(bias);
    let h = star_hash(u32(cell.x), u32(cell.y), u32(cell.z));
    let bright = f32(h & 0xFFFFu) / 65535.0;

    // Only a fraction of cells actually contain a star.
    if bright > 0.94 {
        // Star centre offset inside the cell (also hashed → stable).
        let off = star_hash(
            u32(cell.x) + 0x9E3779B9u,
            u32(cell.y) + 0x85EBCA6Bu,
            u32(cell.z) + 0xC2B2AE35u,
        );
        let ox = f32(off & 0xFFu) / 255.0 - 0.5;
        let oy = f32((off >> 8u) & 0xFFu) / 255.0 - 0.5;
        let oz = f32((off >> 16u) & 0xFFu) / 255.0 - 0.5;
        let centre = (vec3<f32>(cell) - vec3<f32>(f32(bias))) + vec3<f32>(ox, oy, oz);
        let dist = length(D * STAR_GRID - centre);

        let star_val = exp(-dist * dist * 60.0);
        let star_bright = (bright - 0.94) / 0.06 * star_val;
        let star_colour = vec3<f32>(0.85, 0.9, 1.0);
        colour += star_colour * star_bright * params.star_intensity
                * night_factor * 0.8;
    }

    // --- Ground (below the horizon) ----------------------------------------
    let ground = vec3<f32>(
        params.ground_albedo_r,
        params.ground_albedo_g,
        params.ground_albedo_b,
    );
    let ground_fade = smoothstep(-0.02, 0.02, D.y);
    colour = mix(ground, colour, ground_fade);

    // --- Exposure ----------------------------------------------------------
    colour *= params.exposure;

    textureStore(dst, gid.xy, vec4<f32>(colour, 1.0));
}
