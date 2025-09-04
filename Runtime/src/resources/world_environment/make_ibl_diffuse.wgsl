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

// Main function to calculate diffuse IBL for a given normal
fn calculate_pbr_ibl_diffuse(N: vec3<f32>, gid: vec3<u32>) {
    var irradiance = vec3(0.0);

    // Use a reasonable number of samples for quality vs. performance
    const SAMPLE_COUNT_THETA: u32 = 64u;
    const SAMPLE_COUNT_PHI: u32 = 128u;

    // Define separate deltas for theta and phi for correct integration
    let theta_delta: f32 = (0.5 * PI) / f32(SAMPLE_COUNT_THETA);
    let phi_delta: f32 = (2.0 * PI) / f32(SAMPLE_COUNT_PHI);

    // Integrate over the hemisphere around N
    for (var i: u32 = 0u; i < SAMPLE_COUNT_PHI; i++) {
        // Phi goes from 0 to 2*PI
        let phi = (f32(i) + 0.5) * phi_delta;
        for (var j: u32 = 0u; j < SAMPLE_COUNT_THETA; j++) {
            // Theta goes from 0 to PI/2
            let theta = (f32(j) + 0.5) * theta_delta;

            // 1. Generate sample direction in tangent space
            let sin_theta = sin(theta);
            let cos_theta = cos(theta);
            let tangent_sample = vec3(
                sin_theta * cos(phi),
                sin_theta * sin(phi),
                cos_theta
            );

            // 2. Create TBN matrix to transform sample to world space
            let up = select(vec3(0.0, 0.0, 1.0), vec3(0.0, 1.0, 0.0), abs(N.y) > 0.999);
            let tangent = normalize(cross(up, N));
            let bitangent = cross(N, tangent);
            let L = normalize(
                tangent * tangent_sample.x +
                bitangent * tangent_sample.y +
                N * tangent_sample.z
            );

            // 3. Sample environment map
            let eq_uv = vec2(atan2(L.z, L.x), asin(L.y)) * INV_ATAN + 0.5;
            let src_dims = vec2<f32>(textureDimensions(src));
            let eq_pixel = vec2<i32>(eq_uv * src_dims);
            let sample = textureLoad(src, clamp(eq_pixel, vec2(0), vec2<i32>(src_dims) - 1), 0);

            // 4. Accumulate irradiance, weighted by the solid angle and cosine factor
            // The integral is L * cos(theta) * sin(theta) d(theta) d(phi)
            irradiance += sample.rgb * cos_theta * sin_theta;
        }
    }

    // 5. Normalize and apply Lambertian BRDF
    // The integral of (cos(theta) * sin(theta)) over the hemisphere is PI.
    // We multiply by the differential area and divide by PI (the BRDF) which cancels out.
    let prefiltered_color = irradiance * theta_delta * phi_delta * (1.0 / PI);

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
