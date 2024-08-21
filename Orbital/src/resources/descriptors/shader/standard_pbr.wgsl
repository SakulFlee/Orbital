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

struct PBRData {
    // Albedo (color) texture sample
    albedo: vec3<f32>,
    // Metallic factor
    metallic: f32,
    // Roughness factor
    roughness: f32,
    // Occlusion factor
    occlusion: f32,
    // Emissive (like albedo, but ignores light) texture sample
    emissive: vec3<f32>,
    // Irradiance (used for diffuse IBL)
    irradiance: vec3<f32>,
    // Radiance (used for specular IBL)
    radiance: vec3<f32>,
    // BRDF LuT (look-up-table) (used for IBL)
    brdf_lut: vec2<f32>,
    // Tangent-Bitangent-Normal matrix
    TBN: mat3x3<f32>,
    // Normal
    N: vec3<f32>,
    // Outgoing light direction originating from camera
    V: vec3<f32>,
    // Specular lighr reflection 
    R: vec3<f32>,
    // Dot product (multiplication) of normal and outgoing light
    NdotV: f32,
    // Color reflectance at normal standard dielectric incidence
    F0: vec3<f32>,
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

@group(0) @binding(10) var emissive_texture: texture_2d<f32>;
@group(0) @binding(11) var emissive_sampler: sampler;

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

const POINT_LIGHT_DEFAULT_A = 0.1;
const POINT_LIGHT_DEFAULT_B = 0.1;

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

    // Output for Fragment shader
    var out: FragmentData;

    // Vertex position (perspective applied)
    out.position = camera.perspective_view_projection_matrix * world_position;

    // Actual position in world (no perspective)
    out.world_position = world_position.xyz / world_position.w;

    // Transform UV
    out.uv = (model_space_matrix * vec4<f32>(vertex.uv, 0.0, 0.0)).xy;

    // Transform Tangent
    out.tangent = (model_space_matrix * vec4<f32>(vertex.tangent, 0.0)).xyz;

    // Transform Bitangent
    out.bitangent = (model_space_matrix * vec4<f32>(vertex.bitangent, 0.0)).xyz;

    // Transform Normal
    out.normal = (model_space_matrix * vec4<f32>(vertex.normal, 0.0)).xyz;

    return out;
}

@fragment
fn entrypoint_fragment(in: FragmentData) -> @location(0) vec4<f32> {
    let pbr = pbr_data(in);

    // Point Light reflectance light
    let point_light_reflectance = calculate_point_light_reflectance(pbr, in.world_position);

    // IBL Ambient light
    let ambient = calculate_ambient_ibl(pbr);

    // Combine ambient and light reflection output
    let raw_color = ambient + point_light_reflectance;
    let tone_mapped_color = hdr_tone_map_gamma_correction(raw_color);
    return vec4<f32>(tone_mapped_color, 1.0);
}

fn hdr_tone_map_gamma_correction(color: vec3<f32>) -> vec3<f32> {
    var result = color / (color + vec3<f32>(1.0));
    result = pow(result, vec3<f32>(1.0 / camera.global_gamma));
    return result;
}

fn calculate_point_light_reflectance(pbr: PBRData, world_position: vec3<f32>) -> vec3<f32> {
    var Lo = vec3<f32>(0.0);
    for (var i: u32 = 0; i < arrayLength(&lights.point_lights); i++) {
        let point_light = lights.point_lights[i];   

        // Calculate per-light radiance
        let L = normalize(point_light.position - world_position);
        let H = normalize(pbr.V + L);

        let radiance = point_light_radiance(point_light, world_position);

        // Cook-Torrance BRDF
        let NDF = DistributionGGX(pbr.N, H, pbr.roughness);
        let G = geometry_smith(pbr.N, pbr.V, L, pbr.roughness);
        let F = fresnel_schlick(max(dot(H, pbr.V), 0.0), pbr.F0);

        let numerator = NDF * G * F;
        let NdotL = max(dot(pbr.N, L), 0.0);
        let denominator = 4.0 * pbr.NdotV * NdotL + 0.0001; // +0.0001 prevents division by zero
        let specular = numerator / denominator;

        let kD = mix(vec3<f32>(1.0) - F, vec3<f32>(0.0), pbr.metallic);

        // Adding radiance to Lo
        Lo += (kD * pbr.albedo / PI + specular) * radiance * NdotL;
    }
    return Lo;
}

fn calculate_ambient_ibl(pbr: PBRData) -> vec3<f32> {
    // Pre-calculations for IBL/Ambient Light
    let F = fresnel_schlick_roughness(pbr.NdotV, pbr.F0, pbr.roughness);
    let kD = mix(vec3<f32>(1.0) - F, vec3<f32>(0.0), pbr.metallic);

    // IBL Diffuse
    let diffuse_ibl = kD * (pbr.irradiance * pbr.albedo);

    // IBL Specular
    var specular_ibl = pbr.radiance * (pbr.F0 * pbr.brdf_lut.x + pbr.brdf_lut.y);

    // Ambient light calculation (IBL)
    let ambient = (diffuse_ibl + specular_ibl) * pbr.occlusion * pbr.emissive;
    return ambient;
}

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

fn DistributionGGX(N: vec3<f32>, H: vec3<f32>, roughness: f32) -> f32 {
    let alpha = roughness * roughness;
    let alpha_2 = alpha * alpha;

    let NdotH = max(dot(N, H), 0.0);
    let NdotH_2 = NdotH * NdotH;

    let num = alpha_2;
    var denom = (NdotH_2 * (alpha_2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return num / denom;
}

// Schlick-GGX approximation of geometric attenuation function using Smith's method.
fn geometry_schlick_ggx(NdotV: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;

    let num = NdotV;
    let denom = NdotV * (1.0 - k) + k;

    return num / denom;
}

fn geometry_smith(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, roughness: f32) -> f32 {
    let NdotV = max(dot(N, V), 0.0);
    let NdotL = max(dot(N, L), 0.0);

    let ggx2 = geometry_schlick_ggx(NdotV, roughness);
    let ggx1 = geometry_schlick_ggx(NdotL, roughness);

    return ggx1 * ggx2;
}

fn point_light_attenuation(a: f32, b: f32, point_light: PointLight, world_position: vec3<f32>) -> f32 {
    let distance = length(point_light.position - world_position);

    return 1.0 / (1.0 + a * distance + b * distance * distance);
}

fn point_light_radiance(point_light: PointLight, world_position: vec3<f32>) -> vec3<f32> {
    let attenuation = point_light_attenuation(
        POINT_LIGHT_DEFAULT_A,
        POINT_LIGHT_DEFAULT_B,
        point_light,
        world_position
    );
    return point_light.color * attenuation;
}

fn pbr_data(fragment_data: FragmentData) -> PBRData {
    // Precalculations
    let TBN = mat3x3(
        fragment_data.tangent,
        fragment_data.bitangent,
        fragment_data.normal,
    );
    let N = sample_normal_from_map(fragment_data.uv, fragment_data.world_position, TBN);
    let V = normalize(camera.position.xyz - fragment_data.world_position);
    let R = reflect(-V, N);
    let NdotV = max(dot(N, V), 0.0);

    // Material properties
    let albedo = pow(textureSample(
        albedo_texture,
        albedo_sampler,
        fragment_data.uv
    ).rgb, vec3<f32>(camera.global_gamma));
    let metallic = textureSample(
        metallic_texture,
        metallic_sampler,
        fragment_data.uv
    ).r;
    let roughness = textureSample(
        roughness_texture,
        roughness_sampler,
        fragment_data.uv
    ).r;
    let occlusion = textureSample(
        occlusion_texture,
        occlusion_sampler,
        fragment_data.uv
    ).r;
    let emissive = textureSample(
        emissive_texture,
        emissive_sampler,
        fragment_data.uv
    ).rgb;
    let irradiance = textureSample(
        irradiance_env_map,
        irradiance_sampler,
        N
    ).rgb;
    let radiance = textureSampleLevel(
        radiance_env_map,
        radiance_sampler,
        R,
        roughness
    ).rgb;
    let brdf_lut = textureSample(
        ibl_brdf_lut_texture,
        ibl_brdf_lut_sampler,
        vec2<f32>(NdotV, roughness)
    ).rg;

    // Calculate reflectance at normal incidence
    var F0 = mix(F0_DIELECTRIC_STANDARD, albedo, metallic);

    var out: PBRData;
    out.albedo = albedo;
    out.metallic = metallic;
    out.roughness = roughness;
    out.occlusion = occlusion;
    out.emissive = emissive;
    out.irradiance = irradiance;
    out.radiance = radiance;
    out.brdf_lut = brdf_lut;
    out.TBN = TBN;
    out.N = N;
    out.V = V;
    out.R = R;
    out.NdotV = NdotV;
    out.F0 = F0;
    return out;
}
