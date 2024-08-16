struct VertexData {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tangent: vec3<f32>,
    @location(3) bitangent: vec3<f32>,
    @location(4) uv: vec2<f32>,
}

struct InstanceData {
    @location(5) model_space_matrix_0: vec4<f32>,
    @location(6) model_space_matrix_1: vec4<f32>,
    @location(7) model_space_matrix_2: vec4<f32>,
    @location(8) model_space_matrix_3: vec4<f32>,
}

struct FragmentData {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) tangent: vec3<f32>,
    @location(3) bitangent: vec3<f32>,
    @location(4) normal: vec3<f32>,
}

struct CameraUniform {
    position: vec3<f32>,
    view_projection_matrix: mat4x4<f32>,
    perspective_view_projection_matrix: mat4x4<f32>,
    view_projection_transposed: mat4x4<f32>,
    perspective_projection_invert: mat4x4<f32>,
    global_gamma: f32,
    skybox_gamma: f32,
}

struct PointLight {
    position: vec3<f32>,
    color: vec3<f32>,
}

struct LightStorage {
    point_lights: array<PointLight>,
}

@group(0) @binding(0) var normal_texture: texture_2d<f32>;
@group(0) @binding(1) var normal_sampler: sampler;

@group(0) @binding(2) var albedo_texture: texture_2d<f32>;
@group(0) @binding(3) var albedo_sampler: sampler;

@group(0) @binding(4) var metallic_texture: texture_2d<f32>;
@group(0) @binding(5) var metallic_sampler: sampler;

@group(0) @binding(6) var roughness_texture: texture_2d<f32>;
@group(0) @binding(7) var roughness_sampler: sampler;

@group(0) @binding(8) var occlusion_texture: texture_2d<f32>;
@group(0) @binding(9) var occlusion_sampler: sampler;

@group(1) @binding(0) var<uniform> camera: CameraUniform;

@group(2) @binding(0) var<storage> lights: LightStorage;

@group(3) @binding(2) var irradiance_env_map: texture_cube<f32>;
@group(3) @binding(3) var irradiance_sampler: sampler;

@group(3) @binding(4) var radiance_env_map: texture_cube<f32>;
@group(3) @binding(5) var radiance_sampler: sampler;

@group(3) @binding(6) var ibl_brdf_lut_texture: texture_2d<f32>;
@group(3) @binding(7) var ibl_brdf_lut_sampler: sampler;

const PI = 3.14159265359; 
const F0_DIELECTRIC_STANDARD = vec3<f32>(0.04);
const MAX_REFLECTION_LOD = 1.0;

/// Samples the fragment's normal and transforms it into world space
fn sample_normal_from_map(uv: vec2<f32>, world_position: vec3<f32>, TBN: mat3x3<f32>) -> vec3<f32> {
    let normal_sample = textureSample(
        normal_texture,
        normal_sampler,
        uv
    ).rgb;

    let tangent_normal = 2.0 * normal_sample - 1.0;
    let N = normalize(TBN * tangent_normal);

    return N;
}

fn fresnel_schlick(cos_theta: f32, F0: vec3<f32>) -> vec3<f32> {
    return F0 + (1.0 - F0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

fn fresnel_schlick_roughness(cos_theta: f32, F0: vec3<f32>, roughness: f32) -> vec3<f32> {
    return F0 + (max(vec3<f32>(1.0 - roughness), F0) - F0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

fn distribution_ggx(N: vec3<f32>, H: vec3<f32>, roughness: f32) -> f32 {
    let alpha = roughness * roughness;
    let alpha_squared = alpha * alpha;

    let NdotH = max(0.0, dot(N, H));
    let NdotH_squared = NdotH * NdotH;

    let denom = NdotH_squared * (alpha_squared - 1.0) + 1.0;
    let denom_squared = denom * denom;

    return alpha_squared / (PI * denom_squared);
}

fn geometry_schlick_ggx(NdotV: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;

    let denom = NdotV * (1.0 - k) + k;

    return NdotV / denom;
}

fn geometry_smith(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, roughness: f32) -> f32 {
    let NdotV = max(dot(N, V), 0.0);
    let NdotL = max(dot(N, L), 0.0);

    let ggx2 = geometry_schlick_ggx(NdotV, roughness);
    let ggx1 = geometry_schlick_ggx(NdotL, roughness);

    return ggx1 * ggx2;
}

@vertex
fn entrypoint_vertex(
    vertex: VertexData,
    instance: InstanceData
) -> FragmentData {
    let model_space_matrix = mat4x4<f32>(
        instance.model_space_matrix_0,
        instance.model_space_matrix_1,
        instance.model_space_matrix_2,
        instance.model_space_matrix_3,
    );

    // Calculate world position
    let world_position = model_space_matrix * vec4<f32>(vertex.position, 1.0);

    // Passthrough variables
    var out: FragmentData;
    out.position = camera.perspective_view_projection_matrix * world_position;
    out.world_position = world_position.xyz;
    out.uv = vertex.uv;
    out.tangent = vertex.tangent;
    out.bitangent = vertex.bitangent;
    out.normal = vertex.normal;
    return out;
}

@fragment
fn entrypoint_fragment(in: FragmentData) -> @location(0) vec4<f32> {    
    // Precalculate TBN (Tangent, Bitangent, Normal) matrix
    let TBN = mat3x3(
        in.tangent,
        in.bitangent,
        in.normal,
    );

    // Material properties
    let albedo = pow(textureSample(
        albedo_texture,
        albedo_sampler,
        in.uv
    ).rgb, vec3<f32>(camera.global_gamma));
    let metallic = textureSample(
        metallic_texture,
        metallic_sampler,
        in.uv
    ).r;
    let roughness = textureSample(
        roughness_texture,
        roughness_sampler,
        in.uv
    ).r;
    let occlusion = textureSample(
        occlusion_texture,
        occlusion_sampler,
        in.uv
    ).r;

    // Fragment's Normal
    let N = sample_normal_from_map(in.uv, in.world_position, TBN);

    // Outgoing Light direction (camera position == eye)
    let V = normalize(camera.position.xyz - in.world_position);

    // Specular light reflection
    let R = reflect(-V, N);

    // Calculate reflectance at normal incidence
    var F0 = mix(F0_DIELECTRIC_STANDARD, albedo, metallic);

    // Reflectance equation
    var Lo = vec3<f32>(0.0);
    for (var i: u32 = 0; i < arrayLength(&lights.point_lights); i++) {
        let point_light = lights.point_lights[i];   

        // Calculate per-light radiance
        let L = normalize(point_light.position - in.world_position);
        let H = normalize(V + L);

        let distance = length(L);
        let attenuation = 1.0 / (distance * distance);
        let radiance = point_light.color * attenuation;

        // Cook-Torrance BRDF
        let NDF = distribution_ggx(N, H, roughness);
        let G = geometry_smith(N, V, L, roughness);
        let F = fresnel_schlick(max(dot(H, V), 0.0), F0);

        let numerator = NDF * G * F;
        let denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001; // +0.0001 prevents division by zero
        let specular = numerator / denominator;

        let kD = mix(vec3<f32>(1.0) - F, vec3<f32>(0.0), metallic);

        // Adding radiance to Lo
        let NdotL = max(dot(N, L), 0.0);
        Lo += (kD * albedo / PI + specular) * radiance * NdotL;
    }

    // Ambient light calculation (IBL Diffuse)
    let F = fresnel_schlick_roughness(max(dot(N, V), 0.0), F0, roughness);
    let kD = mix(vec3<f32>(1.0) - F, vec3<f32>(0.0), metallic);

    let irradiance = textureSample(
        irradiance_env_map,
        irradiance_sampler,
        N
    ).rgb;
    let diffuse_ibl = irradiance * albedo;

    // IBL Specular
    let radiance_color = textureSampleLevel(
        radiance_env_map,
        radiance_sampler,
        R,
        roughness * MAX_REFLECTION_LOD
    ).rgb;

    let specular_brdf = textureSample(
        ibl_brdf_lut_texture,
        ibl_brdf_lut_sampler,
        vec2<f32>(max(dot(N, V), 0.0), roughness)
    ).rg;
    var specular_ibl = radiance_color * (F * specular_brdf.x + specular_brdf.y);

    // let EMISSIVE = 0.001; // TODO
    let EMISSIVE = 1.0;
    let ambient = (kD * diffuse_ibl + specular_ibl) * occlusion * EMISSIVE;
    var color = ambient + Lo;

    // HDR gamma correction / tone mapping / Reinhard operator
    color = color / (color + vec3<f32>(1.0));
    color = pow(color, vec3<f32>(1.0 / camera.global_gamma));

    return vec4<f32>(color, 1.0);
}
