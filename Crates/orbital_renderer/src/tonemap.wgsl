struct TonemapParams {
    exposure: f32,
    gamma: f32,
    bloom_threshold: f32,
    bloom_intensity: f32,
    contrast: f32,
    glare_stars: u32,
    glare_star_arms: u32,
    glare_star_inner: f32,
    glare_star_outer: f32,
    glare_star_length: f32,
    _padding: vec3<f32>,
};

@group(0) @binding(0) var hdr_texture: texture_2d<f32>;
@group(0) @binding(1) var hdr_sampler: sampler;
@group(0) @binding(2) var<uniform> params: TonemapParams;
@group(0) @binding(3) var scene_depth: texture_depth_2d;

struct GlareLight {
    position: vec4<f32>,
    color: vec4<f32>,
    direction: vec4<f32>,
    // x = outer cone half-angle; yzw reserved.
    params: vec4<f32>,
};

struct GlareParams {
    view_projection: mat4x4<f32>,
    camera_position: vec4<f32>,
    // xy = viewport size, z = active light count, w = glare radius.
    viewport: vec4<f32>,
    // x = near plane, y = far plane.
    camera_params: vec4<f32>,
    lights: array<GlareLight, 32>,
};

@group(0) @binding(4) var<uniform> glare: GlareParams;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) id: u32) -> VertexOutput {
    let uv = vec2<f32>(
        f32((id << 1u) & 2u),
        f32(id & 2u),
    );

    var out: VertexOutput;
    out.position = vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
    out.uv = vec2<f32>(uv.x, 1.0 - uv.y);
    return out;
}

/// A compact cross-shaped glare kernel sampled from the HDR scene before
/// tonemapping. The wider taps give light sources a visible halo without a
/// separate blur pass and keep the cost bounded for every AA mode.
fn sample_glare(uv: vec2<f32>, texel: vec2<f32>) -> vec3<f32> {
    if (params.bloom_intensity <= 0.0) {
        return vec3<f32>(0.0);
    }

    let offsets = array<vec2<f32>, 12>(
        vec2<f32>(-3.0, 0.0),
        vec2<f32>(3.0, 0.0),
        vec2<f32>(0.0, -3.0),
        vec2<f32>(0.0, 3.0),
        vec2<f32>(-2.0, -2.0),
        vec2<f32>(2.0, -2.0),
        vec2<f32>(-2.0, 2.0),
        vec2<f32>(2.0, 2.0),
        vec2<f32>(-7.0, 0.0),
        vec2<f32>(7.0, 0.0),
        vec2<f32>(0.0, -7.0),
        vec2<f32>(0.0, 7.0),
    );

    var glare = vec3<f32>(0.0);
    for (var i = 0u; i < 12u; i = i + 1u) {
        let sample_uv = clamp(uv + offsets[i] * texel, vec2<f32>(0.0), vec2<f32>(1.0));
        let sample = textureSample(hdr_texture, hdr_sampler, sample_uv).rgb;
        glare += max(sample - vec3<f32>(params.bloom_threshold), vec3<f32>(0.0));
    }

    return glare * (params.bloom_intensity / 12.0);
}

/// A soft, tapered diffraction spike. The spike is widest near the core and
/// narrows to nothing at the tip, so it reads as a blurred lens flare rather
/// than a hard geometric ray.
fn star_spike(
    delta: vec2<f32>,
    axis: vec2<f32>,
    radius: f32,
    length_scale: f32,
    inner_intensity: f32,
    outer_intensity: f32,
) -> f32 {
    let along = dot(delta, axis);
    let across = dot(delta, vec2<f32>(-axis.y, axis.x));
    let length = max(radius * params.glare_star_length * length_scale, 1.0e-6);
    let t = clamp(abs(along) / length, 0.0, 1.0);
    let width = radius * mix(0.32, 0.03, t);
    let body = exp(-(across * across) / (2.0 * max(width * width, 1.0e-10)));
    let fade = pow(1.0 - t, 2.0);
    let intensity = mix(inner_intensity, outer_intensity, t);
    return body * fade * intensity;
}

fn hash11(value: f32) -> f32 {
    return fract(sin(value * 12.9898) * 43758.5453);
}

/// Screen-space glare around positional lights. Each pixel compares its own
/// linear view depth against the light depth, so a foreground wall masks the
/// halo instead of letting it bleed through.
fn sample_light_glare(uv: vec2<f32>, pixel: vec2<i32>) -> vec3<f32> {
    let count = u32(glare.viewport.z);
    if (params.bloom_intensity <= 0.0 || count == 0u) {
        return vec3<f32>(0.0);
    }

    let viewport = glare.viewport.xy;
    let aspect = viewport.x / max(viewport.y, 1.0);
    let radius2 = glare.viewport.w * glare.viewport.w;
    let near = glare.camera_params.x;
    let far = glare.camera_params.y;
    let scene_depth_value = textureLoad(scene_depth, pixel, 0);
    let scene_linear_depth =
        (near * far) / max(far - scene_depth_value * (far - near), 1.0e-6);
    var light_glare = vec3<f32>(0.0);

    for (var i = 0u; i < count; i = i + 1u) {
        let light = glare.lights[i];
        let clip = glare.view_projection * vec4<f32>(light.position.xyz, 1.0);
        if (clip.w <= 0.0) {
            continue;
        }

        let ndc = clip.xy / clip.w;
        if (abs(ndc.x) > 1.1 || abs(ndc.y) > 1.1) {
            continue;
        }

        let to_light = light.position.xyz - glare.camera_position.xyz;
        let distance2 = dot(to_light, to_light);
        if (distance2 < 1.0e-6) {
            continue;
        }

        let light_depth = clip.w;
        let occlusion_bias = max(0.05, light_depth * 0.002);
        if (scene_linear_depth + occlusion_bias < light_depth) {
            continue;
        }

        var cone_fade = 1.0;
        if (light.direction.w > 0.5) {
            let spot_dir = light.direction.xyz;
            let spot_len2 = dot(spot_dir, spot_dir);
            if (spot_len2 < 1.0e-6) {
                continue;
            }

            let to_camera = normalize(-to_light);
            let cos_angle = dot(normalize(spot_dir), to_camera);
            let cone_cos = cos(light.params.x);
            cone_fade = smoothstep(cone_cos - 0.05, cone_cos + 0.05, cos_angle);
            if (cone_fade <= 0.0) {
                continue;
            }
        }

        let light_uv = vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);

        // Fade out light glare as the source moves away from the screen
        // center. Looking directly at a light keeps the full flare; a light
        // in the corner of vision should not contribute a visible orb.
        let center_offset = (light_uv - vec2<f32>(0.5)) * vec2<f32>(aspect, 1.0);
        // Keep a small amount of glare at the edges instead of popping to
        // zero; looking directly at the light still gets the full flare.
        let gaze = mix(
            0.08,
            1.0,
            1.0 - smoothstep(0.2, 0.55, length(center_offset)),
        );
        if (gaze <= 0.0) {
            continue;
        }

        let delta = (uv - light_uv) * vec2<f32>(aspect, 1.0);
        let distance2_screen = dot(delta, delta);
        let radius = glare.viewport.w;
        let core = exp(-distance2_screen / (radius2 * 0.33));
        let halo = exp(-distance2_screen / (radius2 * 3.0)) * 0.35;

        var star = 0.0;
        if (params.glare_stars != 0u) {
            let arms = clamp(params.glare_star_arms, 2u, 8u);
            let axis_count = arms / 2u;
            let base_angle = (hash11(light.params.y) - 0.5) * 0.5;
            let step = 3.14159265 / f32(axis_count);
            for (var a = 0u; a < axis_count; a = a + 1u) {
                let angle = base_angle + f32(a) * step;
                let axis = vec2<f32>(cos(angle), sin(angle));
                let length_scale = 1.0 - f32(a % 3u) * 0.18;
                star += star_spike(
                    delta,
                    axis,
                    radius,
                    length_scale,
                    params.glare_star_inner,
                    params.glare_star_outer,
                );
            }
        }

        let attenuation = 1.0 / (1.0 + distance2 * 0.015);
        let energy =
            light.color.w * 0.04 * attenuation * cone_fade * gaze * (core + halo + star);
        light_glare += light.color.rgb * energy;
    }

    return light_glare * params.bloom_intensity;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel = 1.0 / vec2<f32>(textureDimensions(hdr_texture));
    let base = textureSample(hdr_texture, hdr_sampler, in.uv).rgb;
    let scene_glare = sample_glare(in.uv, texel);
    let light_glare = sample_light_glare(in.uv, vec2<i32>(in.position.xy));
    let color = (base + scene_glare + light_glare) * params.exposure;

    // ACES filmic curve.
    let mapped = clamp(
        (color * (2.51 * color + 0.03)) /
        (color * (2.43 * color + 0.59) + 0.14),
        vec3<f32>(0.0),
        vec3<f32>(1.0)
    );

    // A post-tonemap contrast curve restores separation after fog and tone
    // mapping compress the scene. 1.0 is neutral.
    let contrasted = clamp(
        (mapped - vec3<f32>(0.5)) * params.contrast + vec3<f32>(0.5),
        vec3<f32>(0.0),
        vec3<f32>(1.0),
    );

    // Linear targets need the transfer function here; sRGB targets apply it
    // in hardware, so gamma is 1.0 in that case.
    let corrected = pow(contrasted, vec3<f32>(params.gamma));
    return vec4<f32>(corrected, 1.0);
}
