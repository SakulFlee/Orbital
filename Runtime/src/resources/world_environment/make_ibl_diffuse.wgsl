const PI: f32 = 3.14159265359;
const INV_ATAN: vec2<f32> = vec2(0.1591, 0.3183); // 1/(2*PI), 1/PI
const TWO_PI: f32 = 6.28318530718;

// Bindings for textures
@group(0) @binding(0) var src: texture_2d<f32>; // Equirectangular source
@group(0) @binding(1) var dst: texture_storage_2d_array<rgba16float, write>; // Cubemap destination

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

// Radical inverse for a given integer
fn radical_inverse_vdc(bits: u32) -> f32 {
    var reversed = bits;
    reversed = (reversed & 0x55555555u) << 1u | (reversed & 0xAAAAAAAAu) >> 1u;
    reversed = (reversed & 0x33333333u) << 2u | (reversed & 0xCCCCCCCCu) >> 2u;
    reversed = (reversed & 0x0F0F0F0Fu) << 4u | (reversed & 0xF0F0F0F0u) >> 4u;
    reversed = (reversed & 0x00FF00FFu) << 8u | (reversed & 0xFF00FF00u) >> 8u;
    reversed = (reversed << 16u) | (reversed >> 16u);
    return f32(reversed) * 2.3283064365386963e-10; // 1.0 / 2^32
}

// PCG hash function for scrambling
fn pcg_hash(seed: u32) -> u32 {
    var state = seed;
    state = state * 747796405u + 2891336453u;
    return ((state >> ((state >> 28u) + 4u)) ^ state);
}

// Function to generate a scrambled Hammersley 2D point
fn hammersley2d_scrambled(i: u32, N: u32, scramble: u32) -> vec2<f32> {
    return vec2(f32(i) / f32(N), radical_inverse_vdc(i ^ scramble));
}

// Function to generate a cosine-weighted sample direction on a hemisphere
fn importance_sample_lambert(uv: vec2<f32>) -> vec3<f32> {
    let phi = uv.y * TWO_PI;
    let cos_theta = sqrt(1.0 - uv.x);
    let sin_theta = sqrt(uv.x);
    
    return vec3(
        sin_theta * cos(phi),
        sin_theta * sin(phi),
        cos_theta
    );
}

// Main function to calculate diffuse IBL for a given normal
fn calculate_pbr_ibl_diffuse(N: vec3<f32>, gid: vec3<u32>) {
    var irradiance = vec3(0.0);

    const SAMPLE_COUNT: u32 = 8192u; 

    // Get the scrambling seed from the pixel's coordinates
    let scramble_seed = pcg_hash(gid.x * 19u + gid.y * 13u + gid.z * 11u);

    // Create a TBN matrix for transforming samples
    let up = select(vec3(0.0, 0.0, 1.0), vec3(0.0, 1.0, 0.0), abs(N.y) > 0.999);
    let tangent = normalize(cross(up, N));
    let bitangent = cross(N, tangent);

    for (var i: u32 = 0u; i < SAMPLE_COUNT; i++) {
        // Generate a 2D scrambled Hammersley point
        let uv = hammersley2d_scrambled(i, SAMPLE_COUNT, scramble_seed);

        // Generate sample direction in tangent space
        let tangent_sample = importance_sample_lambert(uv);

        // Transform sample from tangent space to world space
        let L = normalize(
            tangent * tangent_sample.x +
            bitangent * tangent_sample.y +
            N * tangent_sample.z
        );
        
        // Sample environment map
        let eq_uv = vec2(atan2(L.z, L.x), asin(L.y)) * INV_ATAN + 0.5;
        let src_dims = vec2<f32>(textureDimensions(src));
        let eq_pixel = vec2<i32>(eq_uv * src_dims);
        let sample_color = textureLoad(src, clamp(eq_pixel, vec2(0), vec2<i32>(src_dims) - 1), 0);

        irradiance += sample_color.rgb;
    }

    let prefiltered_color = irradiance / f32(SAMPLE_COUNT);
    textureStore(dst, gid.xy, gid.z, vec4(prefiltered_color, 1.0));
}

// Main compute shader entry point
@compute
@workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let dst_dims_u = textureDimensions(dst);
    // Bounds check
    if (gid.x >= dst_dims_u.x || gid.y >= dst_dims_u.y) {
        return;
    }

    // Get face definition
    let face = gid_z_to_face(gid.z);

    // Calculate UV coordinates on the cubemap face (-1 to 1)
    let dst_dims_f = vec2<f32>(dst_dims_u.xy);
    let cube_uv = (vec2<f32>(gid.xy) + 0.5) / dst_dims_f * 2.0 - 1.0;

    // Calculate world space normal for this texel
    let N = normalize(
        face.forward +
        face.right * cube_uv.x - 
        face.up * cube_uv.y
    );

    // Calculate diffuse IBL
    calculate_pbr_ibl_diffuse(N, gid);
}
