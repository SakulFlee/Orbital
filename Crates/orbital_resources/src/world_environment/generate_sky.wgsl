// Atmospheric scattering sky generation — produces an equirectangular
// HDR texture via single-scattering ray marching (Rayleigh + Mie).
//
// Each pixel is computed independently: convert equirectangular UV to a
// world-space direction, ray-march through the atmosphere shell
// accumulating scattered light, and apply the sun disk on top.
//
// Parameters are uploaded via a uniform buffer matching the layout below.
// Total struct size: 6 rows × 16 bytes = 96 bytes.

const PI: f32 = 3.14159265359;
const VIEW_SAMPLES: u32 = 64u;
const SUN_SAMPLES: u32 = 8u;

// ---------------------------------------------------------------------------
// Uniform buffer — must stay in sync with GeneratedSkyParameters on the
// Rust side.  Every row is 16 bytes (std140 alignment).
// ---------------------------------------------------------------------------
struct SkyParams {
    // Row 0 (offset  0): vec3<f32> + 4 bytes pad
    sun_direction_x: f32,
    sun_direction_y: f32,
    sun_direction_z: f32,
    _pad0: f32,

    // Row 1 (offset 16): 4 × f32
    sun_angular_radius: f32,
    sun_intensity: f32,
    rayleigh_scale_height: f32,
    mie_scale_height: f32,

    // Row 2 (offset 32): vec3<f32> + 4 bytes pad
    rayleigh_scatter_r: f32,
    rayleigh_scatter_g: f32,
    rayleigh_scatter_b: f32,
    _pad1: f32,

    // Row 3 (offset 48): 4 × f32
    mie_scattering_coeff: f32,
    mie_absorption_coeff: f32,
    mie_anisotropy: f32,
    _pad2: f32,

    // Row 4 (offset 64): vec3<f32> + 4 bytes pad
    ground_albedo_r: f32,
    ground_albedo_g: f32,
    ground_albedo_b: f32,
    _pad3: f32,

    // Row 5 (offset 80): 4 × f32
    planet_radius: f32,
    atmosphere_radius: f32,
    exposure: f32,
    _pad4: f32,
}

// ---------------------------------------------------------------------------
// Bindings
// ---------------------------------------------------------------------------
@group(0) @binding(0) var<uniform> params: SkyParams;
@group(0) @binding(1) var dst: texture_storage_2d<rgba32float, write>;

// ---------------------------------------------------------------------------
// Phase functions
// ---------------------------------------------------------------------------
fn rayleigh_phase(cos_theta: f32) -> f32 {
    return (3.0 / (16.0 * PI)) * (1.0 + cos_theta * cos_theta);
}

fn henyey_greenstein(cos_theta: f32, g: f32) -> f32 {
    let g2 = g * g;
    let denominator = 1.0 + g2 - 2.0 * g * cos_theta;
    return (1.0 - g2) / (4.0 * PI * denominator * sqrt(denominator));
}

// ---------------------------------------------------------------------------
// Ray-sphere intersection
//
// Returns (t_near, t_far) for a ray O + t*D intersecting a sphere centred
// at the origin with the given radius.  If there is no intersection both
// components are -1.
// ---------------------------------------------------------------------------
fn ray_sphere_intersection(
    ray_origin: vec3<f32>,
    ray_dir: vec3<f32>,
    radius: f32,
) -> vec2<f32> {
    let b = 2.0 * dot(ray_origin, ray_dir);
    let c = dot(ray_origin, ray_origin) - radius * radius;
    let discriminant = b * b - 4.0 * c;
    if discriminant < 0.0 {
        return vec2(-1.0, -1.0);
    }
    let d = sqrt(discriminant);
    return vec2((-b - d) * 0.5, (-b + d) * 0.5);
}

// ---------------------------------------------------------------------------
// Integral of exp(-h / scale_height) along a ray segment (optical depth).
// Used for the *inner* sun-direction march.
// ---------------------------------------------------------------------------
fn integrate_optical_depth(
    ray_start: vec3<f32>,
    ray_dir: vec3<f32>,
    ray_len: f32,
    num_samples: u32,
    scale_height: f32,
    planet_radius: f32,
) -> f32 {
    let step = ray_len / f32(num_samples);
    var od: f32 = 0.0;
    for (var i = 0u; i < num_samples; i++) {
        let t = (f32(i) + 0.5) * step;
        let P = ray_start + t * ray_dir;
        let h = length(P) - planet_radius;
        if h > 0.0 {
            od += exp(-h / scale_height) * step;
        }
    }
    return od;
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

    // --- Pack scalar parameters -------------------------------------------
    let R_planet = params.planet_radius;
    let R_atm    = params.atmosphere_radius;
    let H_R      = params.rayleigh_scale_height;
    let H_M      = params.mie_scale_height;

    let beta_R = vec3<f32>(
        params.rayleigh_scatter_r,
        params.rayleigh_scatter_g,
        params.rayleigh_scatter_b,
    );
    let beta_M = vec3<f32>(params.mie_scattering_coeff);
    let beta_M_abs = vec3<f32>(params.mie_absorption_coeff);
    let beta_M_ext = beta_M + beta_M_abs; // Mie extinction
    let g_mie      = params.mie_anisotropy;

    let sun_dir = normalize(vec3<f32>(
        params.sun_direction_x,
        params.sun_direction_y,
        params.sun_direction_z,
    ));

    // --- Equirectangular UV → world-space direction -----------------------
    //
    // Inverse of the mapping used by make_ibl_diffuse / make_ibl_specular:
    //   u = atan2(L.z, L.x) / (2π) + 0.5       →  phi
    //   v = asin(L.y)    / π     + 0.5          →  theta
    let uv = (vec2<f32>(gid.xy) + 0.5) / dims;
    let phi   = (uv.x - 0.5) * 2.0 * PI;
    let y_dir = sin((uv.y - 0.5) * PI);
    let r_dir = cos((uv.y - 0.5) * PI);
    let D = normalize(vec3<f32>(r_dir * cos(phi), y_dir, r_dir * sin(phi)));

    // --- Viewer position (on the planet surface at the "equator") ---------
    let viewer = vec3<f32>(0.0, R_planet, 0.0);

    // --- Atmosphere intersection ------------------------------------------
    let atm_hit = ray_sphere_intersection(viewer, D, R_atm);
    let t_atm_exit = atm_hit.y; // viewer is inside → t_near < 0, t_far > 0

    // --- Planet intersection (second root — where the ray re-enters) ------
    let p_hit = ray_sphere_intersection(viewer, D, R_planet);
    let t_planet = select(p_hit.y, 1e20, p_hit.y <= 0.001);

    let max_t = min(t_atm_exit, t_planet);
    if max_t <= 0.0 {
        let ground = vec3<f32>(
            params.ground_albedo_r,
            params.ground_albedo_g,
            params.ground_albedo_b,
        );
        textureStore(dst, gid.xy, vec4<f32>(ground * params.exposure, 1.0));
        return;
    }

    // --- Ray-march the view ray -------------------------------------------
    let step = max_t / f32(VIEW_SAMPLES);
    var view_od_R: f32 = 0.0;
    var view_od_M: f32 = 0.0;
    var colour = vec3<f32>(0.0);
    // Track the transmittance at the atmosphere exit for the sun disk.
    var final_transmittance = vec3<f32>(1.0);

    for (var i = 0u; i < VIEW_SAMPLES; i++) {
        let t = (f32(i) + 0.5) * step;
        let P = viewer + t * D;
        let h = length(P) - R_planet;

        let dens_R = exp(-h / H_R);
        let dens_M = exp(-h / H_M);

        // --- Optical depth from P toward the sun --------------------------
        let sun_hit = ray_sphere_intersection(P, sun_dir, R_atm);
        let sun_dist = sun_hit.y;
        var od_R_sun: f32 = 0.0;
        var od_M_sun: f32 = 0.0;
        if sun_dist > 0.0 {
            od_R_sun = integrate_optical_depth(
                P, sun_dir, sun_dist, SUN_SAMPLES, H_R, R_planet,
            );
            od_M_sun = integrate_optical_depth(
                P, sun_dir, sun_dist, SUN_SAMPLES, H_M, R_planet,
            );
        }

        let sun_trans = exp(-(beta_R * od_R_sun + beta_M_ext * od_M_sun));

        // --- Accumulate view optical depth --------------------------------
        view_od_R += dens_R * step;
        view_od_M += dens_M * step;

        let view_trans = exp(-(beta_R * view_od_R + beta_M_ext * view_od_M));

        // --- Scattering at point P ----------------------------------------
        let cos_theta = dot(D, sun_dir);
        let phase_R = rayleigh_phase(cos_theta);
        let phase_M = henyey_greenstein(cos_theta, g_mie);

        let scatter_R = beta_R * dens_R;
        let scatter_M = beta_M * dens_M;

        colour += (scatter_R * phase_R + scatter_M * phase_M)
                * sun_trans * view_trans * step;

        final_transmittance = view_trans;
    }

    // --- Sun disk (direct sunlight) ---------------------------------------
    let cos_sun = dot(D, sun_dir);
    let sun_angle = acos(clamp(cos_sun, -1.0, 1.0));
    let disk_radius = params.sun_angular_radius;

    // Only draw the sun disk when the sun is near or above the horizon.
    if sun_dir.y > -0.15 && sun_angle < disk_radius * 3.0 {
        // Optical depth from viewer to space along the sun direction.
        let sun_hit_viewer = ray_sphere_intersection(viewer, sun_dir, R_atm);
        let sun_to_atm = sun_hit_viewer.y;
        var od_R_sun_disk: f32 = 0.0;
        var od_M_sun_disk: f32 = 0.0;
        if sun_to_atm > 0.0 {
            od_R_sun_disk = integrate_optical_depth(
                viewer, sun_dir, sun_to_atm, 16u, H_R, R_planet,
            );
            od_M_sun_disk = integrate_optical_depth(
                viewer, sun_dir, sun_to_atm, 16u, H_M, R_planet,
            );
        }
        let sun_transmittance =
            exp(-(beta_R * od_R_sun_disk + beta_M_ext * od_M_sun_disk));

        // Angular falloff — use smoothstep for a soft edge.
        let fade = 1.0 - smoothstep(
            disk_radius * 0.3, disk_radius * 2.5, sun_angle,
        );

        // The sun disk is white-ish, slightly warm.
        let sun_colour = vec3<f32>(1.0, 0.92, 0.75);
        colour += sun_colour * params.sun_intensity
                * sun_transmittance * fade * 0.002;
    }

    // --- Apply exposure ---------------------------------------------------
    colour *= params.exposure;

    textureStore(dst, gid.xy, vec4<f32>(colour, 1.0));
}
