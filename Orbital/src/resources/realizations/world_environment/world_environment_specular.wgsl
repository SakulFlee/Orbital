const PI: f32 = 3.1415926535897932384626433832795;
const BASE_SAMPLES: u32 = 1024u;
const INV_ATAN = vec2<f32>(0.1591, 0.3183);

struct Face {
    forward: vec3<f32>,
    up: vec3<f32>,
    right: vec3<f32>,
}

@group(0) @binding(0)
var src: texture_2d<f32>;

@group(0) @binding(1)
var dst: texture_storage_2d_array<rgba16float, write>;

@group(0) @binding(2)
var<uniform> roughness: f32;

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

fn radical_inverse_vdc(bitsI: u32) -> f32 {
    var bits = bitsI;
    bits = (bits << 16u) | (bits >> 16u);
    bits = ((bits & 0x55555555u) << 1u) | ((bits & 0xAAAAAAAAu) >> 1u);
    bits = ((bits & 0x33333333u) << 2u) | ((bits & 0xCCCCCCCCu) >> 2u);
    bits = ((bits & 0x0F0F0F0Fu) << 4u) | ((bits & 0xF0F0F0F0u) >> 4u);
    bits = ((bits & 0x00FF00FFu) << 8u) | ((bits & 0xFF00FF00u) >> 8u);
    return f32(bits) * 2.3283064365386963e-10; // / 0x100000000
}

fn hammersley(i: u32, N: u32) -> vec2<f32> {
    return vec2(f32(i)/f32(N), radical_inverse_vdc(i));
}

fn importance_sample_ggx(Xi: vec2<f32>, N: vec3<f32>) -> vec3<f32> {
    let a = roughness * roughness;
    let phi = 2.0 * PI * Xi.x;
    let cosTheta = sqrt((1.0 - Xi.y) / (1.0 + (a*a - 1.0) * Xi.y));
    let sinTheta = sqrt(1.0 - cosTheta * cosTheta);

    // Spherical to cartesian
    let H = vec3(
        sinTheta * cos(phi),
        sinTheta * sin(phi),
        cosTheta
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

    let sample_vec = tangent * H.x + bitangent * H.y + N * H.z;
    return normalize(sample_vec);
}

fn distribution_ggx(NdotH: f32) -> f32 {
    let alpha = roughness * roughness;
    let alpha_squared = alpha * alpha;

    let denom = (NdotH * NdotH) * (alpha_squared - 1.0) + 1.0;
    return alpha_squared / (PI * denom * denom);
}

fn calculate_pbr_ibl_specular(N: vec3<f32>, gid: vec3<u32>, V: vec3<f32>) {
    var prefiltered_color = vec3(0.0);
    var total_weight = 0.0;

    let src_dimensions = vec2<f32>(textureDimensions(src));

    for(var i = 1u; i <= BASE_SAMPLES; i++) {
        let Xi = hammersley(i, BASE_SAMPLES);
        let H = importance_sample_ggx(Xi, N);
        let L = normalize(2.0 * dot(N, H) * H - N);

        let NdotL = max(dot(N, L), 0.0);
        if(NdotL > 0.0) {
            // Convert L to equirectangular UV
            let eq_uv = vec2(
                atan2(L.z, L.x), 
                asin(L.y)
            ) * INV_ATAN + 0.5;
            let eq_pixel = vec2<i32>(eq_uv * src_dimensions);

            let sample = textureLoad(src, eq_pixel, 0);

            prefiltered_color += sample.rgb * NdotL;
            total_weight += NdotL;
        }
    }

    textureStore(dst, gid.xy, gid.z, vec4(prefiltered_color / total_weight, 1.0));
}

@compute
@workgroup_size(16, 16, 1)
fn main(
    @builtin(global_invocation_id)
    gid: vec3<u32>,
) {
    // If texture size is not divisible by 32, we
    // need to make sure we don't try to write to
    // pixels that don't exist.
    if gid.x >= u32(textureDimensions(dst).x) {
        return;
    }

    // Get texture coords relative to cubemap face
    let cube_uv = vec2<f32>(gid.xy) / vec2<f32>(textureDimensions(dst)) * 2.0 - 1.0;

    // Get normal based on face and cube_uv
    let face = gid_z_to_face(gid.z);
    let N = normalize(face.forward + face.right * cube_uv.x + face.up * cube_uv.y);

    // Calculate view vector
    let V = normalize(-N + cube_uv.x * face.right + cube_uv.y * face.up);

    calculate_pbr_ibl_specular(N, gid, V);
}
