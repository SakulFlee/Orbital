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
        case 2u: { // +Y (Corrected)
            return Face(vec3(0.0, 1.0, 0.0), vec3(0.0, 0.0, -1.0), vec3(1.0, 0.0, 0.0));
        }
        case 3u: { // -Y (Corrected)
            return Face(vec3(0.0, -1.0, 0.0), vec3(0.0, 0.0, 1.0), vec3(1.0, 0.0, 0.0));
        }
        case 4u: { // +Z
            return Face(vec3(0.0, 0.0, 1.0), vec3(0.0, 1.0, 0.0), vec3(1.0, 0.0, 0.0));
        }
        case 5u: { // -Z
            return Face(vec3(0.0, 0.0, -1.0), vec3(0.0, 1.0, 0.0), vec3(-1.0, 0.0, 0.0));
        }
        default { // Should never happen
            return Face(vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 0.0));
        }
    }
}

const PCG_MULTIPLIER: u32 = 747796405u;
const PCG_INCREMENT: u32 = 2891336453u;
fn pcg_hash(seed: u32) -> f32 {
    var state = seed;
    state = state * PCG_MULTIPLIER + PCG_INCREMENT;
    let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return f32(word >> 16u) / 65535.0; // Returns a value in [0, 1]
}

// Main function to calculate diffuse IBL for a given normal
fn calculate_pbr_ibl_diffuse(N: vec3<f32>, gid: vec3<u32>) {
    var irradiance = vec3(0.0);

    // Number of samples
    const SAMPLE_COUNT: u32 = 8192u; 

    // Use gid as a seed for the random number generator
    let seed = gid.x * 19u + gid.y * 13u + gid.z * 11u;

    // Create a TBN matrix for transforming samples
    let up = select(vec3(0.0, 0.0, 1.0), vec3(0.0, 1.0, 0.0), abs(N.y) > 0.999);
    let tangent = normalize(cross(up, N));
    let bitangent = cross(N, tangent);

    for (var i: u32 = 0u; i < SAMPLE_COUNT; i++) {
        // Generate two random numbers from our hash function
        let r1 = pcg_hash(seed + i * 3u);
        let r2 = pcg_hash(seed + i * 5u);

        // Cosine-weighted importance sampling in tangent space
        let phi = r1 * TWO_PI;
        let cos_theta = sqrt(1.0 - r2);
        let sin_theta = sqrt(r2);

        let tangent_sample = vec3(
            sin_theta * cos(phi),
            sin_theta * sin(phi),
            cos_theta
        );

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

        // Accumulate irradiance
        irradiance += sample_color.rgb;
    }

    // Correct normalization: simply average the accumulated samples.
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
        face.right * cube_uv.x - // Invert Y for texture coordinates
        face.up * cube_uv.y
    );

    // Calculate diffuse IBL
    calculate_pbr_ibl_diffuse(N, gid);
}

