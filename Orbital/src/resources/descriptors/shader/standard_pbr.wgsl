const PI: f32 = 3.14159265359; 
const F0_DEFAULT: f32 = 0.04;

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
    position: vec4<f32>,
    color: vec4<f32>,
}

struct PointLightStore {
    lights: array<PointLight>,
}

struct PBRFactors {
    albedo_factor: vec3<f32>,
    metallic_factor: f32,
    roughness_factor: f32,
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
    // Diffuse IBL
    ibl_diffuse: vec3<f32>,
    // Specular IBL
    ibl_specular: vec3<f32>,
    // BRDF LuT (look-up-table) (used for IBL)
    brdf_lut: vec2<f32>,
    // Normal
    N: vec3<f32>,
    // Outgoing light direction originating from camera
    V: vec3<f32>,
    // Dot product (multiplication) of normal and outgoing light
    NdotV: f32,
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

@group(0) @binding(12) var<uniform> pbr_factors: PBRFactors;

@group(1) @binding(0) var<uniform> camera: CameraUniform;

@group(2) @binding(0) var<storage> point_light_store: array<PointLight>;

@group(3) @binding(0) var diffuse_env_map: texture_cube<f32>;
@group(3) @binding(1) var diffuse_sampler: sampler;

@group(3) @binding(2) var specular_env_map: texture_cube<f32>;
@group(3) @binding(3) var specular_sampler: sampler;

@group(3) @binding(4) var ibl_brdf_lut_texture: texture_2d<f32>;
@group(3) @binding(5) var ibl_brdf_lut_sampler: sampler;

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
    out.world_position = world_position.xyz;

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
    var output = vec3(0.0);

    // IBL Ambient light
    var ambient = calculate_ambient_ibl(pbr);
    output += ambient;

    // Point Light reflectance light
    let point_light_reflectance = calculate_point_light_specular_contribution(pbr, in.world_position);
    output += point_light_reflectance;

    // Add emissive "ontop"
    output += pbr.emissive;

    // Tonemap / HDR 
    let tone_mapped_color = aces_tone_map(output);
    return vec4<f32>(tone_mapped_color, 1.0);
}

// Note: Unused in favor of ACES
fn hdr_tone_map_gamma_correction(color: vec3<f32>) -> vec3<f32> {
    var result = color / (color + vec3<f32>(1.0));
    result = pow(result, vec3<f32>(1.0 / camera.global_gamma));
    return result;
}

// ACES tone mapping
const ACES_A: f32 = 2.51;
const ACES_B: f32 = 0.03;
const ACES_C: f32 = 2.43;
const ACES_D: f32 = 0.59;
const ACES_E: f32 = 0.14;
fn aces_tone_map(color: vec3<f32>) -> vec3<f32> {
    return clamp(
        (color * (ACES_A * color + ACES_B)) / 
        (color * (ACES_C * color + ACES_D) + ACES_E), 
        vec3(0.0), 
        vec3(1.0)
    );
}

fn brdf(point_light: PointLight, pbr: PBRData, world_position: vec3<f32>) -> vec3<f32> {
    let L = normalize(point_light.position.xyz - world_position);
    let H = normalize(pbr.V + L);

    let NdotL = clamp(dot(pbr.N, L), 0.0, 1.0);
    let NdotH = clamp(dot(pbr.N, H), 0.0, 1.0);

    var Lo: vec3<f32>;
    if NdotL > 0.0 {
        // Normal distribution of the microfacets
        let D = distribution_ggx(NdotH, pbr.roughness);
        // Geometric/Microfacet shadowing term
        let G = schlick_smith_ggx(NdotL, pbr.NdotV, pbr.roughness);
        // Fresnel factor (i.e. reflectance depending on angle of camera)
        let F = fresnel_schlick(pbr.NdotV, pbr);

        let nominator = D * F * G;
        let denominator = 4.0 * NdotL * pbr.NdotV + 0.0001; // +0.0001 prevents division by zero
        let specular = nominator / denominator;
        Lo += specular * NdotL * point_light.color.rgb;
    }
    return Lo;
}

fn calculate_point_light_specular_contribution(pbr: PBRData, world_position: vec3<f32>) -> vec3<f32> {
    var Lo = vec3(0.0);

    for (var i = u32(0); i < arrayLength(&point_light_store); i++) {
        let point_light = point_light_store[i];
        Lo += brdf(point_light, pbr, world_position); 
    }

    return Lo;
}

fn calculate_ambient_ibl(pbr: PBRData) -> vec3<f32> {
    // Calculate reflectance at normal incidence
    let F0 = mix(vec3(F0_DEFAULT), pbr.albedo, pbr.metallic);
    let F = fresnel_schlick_roughness(pbr.NdotV, F0, pbr.roughness);

    // IBL Diffuse
    let diffuse_color = (pbr.albedo * (vec3(1.0) - F) + 0.0001) * (1.0 - pbr.metallic + 0.0001);
    let diffuse_ibl = pbr.ibl_diffuse * diffuse_color;

    // IBL Specular
    let specular_color = mix(F0, pbr.albedo, pbr.metallic);
    var specular_ibl = pbr.ibl_specular * (F * pbr.brdf_lut.x + pbr.brdf_lut.y);

    // Ambient light calculation (IBL), multiplied by ambient occlusion
    return (diffuse_ibl + specular_ibl) * pbr.occlusion;
}

/// Samples the fragment's normal and transforms it into world space
fn sample_normal_from_map(fragment_data: FragmentData) -> vec3<f32> {
    let normal_sample = textureSample(
        normal_texture,
        normal_sampler,
        fragment_data.uv
    ).rgb;
    let tangent_normal = 2.0 * normal_sample - 1.0;

    let TBN = mat3x3(
        fragment_data.tangent,
        fragment_data.bitangent,
        fragment_data.normal,
    );
    let N = normalize(TBN * tangent_normal);
    return N;
}

// Fresnel
fn fresnel_schlick(cos_theta: f32, pbr: PBRData) -> vec3<f32> {
    let F0 = mix(vec3(F0_DEFAULT), pbr.albedo, pbr.metallic);
    let F = F0 + (1.0 - F0) * pow(1.0 - cos_theta, 5.0);
    return F;
}

// Fresnel schlick **with** roughness
fn fresnel_schlick_roughness(cos_theta: f32, F0: vec3<f32>, roughness: f32) -> vec3<f32> {
    return F0 + (max(vec3<f32>(1.0 - roughness), F0) - F0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

/// Geometric Shadowing
fn schlick_smith_ggx(NdotL: f32, NdotV: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;
    let GL = NdotL / (NdotL * (1.0 - k) + k);
    let GV = NdotV / (NdotV * (1.0 - k) + k);
    return GL * GV;
}

// Normal distribution
fn distribution_ggx(NdotH: f32, roughness: f32) -> f32 {
    let alpha = roughness * roughness;
    let alpha_squared = alpha * alpha;

    let denom = (NdotH * NdotH) * (alpha_squared - 1.0) + 1.0;
    return alpha_squared / (PI * denom * denom);
}

fn pbr_data(fragment_data: FragmentData) -> PBRData {
    var out: PBRData;

    // Precalculations
    out.N = sample_normal_from_map(fragment_data);
    out.V = normalize(camera.position.xyz - fragment_data.world_position);
    let R = normalize(reflect(-out.V, out.N));
    out.NdotV = clamp(dot(out.N, out.V), 0.0, 1.0);

    // Material properties
    let albedo_sample = textureSample(
        albedo_texture,
        albedo_sampler,
        fragment_data.uv
    ).rgb;
    let albedo_factored = albedo_sample * pbr_factors.albedo_factor.rgb;
    let albedo_clamped = clamp(albedo_factored, vec3(0.0), vec3(1.0));
    let albedo_gamma_applied = pow(albedo_clamped, vec3(camera.global_gamma));
    out.albedo = albedo_gamma_applied;

    let metallic_sample = textureSample(
        metallic_texture,
        metallic_sampler,
        fragment_data.uv
    ).r;
    let metallic_factored = metallic_sample * pbr_factors.metallic_factor;
    let metallic_clamped = clamp(metallic_factored, 0.0, 1.0);
    out.metallic = metallic_clamped;

    let roughness_sample = textureSample(
        roughness_texture,
        roughness_sampler,
        fragment_data.uv
    ).r;
    let roughness_factored = roughness_sample * pbr_factors.roughness_factor;
    let roughness_clamped = clamp(roughness_factored, 0.0, 1.0);
    out.roughness = roughness_clamped;

    let occlusion_sample = textureSample(
        occlusion_texture,
        occlusion_sampler,
        fragment_data.uv
    ).r;
    let occlusion_clamped = clamp(occlusion_sample, 0.0, 1.0);
    out.occlusion = occlusion_clamped;

    let emissive_sample = textureSample(
        emissive_texture,
        emissive_sampler,
        fragment_data.uv
    ).rgb;
    let emissive_clamped = clamp(emissive_sample, vec3(0.0), vec3(1.0));
    let emissive_gamma_applied = pow(emissive_clamped, vec3(camera.global_gamma));
    out.emissive = emissive_gamma_applied;

    let diffuse_sample = textureSample(
        diffuse_env_map,
        diffuse_sampler,
        out.N
    ).rgb;
    let diffuse_clamped = clamp(diffuse_sample, vec3(0.0), vec3(1.0));
    let diffuse_gamma_applied = pow(diffuse_clamped, vec3(camera.global_gamma));
    out.ibl_diffuse = diffuse_gamma_applied;

    let specular_mip_count = textureNumLevels(specular_env_map);
    let specular_mip_level = out.roughness * out.roughness * f32(specular_mip_count - 1u);
    let specular_sample = textureSampleLevel(
        specular_env_map,
        specular_sampler,
        R,
        specular_mip_level
    ).rgb;
    let specular_clamped = clamp(specular_sample, vec3(0.0), vec3(1.0));
    let specular_gamma_applied = pow(specular_clamped, vec3(camera.global_gamma));
    out.ibl_specular = specular_gamma_applied;

    let brdf_lut_sample = textureSample(
        ibl_brdf_lut_texture,
        ibl_brdf_lut_sampler,
        vec2<f32>(
            max(out.NdotV, 0.0001), 
            clamp(1.0 - out.roughness, 0.0001, 1.0)
    )).rg;
    out.brdf_lut = brdf_lut_sample;

    return out;
}
