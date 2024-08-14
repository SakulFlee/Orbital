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
    @location(2) normal: vec3<f32>,
    // @location(3) tangent: vec3<f32>,
    // @location(4) bitangent: vec3<f32>,
    // @location(5) camera_position: vec3<f32>,
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

@group(3) @binding(0) var irradiance_env_map: texture_cube<f32>;
@group(3) @binding(1) var irradiance_sampler: sampler;

@group(3) @binding(2) var radiance_env_map: texture_cube<f32>;
@group(3) @binding(3) var radiance_sampler: sampler;

const PI = 3.14159265359; 
const STANDARD_F0 = vec3<f32>(0.04);

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

    // Calculate actual position
    let world_position = model_space_matrix * vec4<f32>(vertex.position, 1.0);

    // Passthrough variables
    var out: FragmentData;
    out.position = camera.perspective_view_projection_matrix * world_position;
    out.world_position = world_position.xyz;
    out.uv = vertex.uv;
    out.normal = vertex.normal;
    // out.tangent = vertex.tangent;
    // out.bitangent = vertex.bitangent;
    // out.camera_position = camera.position;
    return out;
}

@fragment
fn entrypoint_fragment(in: FragmentData) -> @location(0) vec4<f32> {    
    // Sample textures
    let albedo = pow(textureSample(
        albedo_texture,
        albedo_sampler,
        in.uv
    ).rgb, vec3<f32>(2.2));
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

    // Sample normal
    let normal = sample_normal_from_map(in.uv, in.world_position, in.normal);
    let N = normalize(normal);

    // TODO: Might be able to get rid of tangent, bitangent, etc.?

    // Precalculations
    let V = normalize(camera.position.xyz - in.world_position);

    var F0 = STANDARD_F0;
    F0 = mix(F0, albedo.xyz, metallic);

    // Reflectance equation
    var Lo = vec3<f32>(0.0);
    for (var i: u32 = 0; i < arrayLength(&lights.point_lights); i++) {
        let point_light = lights.point_lights[i];

        // Calculate per-light radiance
        let L = normalize(point_light.position - in.world_position);
        let H = normalize(V + L);

        let distance = length(point_light.position - in.world_position);
        let attenuation = 1.0 / (distance * distance);
        let radiance = point_light.color * attenuation;

        // Cook-Torrance BRDF
        let NDF = distribution_ggx(N, H, roughness);
        let G = geometry_smith(N, V, L, roughness);
        let F = fresnel_schlick(max(dot(H, V), 0.0), F0);

        let kS = F;
        var refraction_ratio = vec3<f32>(1.0) - kS; // refraction_ratio = kD
        refraction_ratio *= 1.0 - metallic;

        let numerator = NDF * G * F;
        let denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001;
        let specular = numerator / denominator;

        // Adding radiance to Lo
        let NdotL = max(dot(N, L), 0.0);
        Lo += (refraction_ratio * albedo.xyz / PI + specular) * radiance * NdotL;
    }

    // Ambient calculation
    let ambient = vec3<f32>(0.03) * albedo.xyz * occlusion;
    var color = ambient + Lo;

    // HDR gamma correction / tone mapping / Reinhard operator
    color = color / (color + vec3<f32>(1.0));
    color = pow(color, vec3<f32>(1.0 / camera.global_gamma));

    return vec4<f32>(color, 1.0);
}

fn sample_normal_from_map(uv: vec2<f32>, world_position: vec3<f32>, vertex_normal: vec3<f32>) -> vec3<f32> {
    let normal_sample = textureSample(
        normal_texture,
        normal_sampler,
        uv
    ).xyz;
    let tangent_normal = normal_sample * 2.0 - 1.0;

    let Q1 = dpdx(world_position);
    let Q2 = dpdy(world_position);
    let st1 = dpdx(uv);
    let st2 = dpdy(uv);

    let N = normalize(vertex_normal);
    let T = normalize(Q1 * st2.y - Q2 * st1.y);
    let B = -normalize(cross(N, T));

    let TBN = mat3x3<f32>(T, B, N);
    return normalize(TBN * tangent_normal);
}

fn fresnel_schlick(cos_theta: f32, F0: vec3<f32>) -> vec3<f32> {
    let c = pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);

    return F0 + (1.0 - F0) * c;
}

fn distribution_ggx(N: vec3<f32>, H: vec3<f32>, roughness: f32) -> f32 {
    let a2 = roughness * roughness;
    let a2_2 = a2 * a2;

    let NdotH = max(dot(N, H), 0.0);
    let NdotH_2 = NdotH * NdotH;

    var denom = (NdotH_2 * (a2_2 - 1.0) + 1.0);
    denom = PI * pow(denom, 2.0);

    return a2_2 / denom;
}

fn geometry_schlick_ggx(NdotV: f32, roughness: f32) -> f32 {
    let r = (roughness + 1.0);
    let r_2 = r * r;

    let k = r_2 / 8.0;

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
