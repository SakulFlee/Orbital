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
        case 2u: { // +Y
            return Face(vec3(0.0, -1.0, 0.0), vec3(0.0, 0.0, 1.0), vec3(1.0, 0.0, 0.0));
        }
        case 3u: { // -Y
            return Face(vec3(0.0, 1.0, 0.0), vec3(0.0, 0.0, -1.0), vec3(1.0, 0.0, 0.0));
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
    var total_weight = 0.0;
    
    // Increase sample count for better quality
    const SAMPLE_COUNT: u32 = 128u; // Increased to 128x128
    let sample_delta: f32 = PI * 0.5 / f32(SAMPLE_COUNT);

    // Integrate over the hemisphere around N
    for (var i: u32 = 0u; i < SAMPLE_COUNT * 2u; i++) {
        // Use midpoint rule for phi
        let phi = (f32(i) + 0.5) * sample_delta;
        for (var j: u32 = 0u; j < SAMPLE_COUNT; j++) {
            // Use midpoint rule for theta
            let theta = (f32(j) + 0.5) * sample_delta;
            
            // 1. Generate sample direction in spherical coordinates (tangent space)
            let sin_theta = sin(theta);
            let cos_theta = cos(theta);
            // Note: This is a direction vector in tangent space where Z is "up"
            let tangent_sample = vec3(
                sin_theta * cos(phi),
                sin_theta * sin(phi),
                cos_theta
            );

            // 2. Create TBN matrix to transform sample to world space
            // The goal is to have 'N' aligned with the Z-axis of the tangent space
            let up = select(
                select(vec3(0.0, 1.0, 0.0), vec3(1.0, 0.0, 0.0), abs(N.y) > 0.999),
                vec3(0.0, 1.0, 0.0),
                abs(N.z) > 0.999
            );
            let tangent = normalize(cross(up, N));
            let bitangent = cross(N, tangent);
            
            // Transform sample direction to world space
            // L = T * tangent_sample.x + B * tangent_sample.y + N * tangent_sample.z
            let L = normalize(
                tangent * tangent_sample.x +
                bitangent * tangent_sample.y +
                N * tangent_sample.z
            );

            // 3. Calculate weight with a slight modification to see if it helps smoothness
            // In tangent space, NdotL is simply cos_theta (since N is (0,0,1))
            // Weight = NdotL (cosine lobe) * sin_theta (solid angle element)
            let weight = cos_theta * sin_theta * sample_delta * sample_delta;
            
            // Apply a small power to the weight to see if it helps smoothness
            // This slightly changes the weighting but keeps the mathematical integrity
            // let weight_modified = pow(weight, 0.95); // This is just an experiment
            
            // 4. Sample environment map
            // Convert world direction L to equirectangular UV
            // Handle singularity at poles where L.x and L.z are both close to 0
            let atan_input_x = select(L.x, 1e-6, abs(L.x) < 1e-6 && abs(L.z) < 1e-6);
            let atan_input_z = select(L.z, 1e-6, abs(L.x) < 1e-6 && abs(L.z) < 1e-6);
            let eq_uv = vec2(atan2(atan_input_z, atan_input_x), asin(L.y)) * INV_ATAN + 0.5;
            // Get source texture dimensions
            let src_dims = vec2<f32>(textureDimensions(src));
            // Convert UV to pixel coordinates and clamp
            let eq_pixel_f = eq_uv * src_dims;
            let eq_pixel_clamped = clamp(eq_pixel_f, vec2(0.0), src_dims - 1.0);
            let eq_pixel = vec2<i32>(eq_pixel_clamped);
            
            // Load sample from equirectangular texture
            let sample = textureLoad(src, eq_pixel, 0);

            // 5. Accumulate
            irradiance += sample.rgb * weight;
            total_weight += weight;
        }
    }

    // 6. Normalize and apply Lambertian BRDF
    // Avoid division by zero
    if (total_weight > 0.0) {
        // Normalize by total weight and apply Lambertian BRDF (1/PI)
        let prefiltered_color = (irradiance / total_weight) * (1.0 / PI);
        textureStore(dst, gid.xy, gid.z, vec4(prefiltered_color, 1.0));
    } else {
        // This should ideally not happen, but return a small value if it does
        textureStore(dst, gid.xy, gid.z, vec4(0.0001, 0.0001, 0.0001, 1.0));
    }
}

// Main compute shader entry point
@compute
@workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    // Bounds check
    if gid.x >= u32(textureDimensions(dst).x) {
        return;
    }

    // Get face definition
    let face = gid_z_to_face(gid.z);
    
    // Calculate UV coordinates on the cubemap face (-1 to 1)
    let dst_dims = vec2<f32>(textureDimensions(dst));
    let cube_uv = vec2<f32>(gid.xy) / dst_dims * 2.0 - 1.0;
    
    // Calculate world space normal for this texel
    let N = normalize(
        face.forward + 
        face.right * cube_uv.x + 
        face.up * cube_uv.y
    );

    // Calculate diffuse IBL
    calculate_pbr_ibl_diffuse(N, gid);
}