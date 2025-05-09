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

fn calculate_pbr_ibl_diffuse(N: vec3<f32>, gid: vec3<u32>) {
    var irradiance = vec3(0.0);
    let sample_delta = PI * 0.5 / 64.0;

    for(var phi = 0.0; phi < 2.0 * PI; phi += sample_delta) {
        for(var theta = 0.0; theta < 0.5 * PI; theta += sample_delta) {
            let tangent = vec3(sin(theta) * cos(phi), sin(theta) * sin(phi), cos(theta));
            let L = normalize(tangent);
            let NdotL = max(dot(N, L), 0.0);

            let eq_uv = vec2(atan2(L.z, L.x), asin(L.y)) * INV_ATAN + 0.5;
            let eq_pixel = vec2<i32>(eq_uv * vec2<f32>(textureDimensions(src)));
            
            let sample = textureLoad(src, eq_pixel, 0);
            irradiance += sample.rgb * cos(theta) * sin(theta) * NdotL;
        }
    }

    let prefiltered_color = irradiance * PI * (1.0 / 64.0) * (1.0 / 64.0);
    textureStore(dst, gid.xy, gid.z, vec4(prefiltered_color, 1.0));
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

    calculate_pbr_ibl_diffuse(N, gid);
}
