// Sky colour for a world-space direction `D`.
//
// Takes individual parameters instead of a `SkyParams` struct by value,
// because some Adreno Vulkan drivers miscompile large struct-by-value
// arguments (fields read as ~0, producing a black skybox).

fn sky_color(
    D: vec3<f32>,
    sun_direction: vec3<f32>,
    sun_angular_radius: f32,
    sun_intensity: f32,
    moon_angular_radius: f32,
    moon_intensity: f32,
    star_intensity: f32,
    star_density: f32,
    exposure: f32,
    ground_albedo: vec3<f32>,
    day_zenith: vec3<f32>,
    day_horizon: vec3<f32>,
    night_zenith: vec3<f32>,
    night_horizon: vec3<f32>,
    twilight: vec3<f32>,
    sun_color: vec3<f32>,
    moon_color: vec3<f32>,
) -> vec3<f32> {
    let sun_dir = sun_direction;
    let sun_elev = sun_dir.y; // in [-1, 1]

    let day_factor = smoothstep(-0.1, 0.25, sun_elev);
    let night_factor = 1.0 - day_factor;

    let zenith = mix(night_zenith, day_zenith, day_factor);
    let horizon = mix(night_horizon, day_horizon, day_factor);

    let height = clamp(D.y, 0.0, 1.0);
    var colour = mix(horizon, zenith, pow(height, 0.3));

    let twilight_ish = exp(-abs(sun_elev) * 5.0);
    let twilight_visible = smoothstep(-0.15, 0.0, sun_elev);
    let horizon_term = pow(1.0 - height, 2.0);
    colour += twilight * twilight_ish * horizon_term * twilight_visible;

    let cos_a = clamp(dot(D, sun_dir), -1.0, 1.0);
    let sun_ang = acos(cos_a);
    let sun_visible = smoothstep(-0.05, 0.05, sun_elev);

    let core_sigma = sun_angular_radius;
    let disk = exp(-0.5 * sun_ang * sun_ang / (core_sigma * core_sigma));
    colour += sun_color * sun_intensity * disk * sun_visible;

    let halo_sigma = sun_angular_radius * 2.5;
    let halo = exp(-0.5 * sun_ang * sun_ang / (halo_sigma * halo_sigma));
    colour += twilight * sun_intensity * 0.1 * halo * sun_visible;

    let moon_dir = -sun_dir;
    let moon_ang = acos(clamp(dot(D, moon_dir), -1.0, 1.0));
    let moon_visible = 1.0 - smoothstep(-0.05, 0.05, sun_elev); // night only

    let moon_sigma = moon_angular_radius;
    let moon_disk = exp(-pow(moon_ang / moon_sigma, 8.0));
    colour += moon_color * moon_intensity * moon_disk * moon_visible;

    let moon_halo_sigma = moon_sigma * 1.5;
    let moon_halo = exp(-moon_ang * moon_ang / (2.0 * moon_halo_sigma * moon_halo_sigma));
    colour += moon_color * moon_intensity * 0.1 * moon_halo * moon_visible;

    const STAR_GRID: f32 = 90.0;
    let bias = i32(STAR_GRID);
    let cell = vec3<i32>(floor(D * STAR_GRID)) + vec3<i32>(bias);
    let h = star_hash(u32(cell.x), u32(cell.y), u32(cell.z));
    let bright = f32(h & 0xFFFFu) / 65535.0;

    let threshold = 1.0 - clamp(star_density, 0.0, 1.0);
    if bright > threshold {
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

        let star_val = exp(-dist * dist * 30.0);
        let star_bright = (bright - threshold) / (1.0 - threshold) * star_val;
        let star_colour = vec3<f32>(0.85, 0.9, 1.0);
        colour += star_colour * star_bright * star_intensity
                * night_factor * 0.8;
    }

    let ground_fade = smoothstep(-0.02, 0.02, D.y);
    let ground_tint = mix(0.03, 1.0, day_factor);
    colour = mix(ground_albedo * ground_tint, colour, ground_fade);

    colour *= exposure;

    return colour;
}
