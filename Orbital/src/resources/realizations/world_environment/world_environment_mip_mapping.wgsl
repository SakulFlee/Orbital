const LUMINANCE_CONVERSION: vec3<f32> = vec3<f32>(0.2126, 0.7152, 0.0722);
const ACES_A: f32 = 2.51;
const ACES_B: f32 = 0.03;
const ACES_C: f32 = 2.43;
const ACES_D: f32 = 0.59;
const ACES_E: f32 = 0.14;
const IMPORTANCE_SAMPLE_COUNT: u32 = 1024u;

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

fn sample_importance(N: vec3<f32>, roughness: f32) -> vec4<f32> {
    var result = vec4(0.0);
    var total_weight = 0.0;
    
    for(var i = 0u; i < IMPORTANCE_SAMPLE_COUNT; i++) {
        let Xi = vec2(f32(i) / f32(IMPORTANCE_SAMPLE_COUNT), fract(f32(i) * 0.618034));
        let H = importance_sample_ggx(Xi, roughness, N);
        let L = normalize(2.0 * dot(N, H) * H - N);
        
        let NdotL = max(dot(N, L), 0.0);
        if(NdotL > 0.0) {
            let sample = textureSampleLevel(src, src_sampler, L, 0.0);
            let lum_weight = 1.0 / (1.0 + luminance(sample.rgb));
            
            result += sample * (NdotL * lum_weight);
            total_weight += NdotL * lum_weight;
        }
    }
    
    return vec4(aces_tone_map(result.rgb / total_weight), result.a / total_weight);
}

// Box filtering
fn sample_filtered_box(N: vec3<f32>, mip_level: f32) -> vec4<f32> {
    var result = vec4(0.0);
    let sample_count = 4u << u32(mip_level); // Increase samples with mip level
    let radius = 0.001 + (mip_level * 0.005); // Increase radius with mip level
    
    for(var i = 0u; i < sample_count; i++) {
        let offset = vec2(
            (f32(i % 4u) - 1.5) * radius,
            (f32(i / 4u) - 1.5) * radius
        );
        let offset_N = normalize(N + vec3(offset, 0.0));
        result += textureSampleLevel(src, src_sampler, offset_N, 0.0);
    }
    
    return result / f32(sample_count);
}

// Gaussian filtering
fn sample_filtered_gaussian(N: vec3<f32>, mip_level: f32) -> vec4<f32> {
    var result = vec4(0.0);
    var total_weight = 0.0;
    let sigma = 0.001 + (mip_level * 0.005);
    let radius = 2.0 * sigma;
    let sample_count = 4u << u32(mip_level);

    for(var i = 0u; i < sample_count; i++) {
        let offset = vec2(
            (f32(i % 4u) - 1.5) * radius,
            (f32(i / 4u) - 1.5) * radius
        );
        let weight = exp(-(length(offset) * length(offset)) / (2.0 * sigma * sigma));
        let offset_N = normalize(N + vec3(offset, 0.0));
        
        result += weight * textureSampleLevel(src, src_sampler, offset_N, 0.0);
        total_weight += weight;
    }
    
    return result / total_weight;
}

@compute
@workgroup_size(16, 16, 1)
fn main(
    @builtin(global_invocation_id)
    gid: vec3<u32>,
) {
    let src_dimensions = vec2<f32>(textureDimensions(src));
    let dst_dimensions = vec2<f32>(textureDimensions(dst));
    let mip_dimensions = vec2<f32>(dst_dimensions / pow(2.0, f32(mip_info.mip_level)));

    // Return early if outside of mip bounds
    if gid.x >= u32(mip_dimensions.x) || gid.y >= u32(mip_dimensions.y) {
        return;
    }

        // Scale UV coordinates based on mip level
    let cube_uv = vec2<f32>(gid.xy) / mip_dimensions * 2.0 - 1.0;
    let face = gid_z_to_face(gid.z);
    let N = normalize(face.forward + face.right * cube_uv.x + face.up * cube_uv.y);
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
            sample = sample_importance(Nmod, f32(mip_info.mip_level) / f32(mip_info.max_mip_level));
            break;
        }
    }
    textureStore(dst, gid.xy, gid.z, sample);
}
