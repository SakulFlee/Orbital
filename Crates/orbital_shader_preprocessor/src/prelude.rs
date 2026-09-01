//! The engine's built-in library of reusable shader nodes.
//!
//! Each WGSL function / constant / struct is exposed as one named
//! [`crate::ShaderNode`], so shaders can be assembled by name and the pieces
//! are shared across all shaders. Nodes are extracted from the existing shader
//! files; during the shader-port task these sources become the canonical
//! building blocks that the full shaders reference.

use crate::NodeLibrary;

/// Returns the engine's built-in node library, preloaded into the global
/// [`crate::NodeRegistry`].
pub fn prelude_library() -> NodeLibrary {
    let mut lib = NodeLibrary::new("orbital_prelude");

    // --- Constants ---------------------------------------------------------
    lib.add(crate::ShaderNode::new(
        "pi",
        "const PI: f32 = 3.14159265359;\nconst TWO_PI: f32 = 6.28318530718;\n",
    ));
    lib.add(crate::ShaderNode::new(
        "f0_default",
        "const F0_DEFAULT: f32 = 0.04;\n",
    ));
    lib.add(crate::ShaderNode::new(
        "ambient_intensity",
        "const AMBIENT_INTENSITY: f32 = 0.2;\n",
    ));
    lib.add(crate::ShaderNode::new(
        "light_types",
        "const LIGHT_TYPE_POINT: f32 = 0.0;\n\
         const LIGHT_TYPE_DIRECTIONAL: f32 = 1.0;\n\
         const LIGHT_TYPE_SPOT: f32 = 2.0;\n",
    ));
    lib.add(crate::ShaderNode::new(
        "shadow_types",
        "const MAX_SHADOW_SLOTS: u32 = 16u;\n\
         const SHADOW_TYPE_DIRECTIONAL_CASCADE: u32 = 0u;\n\
         const SHADOW_TYPE_SPOT: u32 = 1u;\n\
         const SHADOW_TYPE_POINT: u32 = 2u;\n",
    ));
    lib.add(crate::ShaderNode::new(
        "aces_constants",
        "const ACES_A: f32 = 2.51;\n\
         const ACES_B: f32 = 0.03;\n\
         const ACES_C: f32 = 2.43;\n\
         const ACES_D: f32 = 0.59;\n\
         const ACES_E: f32 = 0.14;\n",
    ));

    // --- Structures --------------------------------------------------------
    lib.add(crate::ShaderNode::new(
        "camera_uniform",
        "struct CameraUniform {\n\
                 position: vec3<f32>,\n\
                 view_projection_matrix: mat4x4<f32>,\n\
                 perspective_view_projection_matrix: mat4x4<f32>,\n\
                 view_projection_transposed: mat4x4<f32>,\n\
                 perspective_projection_invert: mat4x4<f32>,\n\
                 global_gamma: f32,\n\
             }\n",
    ));
    lib.add(crate::ShaderNode::new(
        "light_struct",
        "struct Light {\n\
                 position: vec4<f32>,     // xyz: position, w: padding\n\
                 color: vec4<f32>,        // xyz: color, w: intensity\n\
                 direction: vec4<f32>,    // xyz: direction, w: type\n\
                 params: vec4<f32>,       // x: inner cone, y: outer cone, zw: padding\n\
             }\n",
    ));
    lib.add(crate::ShaderNode::new(
        "vertex_data_simple",
        "struct VertexData {\n\
                 @builtin(vertex_index) vertex_index: u32,\n\
                 @location(0) position: vec3<f32>,\n\
             }\n",
    ));
    lib.add(crate::ShaderNode::new(
        "instance_data",
        "struct InstanceData {\n\
                 @location(5) model_space_matrix_0: vec4<f32>,\n\
                 @location(6) model_space_matrix_1: vec4<f32>,\n\
                 @location(7) model_space_matrix_2: vec4<f32>,\n\
                 @location(8) model_space_matrix_3: vec4<f32>,\n\
             }\n",
    ));
    lib.add(crate::ShaderNode::new(
        "fragment_data",
        "struct FragmentData {\n\
                 @builtin(position) position: vec4<f32>,\n\
                 @location(0) world_position: vec3<f32>,\n\
                 @location(1) uv: vec2<f32>,\n\
                 @location(2) tangent: vec3<f32>,\n\
                 @location(3) bitangent: vec3<f32>,\n\
                 @location(4) normal: vec3<f32>,\n\
             }\n",
    ));
    lib.add(crate::ShaderNode::new(
        "shadow_slot",
        "struct ShadowSlot {\n\
                 light_view_proj: mat4x4<f32>,\n\
                 shadow_type: u32,\n\
                 layer_index: u32,\n\
                 cascade_split_depth: f32,\n\
                 bias: f32,\n\
                 light_index: u32,\n\
                 near_plane: f32,\n\
             }\n\
             \n\
             struct ShadowData {\n\
                 slots: array<ShadowSlot, 16>,\n\
                 cascade_count: u32,\n\
             }\n",
    ));
    lib.add(
        crate::ShaderNode::new(
            "cubemap_face",
            "struct Face {\n\
                 forward: vec3<f32>,\n\
                 up: vec3<f32>,\n\
                 right: vec3<f32>,\n\
             }\n\
             \n\
             fn gid_z_to_face(gid_z: u32) -> Face {\n\
                 switch gid_z {\n\
                     case 0u: { return Face(vec3(1.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0), vec3(0.0, 0.0, -1.0)); }\n\
                     case 1u: { return Face(vec3(-1.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0), vec3(0.0, 0.0, 1.0)); }\n\
                     case 2u: { return Face(vec3(0.0, 1.0, 0.0), vec3(0.0, 0.0, -1.0), vec3(1.0, 0.0, 0.0)); }\n\
                     case 3u: { return Face(vec3(0.0, -1.0, 0.0), vec3(0.0, 0.0, 1.0), vec3(1.0, 0.0, 0.0)); }\n\
                     case 4u: { return Face(vec3(0.0, 0.0, 1.0), vec3(0.0, 1.0, 0.0), vec3(1.0, 0.0, 0.0)); }\n\
                     case 5u: { return Face(vec3(0.0, 0.0, -1.0), vec3(0.0, 1.0, 0.0), vec3(-1.0, 0.0, 0.0)); }\n\
                     default { return Face(vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 0.0)); }\n\
                 }\n\
             }\n",
        ),
    );

    // --- Functions (one node each) ----------------------------------------
    lib.add(
        crate::ShaderNode::new(
            "aces_tone_map",
            "fn aces_tone_map(color: vec3<f32>) -> vec3<f32> {\n\
                 return clamp(\n\
                     (color * (ACES_A * color + ACES_B)) /\n\
                     (color * (ACES_C * color + ACES_D) + ACES_E),\n\
                     vec3(0.0),\n\
                     vec3(1.0)\n\
                 );\n\
             }\n",
        )
        .with_deps(["aces_constants"]),
    );
    lib.add(
        crate::ShaderNode::new(
            "fresnel_schlick",
            "fn fresnel_schlick(cos_theta: f32, F0: vec3<f32>) -> vec3<f32> {\n\
                 return F0 + (1.0 - F0) * pow(1.0 - cos_theta, 5.0);\n\
             }\n",
        ),
    );
    lib.add(crate::ShaderNode::new("fresnel_schlick_roughness",
        "fn fresnel_schlick_roughness(cos_theta: f32, F0: vec3<f32>, roughness: f32) -> vec3<f32> {\n\
             return F0 + (max(vec3<f32>(1.0 - roughness), F0) - F0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);\n\
         }\n",
    ));
    lib.add(
        crate::ShaderNode::new(
            "distribution_ggx",
            "fn distribution_ggx(NdotH: f32, roughness: f32) -> f32 {\n\
                 let alpha = roughness * roughness;\n\
                 let alpha_squared = alpha * alpha;\n\
                 let denom = (NdotH * NdotH) * (alpha_squared - 1.0) + 1.0;\n\
                 return alpha_squared / (PI * denom);\n\
             }\n",
        )
        .with_deps(["pi"]),
    );
    lib.add(crate::ShaderNode::new(
        "schlick_smith_ggx",
        "fn schlick_smith_ggx(NdotL: f32, NdotV: f32, roughness: f32) -> f32 {\n\
             let k = (roughness * roughness) / 2.0;\n\
             let GL = NdotL / (NdotL * (1.0 - k) + k);\n\
             let GV = NdotV / (NdotV * (1.0 - k) + k);\n\
             return GL * GV;\n\
         }\n",
    ));
    lib.add(crate::ShaderNode::new(
        "slope_scaled_bias",
        "fn slope_scaled_bias(base_bias: f32, n_dot_l: f32) -> f32 {\n\
             return base_bias * (1.0 + 0.5 * (1.0 - clamp(n_dot_l, 0.0, 1.0)));\n\
         }\n",
    ));
    lib.add(crate::ShaderNode::new(
        "star_hash",
        "fn star_hash(cx: u32, cy: u32, cz: u32) -> u32 {\n\
             var h = cx * 374761393u + cy * 668265263u + cz * 1274126177u;\n\
             h = (h ^ (h >> 13u)) * 1103515245u;\n\
             h = h ^ (h >> 16u);\n\
             return h;\n\
         }\n",
    ));
    lib.add(
        crate::ShaderNode::new(
            "fib_direction",
            "fn fib_direction(i: u32, n: u32) -> vec3<f32> {\n\
                 const GOLDEN_ANGLE: f32 = 2.399963229728653;\n\
                 let phi = f32(i) * GOLDEN_ANGLE;\n\
                 let y = 1.0 - (f32(i) + 0.5) / f32(n) * 2.0;\n\
                 let r = sqrt(max(0.0, 1.0 - y * y));\n\
                 return vec3<f32>(cos(phi) * r, y, sin(phi) * r);\n\
             }\n",
        )
        .with_deps(["pi"]),
    );
    lib.add(
        crate::ShaderNode::new(
            "disk_irradiance",
            "fn disk_irradiance(\n\
                 disk_dir: vec3<f32>,\n\
                 radius: f32,\n\
                 radiance: vec3<f32>,\n\
                 N: vec3<f32>,\n\
             ) -> vec3<f32> {\n\
                 let cos_n = max(dot(N, disk_dir), 0.0);\n\
                 if cos_n <= 0.0 {\n\
                     return vec3<f32>(0.0);\n\
                 }\n\
                 let solid_angle = TWO_PI * (1.0 - cos(radius));\n\
                 return radiance * solid_angle * cos_n;\n\
             }\n",
        )
        .with_deps(["pi"]),
    );
    lib.add(crate::ShaderNode::new(
        "radical_inverse_vdc",
        "fn radical_inverse_vdc(bits: u32) -> f32 {\n\
             var reversed = bits;\n\
             reversed = (reversed & 0x55555555u) << 1u | (reversed & 0xAAAAAAAAu) >> 1u;\n\
             reversed = (reversed & 0x33333333u) << 2u | (reversed & 0xCCCCCCCCu) >> 2u;\n\
             reversed = (reversed & 0x0F0F0F0Fu) << 4u | (reversed & 0xF0F0F0F0u) >> 4u;\n\
             reversed = (reversed & 0x00FF00FFu) << 8u | (reversed & 0xFF00FF00u) >> 8u;\n\
             reversed = (reversed << 16u) | (reversed >> 16u);\n\
             return f32(reversed) * 2.3283064365386963e-10;\n\
         }\n",
    ));
    lib.add(crate::ShaderNode::new(
        "pcg_hash",
        "fn pcg_hash(seed: u32) -> u32 {\n\
             var state = seed;\n\
             state = state * 747796405u + 2891336453u;\n\
             return ((state >> ((state >> 28u) + 4u)) ^ state);\n\
         }\n",
    ));
    lib.add(
        crate::ShaderNode::new(
            "hammersley2d_scrambled",
            "fn hammersley2d_scrambled(i: u32, N: u32, scramble: u32) -> vec2<f32> {\n\
                 return vec2(f32(i) / f32(N), radical_inverse_vdc(i ^ scramble));\n\
             }\n",
        )
        .with_deps(["radical_inverse_vdc"]),
    );
    lib.add(
        crate::ShaderNode::new(
            "importance_sample_lambert",
            "fn importance_sample_lambert(uv: vec2<f32>) -> vec3<f32> {\n\
                 let phi = uv.y * TWO_PI;\n\
                 let cos_theta = sqrt(1.0 - uv.x);\n\
                 let sin_theta = sqrt(uv.x);\n\
                 return vec3(sin_theta * cos(phi), sin_theta * sin(phi), cos_theta);\n\
             }\n",
        )
        .with_deps(["pi"]),
    );
    lib.add(crate::ShaderNode::new(
        "radical_inverse",
        "fn radical_inverse(i: u32) -> f32 {\n\
                 var bits: u32 = (i << 16u) | (i >> 16u);\n\
                 bits = ((bits & 0x55555555u) << 1u) | ((bits & 0xAAAAAAAAu) >> 1u);\n\
                 bits = ((bits & 0x33333333u) << 2u) | ((bits & 0xCCCCCCCCu) >> 2u);\n\
                 bits = ((bits & 0x0F0F0F0Fu) << 4u) | ((bits & 0xF0F0F0F0u) >> 4u);\n\
                 bits = ((bits & 0x00FF00FFu) << 8u) | ((bits & 0xFF00FF00u) >> 8u);\n\
                 return f32(bits) * 2.3283064365386963e-10;\n\
             }\n",
    ));
    lib.add(
        crate::ShaderNode::new(
            "hammersley",
            "fn hammersley(i: u32, N: u32) -> vec2<f32> {\n\
                 let radical_inverse = radical_inverse(i);\n\
                 return vec2<f32>(f32(i) / f32(N), radical_inverse);\n\
             }\n",
        )
        .with_deps(["radical_inverse"]),
    );
    lib.add(
        crate::ShaderNode::new("importance_sample_ggx_ibl",
            "fn importance_sample_ggx(Xi: vec2<f32>, N: vec3<f32>, roughness: f32) -> vec3<f32> {\n\
                 let alpha = roughness * roughness;\n\
                 let phi = 2.0 * PI * Xi.x;\n\
                 let cosTheta = sqrt((1.0 - Xi.y) / (1.0 + (alpha * alpha - 1.0) * Xi.y));\n\
                 let sinTheta = sqrt(1.0 - cosTheta * cosTheta);\n\
                 var H = vec3<f32>(cos(phi) * sinTheta, sin(phi) * sinTheta, cosTheta);\n\
                 var up: vec3<f32> = select(vec3(1.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0), abs(N.z) < 0.999);\n\
                 let tangent = normalize(cross(up, N));\n\
                 let bitangent = cross(N, tangent);\n\
                 return normalize(tangent * H.x + bitangent * H.y + N * H.z);\n\
             }\n",
        )
        .with_deps(["pi"]),
    );
    lib.add(
        crate::ShaderNode::new("importance_sample_ggx_mip",
            "fn importance_sample_ggx(Xi: vec2<f32>, roughness: f32, N: vec3<f32>) -> vec3<f32> {\n\
                 let a = roughness * roughness;\n\
                 let phi = 2.0 * 3.14159 * Xi.x;\n\
                 let cos_theta = sqrt((1.0 - Xi.y) / (1.0 + (a*a - 1.0) * Xi.y));\n\
                 let sin_theta = sqrt(1.0 - cos_theta * cos_theta);\n\
                 let H = vec3(sin_theta * cos(phi), sin_theta * sin(phi), cos_theta);\n\
                 let up = select(select(vec3(0.0, 1.0, 0.0), vec3(1.0, 0.0, 0.0), abs(N.y) > 0.999), vec3(0.0, 1.0, 0.0), abs(N.z) > 0.999);\n\
                 let tangent = normalize(cross(up, N));\n\
                 let bitangent = cross(N, tangent);\n\
                 return normalize(tangent * H.x + bitangent * H.y + N * H.z);\n\
             }\n",
        ),
    );
    lib.add(crate::ShaderNode::new(
        "luminance",
        "const LUMINANCE_CONVERSION: vec3<f32> = vec3<f32>(0.2126, 0.7152, 0.0722);\n\
         fn luminance(color: vec3<f32>) -> f32 {\n\
             return dot(color, LUMINANCE_CONVERSION);\n\
         }\n",
    ));

    // --- Vertex / fragment output layouts ----------------------------------
    lib.add(crate::ShaderNode::new(
        "vertex_output",
        "struct VertexOutput {\n\
             @builtin(position) frag_position: vec4<f32>,\n\
             @location(0) clip_position: vec4<f32>,\n\
         }\n",
    ));
    lib.add(crate::ShaderNode::new(
        "vertex_data_complex",
        "struct VertexData {\n\
             @builtin(vertex_index) vertex_index: u32,\n\
             @location(0) position: vec3<f32>,\n\
             @location(1) normal: vec3<f32>,\n\
             @location(2) tangent: vec3<f32>,\n\
             @location(3) bitangent: vec3<f32>,\n\
             @location(4) uv: vec2<f32>,\n\
         }\n",
    ));

    // --- Sky ----------------------------------------------------------------
    lib.add(crate::ShaderNode::new(
        "sky_params",
        "struct SkyParams {\n\
             sun_direction: vec3<f32>,\n\
             _pad0: f32,\n\
             sun_angular_radius: f32,\n\
             sun_intensity: f32,\n\
             moon_angular_radius: f32,\n\
             moon_intensity: f32,\n\
             star_intensity: f32,\n\
             star_density: f32,\n\
             exposure: f32,\n\
             _pad1: f32,\n\
             ground_albedo: vec3<f32>,\n\
             _pad2: f32,\n\
             day_zenith: vec3<f32>,\n\
             day_horizon: vec3<f32>,\n\
             night_zenith: vec3<f32>,\n\
             night_horizon: vec3<f32>,\n\
             twilight: vec3<f32>,\n\
             sun_color: vec3<f32>,\n\
             moon_color: vec3<f32>,\n\
         }\n",
    ));
    lib.add(
        crate::ShaderNode::new(
            "sky_color",
            "fn sky_color(D: vec3<f32>, params: SkyParams) -> vec3<f32> {\n\
                 let sun_dir = params.sun_direction;\n\
                 let sun_elev = sun_dir.y;\n\
                 let day_factor = smoothstep(-0.1, 0.25, sun_elev);\n\
                 let night_factor = 1.0 - day_factor;\n\
                 let zenith = mix(params.night_zenith, params.day_zenith, day_factor);\n\
                 let horizon = mix(params.night_horizon, params.day_horizon, day_factor);\n\
                 let height = clamp(D.y, 0.0, 1.0);\n\
                 var colour = mix(horizon, zenith, pow(height, 0.3));\n\
                 let twilight = exp(-abs(sun_elev) * 5.0);\n\
                 let twilight_visible = smoothstep(-0.15, 0.0, sun_elev);\n\
                 let horizon_term = pow(1.0 - height, 2.0);\n\
                 colour += params.twilight * twilight * horizon_term * twilight_visible;\n\
                 let cos_a = clamp(dot(D, sun_dir), -1.0, 1.0);\n\
                 let sun_ang = acos(cos_a);\n\
                 let sun_visible = smoothstep(-0.05, 0.05, sun_elev);\n\
                 let core_sigma = params.sun_angular_radius;\n\
                 let disk = exp(-0.5 * sun_ang * sun_ang / (core_sigma * core_sigma));\n\
                 colour += params.sun_color * params.sun_intensity * disk * sun_visible;\n\
                 let halo_sigma = params.sun_angular_radius * 2.5;\n\
                 let halo = exp(-0.5 * sun_ang * sun_ang / (halo_sigma * halo_sigma));\n\
                 colour += params.twilight * params.sun_intensity * 0.1 * halo * sun_visible;\n\
                 let moon_dir = -sun_dir;\n\
                 let moon_ang = acos(clamp(dot(D, moon_dir), -1.0, 1.0));\n\
                 let moon_visible = 1.0 - smoothstep(-0.05, 0.05, sun_elev);\n\
                 let moon_sigma = params.moon_angular_radius;\n\
                 let moon_disk = exp(-pow(moon_ang / moon_sigma, 8.0));\n\
                 colour += params.moon_color * params.moon_intensity * moon_disk * moon_visible;\n\
                 let moon_halo_sigma = moon_sigma * 1.5;\n\
                 let moon_halo = exp(-moon_ang * moon_ang / (2.0 * moon_halo_sigma * moon_halo_sigma));\n\
                 colour += params.moon_color * params.moon_intensity * 0.1 * moon_halo * moon_visible;\n\
                 const STAR_GRID: f32 = 90.0;\n\
                 let bias = i32(STAR_GRID);\n\
                 let cell = vec3<i32>(floor(D * STAR_GRID)) + vec3<i32>(bias);\n\
                 let h = star_hash(u32(cell.x), u32(cell.y), u32(cell.z));\n\
                 let bright = f32(h & 0xFFFFu) / 65535.0;\n\
                 let threshold = 1.0 - clamp(params.star_density, 0.0, 1.0);\n\
                 if bright > threshold {\n\
                     let off = star_hash(u32(cell.x) + 0x9E3779B9u, u32(cell.y) + 0x85EBCA6Bu, u32(cell.z) + 0xC2B2AE35u);\n\
                     let ox = f32(off & 0xFFu) / 255.0 - 0.5;\n\
                     let oy = f32((off >> 8u) & 0xFFu) / 255.0 - 0.5;\n\
                     let oz = f32((off >> 16u) & 0xFFu) / 255.0 - 0.5;\n\
                     let centre = (vec3<f32>(cell) - vec3<f32>(f32(bias))) + vec3<f32>(ox, oy, oz);\n\
                     let dist = length(D * STAR_GRID - centre);\n\
                     let star_val = exp(-dist * dist * 30.0);\n\
                     let star_bright = (bright - threshold) / (1.0 - threshold) * star_val;\n\
                     let star_colour = vec3<f32>(0.85, 0.9, 1.0);\n\
                     colour += star_colour * star_bright * params.star_intensity * night_factor * 0.8;\n\
                 }\n\
                 let ground_fade = smoothstep(-0.02, 0.02, D.y);\n\
                 let ground_tint = mix(0.03, 1.0, day_factor);\n\
                 colour = mix(params.ground_albedo * ground_tint, colour, ground_fade);\n\
                 colour *= params.exposure;\n\
                 return colour;\n\
             }\n",
        )
        .with_deps(["sky_params", "star_hash"]),
    );

    lib
}
