@group(0) @binding(0)
var dst: texture_storage_2d<rgba32float, write>;

const PI: f32 = 3.1415926535897932384626433832795;
const SAMPLE_COUNT: u32 = 1024;
const DIMENSIONS: u32 = 512;

@compute
@workgroup_size(16, 16, 1)
fn main(
    @builtin(global_invocation_id)
    gid: vec3<u32>,
) {
    let base = 1.0 / f32(DIMENSIONS - 1);
    let x = base * f32(gid.x);
    let y = base * f32(gid.y);

    // Note: Use `1.0 - f32(y)` for Y if you want the OpenGL orientation.
    // Hint: OpenGL orientation has the green part in the bottom left corner.
    //       Here it's in the top left corner.
    let integrate_brdf = integrate_brdf(f32(x), f32(y));
    textureStore(dst, gid.xy, vec4<f32>(integrate_brdf, 0.0, 1.0));
}

fn integrate_brdf(NdotV: f32, roughness: f32) -> vec2<f32> {
    let N = vec3<f32>(0.0, 0.0, 1.0);
    let V = vec3<f32>(sqrt(1.0 - NdotV * NdotV), 0.0, NdotV);

    var LUT = vec2<f32>(0.0);
    for (var i = 0u; i < SAMPLE_COUNT; i++) {
        let Xi: vec2<f32> = hammersley(i, SAMPLE_COUNT);
        let H: vec3<f32> = importance_sample_ggx(Xi, N, roughness);
        let L: vec3<f32> = normalize(2.0 * dot(V, H) * H - V);

        let NdotL: f32 = max(dot(N, L), 0.0);
        let NdotV: f32 = max(dot(N, V), 0.0);
        let VdotH: f32 = max(dot(V, H), 0.0);
        let NdotH: f32 = max(dot(H, N), 0.0);

        if NdotL > 0.0 {
            let G: f32 = geometry_schlick_smith_ggx(NdotL, NdotV, roughness);
            let G_Vis: f32 = (G * VdotH) / (NdotH * NdotV);
            let Fc: f32 = pow(1.0 - VdotH, 5.0);
            LUT += vec2<f32>((1.0 - Fc) * G_Vis, Fc * G_Vis);
        }
    }

    return LUT / f32(SAMPLE_COUNT);
}

fn geometry_schlick_smith_ggx(NdotL: f32, NdotV: f32, roughness: f32) -> f32 {
    let k: f32 = (roughness * roughness) / 2.0;
    let GL: f32 = NdotL / (NdotL * (1.0 - k) + k);
    let GV: f32 = NdotV / (NdotV * (1.0 - k) + k);
    return GL * GV;
}

// Adapted from: 
// http://holger.dammertz.org/stuff/notes_HammersleyOnHemisphere.html
// More efficient version of VanDerCorpus.
fn radical_inverse(i: u32) -> f32 {
    var bits: u32 = (i << 16u) | (i >> 16u);
    bits = ((bits & 0x55555555u) << 1u) | ((bits & 0xAAAAAAAAu) >> 1u);
    bits = ((bits & 0x33333333u) << 2u) | ((bits & 0xCCCCCCCCu) >> 2u);
    bits = ((bits & 0x0F0F0F0Fu) << 4u) | ((bits & 0xF0F0F0F0u) >> 4u);
    bits = ((bits & 0x00FF00FFu) << 8u) | ((bits & 0xFF00FF00u) >> 8u);
    return f32(bits) * 2.3283064365386963e-10;
}

fn hammersley(i: u32, N: u32) -> vec2<f32> {
    let radical_inverse = radical_inverse(i);
    return vec2<f32>(f32(i) / f32(N), radical_inverse);
}  

fn importance_sample_ggx(Xi: vec2<f32>, N: vec3<f32>, roughness: f32) -> vec3<f32> {
    // 2D point to hemisphere mapping, based on roughness
    let alpha = roughness * roughness;
    let phi = 2.0 * PI * Xi.x;
    let cosTheta = sqrt((1.0 - Xi.y) / (1.0 + (alpha * alpha - 1.0) * Xi.y));
    let sinTheta = sqrt(1.0 - cosTheta * cosTheta);
    var H = vec3<f32>(cos(phi) * sinTheta, sin(phi) * sinTheta, cosTheta);
	
    // from tangent-space vector to world-space sample vector
    var up: vec3<f32> = select(vec3(1.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0), abs(N.z) < 0.999);
    let tangent = normalize(cross(up, N));
    let bitangent = cross(N, tangent);

    return normalize(tangent * H.x + bitangent * H.y + N * H.z);
}  
