// Shared analytic time-of-day sky — the colour of a given world-space
// direction `D`.  Used by both the direct-to-cube sky generator and the
// analytic diffuse generator, so the skybox and the irradiance stay in sync.
//
// Parameters are uploaded via a uniform buffer matching the layout below.
// Total struct size: 11 rows × 16 bytes = 176 bytes.
//
// Must stay in sync with `GeneratedSkyParameters` / `make_sky_parameters_buffer`.

const PI: f32 = 3.14159265359;
const TWO_PI: f32 = 6.28318530718;

// ---------------------------------------------------------------------------
// Uniform buffer — every row is 16 bytes (std140 alignment).
// ---------------------------------------------------------------------------
struct SkyParams {
    // Row 0 (offset  0): sun_direction (vec3) + 4 bytes pad
    sun_direction: vec3<f32>,
    _pad0: f32,

    // Row 1 (offset 16): 4 × f32
    sun_angular_radius: f32,
    sun_intensity: f32,
    moon_angular_radius: f32,
    moon_intensity: f32,

    // Row 2 (offset 32): 4 × f32
    star_intensity: f32,
    star_density: f32,
    exposure: f32,
    _pad1: f32,

    // Row 3 (offset 48): ground_albedo (vec3) + 4 bytes pad
    ground_albedo: vec3<f32>,
    _pad2: f32,

    // Rows 4-10 (offset 64..): palette (7 × vec3 + pad each)
    day_zenith: vec3<f32>,
    day_horizon: vec3<f32>,
    night_zenith: vec3<f32>,
    night_horizon: vec3<f32>,
    twilight: vec3<f32>,
    sun_color: vec3<f32>,
    moon_color: vec3<f32>,
}

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
// Sky colour for a world-space direction `D`.
// ---------------------------------------------------------------------------
fn sky_color(D: vec3<f32>, params: SkyParams) -> vec3<f32> {
    let sun_dir = params.sun_direction;
    let sun_elev = sun_dir.y; // in [-1, 1]

    let day_factor = smoothstep(-0.1, 0.25, sun_elev);
    let night_factor = 1.0 - day_factor;

    // --- Day / night sky gradient -----------------------------------------
    let zenith = mix(params.night_zenith, params.day_zenith, day_factor);
    let horizon = mix(params.night_horizon, params.day_horizon, day_factor);

    // Per-pixel vertical fade between horizon and zenith.
    let height = clamp(D.y, 0.0, 1.0);
    var colour = mix(horizon, zenith, pow(height, 0.5));

    // --- Twilight warm band -----------------------------------------------
    // Peaks when the sun sits on the horizon and hugs the horizon line.
    let twilight = exp(-abs(sun_elev) * 8.0);
    let horizon_term = pow(1.0 - height, 2.0);
    colour += params.twilight * twilight * horizon_term;

    // --- Sun disk + halo --------------------------------------------------
    let cos_a = clamp(dot(D, sun_dir), -1.0, 1.0);
    let sun_ang = acos(cos_a);
    let sun_visible = smoothstep(-0.05, 0.05, sun_elev);

    // Soft Gaussian core: a smooth falloff from a bright centre so the sun
    // reads as a gradient instead of a flat-topped dot.
    let core_sigma = params.sun_angular_radius;
    let disk = exp(-0.5 * sun_ang * sun_ang / (core_sigma * core_sigma));
    colour += params.sun_color * params.sun_intensity * disk * sun_visible;

    // Compact halo, sized relative to the sun so it stays a tight glow.
    let halo_sigma = params.sun_angular_radius * 2.5;
    let halo = exp(-0.5 * sun_ang * sun_ang / (halo_sigma * halo_sigma));
    colour += params.twilight * params.sun_intensity * 0.1 * halo * sun_visible;

    // --- Moon (opposite the sun, visible at night) -------------------------
    let moon_dir = -sun_dir;
    let moon_ang = acos(clamp(dot(D, moon_dir), -1.0, 1.0));
    let moon_visible = smoothstep(0.05, -0.05, sun_elev); // night only

    let moon_disk = smoothstep(
        params.moon_angular_radius * 1.5,
        params.moon_angular_radius * 0.5,
        moon_ang,
    );
    colour += params.moon_color * params.moon_intensity * moon_disk * moon_visible;

    let moon_halo = exp(-moon_ang * moon_ang / (2.0 * 0.02 * 0.02));
    colour += params.moon_color * params.moon_intensity * 0.3 * moon_halo * moon_visible;

    // --- Stars (deterministic, faded in at night) --------------------------
    const STAR_GRID: f32 = 90.0;
    let bias = i32(STAR_GRID);
    let cell = vec3<i32>(floor(D * STAR_GRID)) + vec3<i32>(bias);
    let h = star_hash(u32(cell.x), u32(cell.y), u32(cell.z));
    let bright = f32(h & 0xFFFFu) / 65535.0;

    // Only a fraction of cells (star_density) actually contain a star.
    let threshold = 1.0 - clamp(params.star_density, 0.0, 1.0);
    if bright > threshold {
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
        let star_bright = (bright - threshold) / (1.0 - threshold) * star_val;
        let star_colour = vec3<f32>(0.85, 0.9, 1.0);
        colour += star_colour * star_bright * params.star_intensity
                * night_factor * 0.8;
    }

    // --- Ground (below the horizon) ----------------------------------------
    let ground_fade = smoothstep(-0.02, 0.02, D.y);
    colour = mix(params.ground_albedo, colour, ground_fade);

    // --- Exposure ----------------------------------------------------------
    colour *= params.exposure;

    return colour;
}
