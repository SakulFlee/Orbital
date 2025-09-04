const PI: f32 = 3.14159265359;
const INV_ATAN: vec2<f32> = vec2(0.1591, 0.3183); // 1/(2*PI), 1/PI

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

// Generates a single dimension of a low-discrepancy sequence (Radical Inverse base 2)
fn radical_inverse_vdc(bits: u32) -> f32 {
    var reversed_bits = reverseBits(bits);
    reversed_bits = (reversed_bits & 0x55555555u) << 1u | (reversed_bits & 0xAAAAAAAAu) >> 1u;
    reversed_bits = (reversed_bits & 0x33333333u) << 2u | (reversed_bits & 0xCCCCCCCCu) >> 2u;
    reversed_bits = (reversed_bits & 0x0F0F0F0Fu) << 4u | (reversed_bits & 0xF0F0F0F0u) >> 4u;
    reversed_bits = (reversed_bits & 0x00FF00FFu) << 8u | (reversed_bits & 0xFF00FF00u) >> 8u;
    // Scale to the [0, 1] range
    return f32(reversed_bits) * 2.3283064365386963e-10; // / 0x100000000
}

// Generates a 2D low-discrepancy Hammersley sequence point
fn hammersley(i: u32, N: u32) -> vec2<f32> {
    return vec2<f32>(f32(i) / f32(N), radical_inverse_vdc(i));
}

// Generates a cosine-weighted sample vector in tangent space for diffuse lighting
fn importance_sample_lambert(xi: vec2<f32>) -> vec3<f32> {
    let cos_theta = sqrt(1.0 - xi.y);
    let sin_theta = sqrt(xi.y); // This is sqrt(1.0 - cos_theta^2)
    let phi = 2.0 * PI * xi.x;
    let sample_vec = vec3<f32>(cos(phi) * sin_theta, sin(phi) * sin_theta, cos_theta);
    return sample_vec;
}

fn calculate_pbr_ibl_diffuse(N: vec3<f32>, gid: vec3<u32>) {
    var total_radiance = vec3(0.0);

    const SAMPLE_COUNT: u32 = 1024u; // 1024 or 2048 is a good number

    // Create the TBN matrix once before the loop
    let up = select(vec3(0.0, 0.0, 1.0), vec3(0.0, 1.0, 0.0), abs(N.y) > 0.999);
    let tangent = normalize(cross(up, N));
    let bitangent = cross(N, tangent);

    for (var i: u32 = 0u; i < SAMPLE_COUNT; i++) {
        // 1. Get a low-discrepancy point on the unit square
        let xi = hammersley(i, SAMPLE_COUNT);

        // 2. Generate a cosine-weighted sample direction in tangent space
        let tangent_sample = importance_sample_lambert(xi);

        // 3. Transform sample to world space
        let L = normalize(
            tangent * tangent_sample.x +
            bitangent * tangent_sample.y +
            N * tangent_sample.z
        );

        // 4. Sample environment map
        let eq_uv = vec2(atan2(L.z, L.x), asin(L.y)) * INV_ATAN + 0.5;
        let src_dims = vec2<f32>(textureDimensions(src));
        let eq_pixel = vec2<i32>(eq_uv * src_dims);
        let sample_color = textureLoad(src, clamp(eq_pixel, vec2(0), vec2<i32>(src_dims) - 1), 0).rgb;

        // 5. Accumulate radiance.
        // With importance sampling, the PDF cancels out the cosine term,
        // so we just sum the sampled colors.
        total_radiance += sample_color;
    }

    // 6. The final irradiance is the average radiance scaled by the Lambertian BRDF (1/PI)
    let irradiance = total_radiance / f32(SAMPLE_COUNT);
    let prefiltered_color = irradiance; // The 1/PI factor is applied in the main render shader, not here.

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
