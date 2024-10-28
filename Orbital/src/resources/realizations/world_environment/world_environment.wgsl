const PI: f32 = 3.1415926535897932384626433832795;
const SAMPLE_COUNT: u32 = 1024u;

struct Face {
    forward: vec3<f32>,
    up: vec3<f32>,
    right: vec3<f32>,
}

struct Info {
    // If >= 0, we're generating a mipmap for specular IBL
    // If < 0, we're generating diffuse IBL
    rougness_percent: i32,
}

@group(0) @binding(0)
var<uniform> info: Info;

@group(0) @binding(1)
var src: texture_2d<f32>;

@group(0) @binding(2)
var dst: texture_storage_2d_array<rgba16float, write>;

// Importance sampling functions
fn van_der_corput(n: u32, base: u32) -> f32 {
    var m = n;
    var q = 0.0;
    var b = f32(base);
    var k = 1.0;
    
    while(m > 0u) {
        let a = f32(m % base);
        q = q + a * k;
        k = k / b;
        m = m / base;
    }
    return q;
}

fn hammersley(i: u32, N: u32) -> vec2<f32> {
    return vec2(f32(i) / f32(N), van_der_corput(i, 2u));
}

fn importance_sample_ggx(Xi: vec2<f32>, roughness: f32, N: vec3<f32>) -> vec3<f32> {
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
    let up = select(vec3(1.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0), abs(N.y) < 0.999);
    let tangentX = normalize(cross(up, N));
    let tangentY = cross(N, tangentX);

    return tangentX * H.x + tangentY * H.y + N * H.z;
}

fn calculate_pbr_ibl_diffuse(N: vec3<f32>, gid: vec3<u32>) {
    var irradiance = vec3(0.0);
    let sample_delta = PI * 0.5 / 64.0;

    for(var phi = 0.0; phi < 2.0 * PI; phi += sample_delta) {
        for(var theta = 0.0; theta < 0.5 * PI; theta += sample_delta) {
            let tangent = vec3(sin(theta) * cos(phi), sin(theta) * sin(phi), cos(theta));
            let L = normalize(tangent);
            let NdotL = max(dot(N, L), 0.0);

            let inv_atan = vec2(0.1591, 0.3183);
            let eq_uv = vec2(atan2(L.z, L.x), asin(L.y)) * inv_atan + 0.5;
            let eq_pixel = vec2<i32>(eq_uv * vec2<f32>(textureDimensions(src)));
            
            let sample = textureLoad(src, eq_pixel, 0);
            irradiance += sample.rgb * cos(theta) * sin(theta) * NdotL;
        }
    }

    let prefiltered_color = irradiance * PI * (1.0 / 64.0) * (1.0 / 64.0);
    textureStore(dst, gid.xy, gid.z, vec4(prefiltered_color, 1.0));
}

fn calculate_pbr_ibl_specular(N: vec3<f32>, gid: vec3<u32>, roughness: f32) {
    var prefiltered_color = vec3(0.0);
    var total_weight = 0.0;

    for(var i = 0u; i < SAMPLE_COUNT; i++) {
        let Xi = hammersley(i, SAMPLE_COUNT);
        let H = importance_sample_ggx(Xi, roughness, N);
        let L = normalize(2.0 * dot(N, H) * H - N);
        
        let NdotL = max(dot(N, L), 0.0);
        if(NdotL > 0.0) {
            let edge_weight = smoothstep(0.0, 0.2, NdotL);

            // Convert L to equirectangular UV
            let inv_atan = vec2(0.1591, 0.3183);
            let eq_uv = vec2(atan2(L.z, L.x), asin(L.y)) * inv_atan + 0.5;
            let eq_pixel = vec2<i32>(eq_uv * vec2<f32>(textureDimensions(src)));
            
            let sample = textureLoad(src, eq_pixel, 0);
            prefiltered_color += sample.rgb * NdotL * edge_weight;
            total_weight += NdotL * edge_weight;
        }
    }

    prefiltered_color = select(prefiltered_color / total_weight, prefiltered_color, total_weight == 0.0);
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

    var FACES: array<Face, 6> = array(
        // FACE +X
        Face(
            vec3(1.0, 0.0, 0.0),  // forward
            vec3(0.0, 1.0, 0.0),  // up
            vec3(0.0, 0.0, -1.0), // right
        ),
        // FACE -X
        Face(
            vec3(-1.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            vec3(0.0, 0.0, 1.0),
        ),
        // FACE +Y
        Face(
            vec3(0.0, -1.0, 0.0),
            vec3(0.0, 0.0, 1.0),
            vec3(1.0, 0.0, 0.0),
        ),
        // FACE -Y
        Face(
            vec3(0.0, 1.0, 0.0),
            vec3(0.0, 0.0, -1.0),
            vec3(1.0, 0.0, 0.0),
        ),
        // FACE +Z
        Face(
            vec3(0.0, 0.0, 1.0),
            vec3(0.0, 1.0, 0.0),
            vec3(1.0, 0.0, 0.0),
        ),
        // FACE -Z
        Face(
            vec3(0.0, 0.0, -1.0),
            vec3(0.0, 1.0, 0.0),
            vec3(-1.0, 0.0, 0.0),
        ),
    );

    // Get texture coords relative to cubemap face
    let cube_uv = vec2<f32>(gid.xy) / vec2<f32>(textureDimensions(dst)) * 2.0 - 1.0;

    // Get normal based on face and cube_uv
    let face = FACES[gid.z];
    let N = normalize(face.forward + face.right * cube_uv.x + face.up * cube_uv.y);

    if (info.rougness_percent < 0) {
        // Diffuse
        calculate_pbr_ibl_diffuse(N, gid);
    } else {
        // Specular

        // Convert percentage (0-100%) into expected float (0.0-1.0)
        let f_roughness = f32(info.rougness_percent) / 100.0;

        calculate_pbr_ibl_specular(N, gid, f_roughness);
    }
}
