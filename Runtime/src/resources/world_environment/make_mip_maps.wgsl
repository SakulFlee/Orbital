const LUMINANCE_CONVERSION: vec3<f32> = vec3<f32>(0.2126, 0.7152, 0.0722);
const ACES_A: f32 = 2.51;
const ACES_B: f32 = 0.03;
const ACES_C: f32 = 2.43;
const ACES_D: f32 = 0.59;
const ACES_E: f32 = 0.14;
const IMPORTANCE_SAMPLE_COUNT: u32 = 2048u;

@group(0) @binding(0)
var src: texture_cube<f32>; 

@group(0) @binding(1)
var src_sampler: sampler;

@group(0) @binding(2)
var dst: texture_storage_2d_array<rgba16float, write>;

@group(1) @binding(0)
var<uniform> mip_info: MipInfo;

struct MipInfo {
    mip_level: u32,
    max_mip_level: u32,
    sampling_type: u32,
    current_mip_width: u32,
    current_mip_height: u32,
}

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
                vec3(0.0, -1.0, 0.0),
                vec3(0.0, 0.0, 1.0),
                vec3(1.0, 0.0, 0.0),
            );
        }
        // FACE -Y
        case 3u: {
            return Face(
                vec3(0.0, 1.0, 0.0),
                vec3(0.0, 0.0, -1.0),
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

fn luminance(color: vec3<f32>) -> f32 {
    return dot(color, LUMINANCE_CONVERSION);
}

// ACES tone mapping
fn aces_tone_map(color: vec3<f32>) -> vec3<f32> {
    return clamp(
        (color * (ACES_A * color + ACES_B)) / 
        (color * (ACES_C * color + ACES_D) + ACES_E), 
        vec3(0.0), 
        vec3(1.0)
    );
}

fn importance_sample_ggx(Xi: vec2<f32>, roughness: f32, N: vec3<f32>) -> vec3<f32> {
    let a = roughness * roughness;
    let phi = 2.0 * 3.14159 * Xi.x;
    let cos_theta = sqrt((1.0 - Xi.y) / (1.0 + (a*a - 1.0) * Xi.y));
    let sin_theta = sqrt(1.0 - cos_theta * cos_theta);
    
    // Spherical to cartesian
    let H = vec3(
        sin_theta * cos(phi),
        sin_theta * sin(phi),
        cos_theta
    );
    
    // Tangent space to world space
    let up = select(
        select(
            vec3(0.0, 1.0, 0.0),
            vec3(1.0, 0.0, 0.0),
            abs(N.y) > 0.999
        ), 
        vec3(0.0, 1.0, 0.0), 
        abs(N.z) > 0.999
    );
    let tangent = normalize(cross(up, N));
    let bitangent = cross(N, tangent);
    
    return normalize(tangent * H.x + bitangent * H.y + N * H.z);
}

// --- DEBUG CONFIGURATION ---
// Set to true to enable debug visualization for specific conditions
const DEBUG_IMPORTANCE_SAMPLING: bool = false;
// --- END DEBUG CONFIGURATION ---

fn sample_importance(N: vec3<f32>, roughness: f32) -> vec4<f32> {
    // --- DEBUG VISUALIZATION MODE ---
    // Example: Visualize L.z for samples when N is close to +Z (0,0,1) and roughness is mid-range
    if (DEBUG_IMPORTANCE_SAMPLING && 
        abs(N.x) < 0.1 && abs(N.y) < 0.1 && abs(N.z - 1.0) < 0.1 && // N close to (0,0,1)
        roughness > 0.4 && roughness < 0.6) { // Mid-range roughness
        
        var avg_Lz = 0.0;
        var count = 0.0;
        for(var i = 0u; i < IMPORTANCE_SAMPLE_COUNT; i++) {
            let Xi = vec2(f32(i) / f32(IMPORTANCE_SAMPLE_COUNT), fract(f32(i) * 0.618034));
            let H = importance_sample_ggx(Xi, roughness, N);
            let L = normalize(2.0 * dot(N, H) * H - N);
            // Accumulate L.z to see directional bias
            avg_Lz += L.z;
            count += 1.0;
        }
        let avg_Lz_norm = avg_Lz / max(count, 1.0);
        // Map L.z from [-1, 1] to [0, 1] for visualization (blue to red)
        let viz_value = (avg_Lz_norm + 1.0) * 0.5;
        return vec4<f32>(viz_value, 0.0, 1.0 - viz_value, 1.0); // Blue to Red gradient
    }
    // --- END DEBUG MODE ---

    var result = vec4(0.0);
    var total_weight = 0.0;
    
    // Improved adaptive sampling with better distribution
    let min_samples = 512u;
    let max_samples = IMPORTANCE_SAMPLE_COUNT;
    // Use a smoother transition for sample count
    let sample_factor = smoothstep(0.0, 1.0, roughness);
    let adaptive_sample_count = min_samples + u32(f32(max_samples - min_samples) * sample_factor);
    
    // Add a small offset to roughness to avoid singularities
    let adjusted_roughness = max(roughness, 0.001);
    
    for(var i = 0u; i < adaptive_sample_count; i++) {
        // Improved sample distribution using radical inverse for better blue noise
        let epsilon = 0.0001;
        let Xi = vec2(
            fract(f32(i) / f32(adaptive_sample_count) + epsilon),
            fract(f32(i) * 0.618034 + epsilon)
        );
        let H = importance_sample_ggx(Xi, adjusted_roughness, N);
        let L = normalize(2.0 * dot(N, H) * H - N);
        
        let NdotL = max(dot(N, L), 0.0);
        if(NdotL > 0.0) {
            let sample = textureSampleLevel(src, src_sampler, L, 0.0);
            // Improved weighting to reduce variance
            let lum = luminance(sample.rgb);
            // Reduce the impact of very bright samples while preserving detail
            let lum_weight = 1.0 / (1.0 + 0.05 * lum);
            
            result += sample * (NdotL * lum_weight);
            total_weight += NdotL * lum_weight;
        }
    }
    
    // Avoid division by zero or very small numbers
    if (total_weight > 0.0001) {
        return vec4<f32>(result.rgb / total_weight, result.a / total_weight);
    } else {
        // Return a small positive value to avoid pure black
        return vec4<f32>(0.0001, 0.0001, 0.0001, 1.0);
    }
}

// Box filtering with improved artifact reduction
fn sample_filtered_box(N: vec3<f32>, mip_level: f32) -> vec4<f32> {
    var result = vec4(0.0);
    // Clamp mip_level to prevent excessive blurring
    let clamped_mip_level = clamp(mip_level, 0.0, 10.0);
    let sample_count = 4u << u32(clamped_mip_level); // Increase samples with mip level
    // Limit maximum sample count to prevent performance issues
    let limited_sample_count = min(sample_count, 64u);
    let radius = 0.001 + (clamped_mip_level * 0.005); // Increase radius with mip level
    
    for(var i = 0u; i < limited_sample_count; i++) {
        let offset = vec2(
            (f32(i % 8u) - 3.5) * radius,
            (f32(i / 8u) - 3.5) * radius
        );
        let offset_N = normalize(N + vec3(offset, 0.0));
        // Clamp offset_N to prevent sampling artifacts at cube edges
        let clamped_offset_N = clamp(offset_N, vec3(-1.0), vec3(1.0));
        result += textureSampleLevel(src, src_sampler, normalize(clamped_offset_N), 0.0);
    }
    
    // Avoid division by zero
    if (limited_sample_count > 0u) {
        return result / f32(limited_sample_count);
    } else {
        return vec4(0.0);
    }
}

// Gaussian filtering with improved artifact reduction
fn sample_filtered_gaussian(N: vec3<f32>, mip_level: f32) -> vec4<f32> {
    var result = vec4(0.0);
    var total_weight = 0.0;
    // Clamp mip_level to prevent excessive blurring
    let clamped_mip_level = clamp(mip_level, 0.0, 10.0);
    let sigma = 0.001 + (clamped_mip_level * 0.005);
    let radius = 2.0 * sigma;
    let sample_count = 4u << u32(clamped_mip_level);
    // Limit maximum sample count to prevent performance issues
    let limited_sample_count = min(sample_count, 64u);

    for(var i = 0u; i < limited_sample_count; i++) {
        let offset = vec2(
            (f32(i % 8u) - 3.5) * radius,
            (f32(i / 8u) - 3.5) * radius
        );
        let weight = exp(-(length(offset) * length(offset)) / (2.0 * sigma * sigma));
        let offset_N = normalize(N + vec3(offset, 0.0));
        // Clamp offset_N to prevent sampling artifacts at cube edges
        let clamped_offset_N = clamp(offset_N, vec3(-1.0), vec3(1.0));
        
        result += weight * textureSampleLevel(src, src_sampler, normalize(clamped_offset_N), 0.0);
        total_weight += weight;
    }
    
    // Avoid division by zero
    if (total_weight > 0.0001) {
        return result / total_weight;
    } else {
        return vec4(0.0);
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

    // Return early if outside of mip bounds
    if gid.x >= mip_info.current_mip_width || gid.y >= mip_info.current_mip_height {
        return;
    }

    // Calculate mip dimensions for UV calculation
    let mip_dimensions = vec2<f32>(f32(mip_info.current_mip_width), f32(mip_info.current_mip_height));

    // Scale UV coordinates based on mip level
    let cube_uv = vec2<f32>(gid.xy) / mip_dimensions * 2.0 - 1.0;
    let face = gid_z_to_face(gid.z);
    let N = normalize(face.forward + face.right * cube_uv.x + face.up * cube_uv.y);
    // Re-add Nmod line to compensate for coordinate system mismatch
    let Nmod = vec3(N.x, -N.y, N.z);

    // Sample based on sampling type
    var sample = vec4(0.0);
    switch mip_info.sampling_type {
        case 2u: {
            sample = sample_filtered_box(Nmod, f32(mip_info.mip_level));
            break;
        }
        case 1u: {
            sample = sample_filtered_gaussian(Nmod, f32(mip_info.mip_level));
            break;
        }
        case 0u, default: {
            // Improved roughness mapping to reduce artifacts
            let perceptual_roughness = f32(mip_info.mip_level) / f32(mip_info.max_mip_level);
            let roughness = perceptual_roughness * perceptual_roughness;
            sample = sample_importance(Nmod, roughness);
            break;
        }
    }
    textureStore(dst, gid.xy, gid.z, sample);
}
