// --- Structures ---

struct VertexPoint {
    @location(0) position_coordinates: vec3<f32>,
    @location(1) texture_coordinates: vec2<f32>,
    @location(2) normal_coordinates: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
}

struct InstanceUniform {
    @location(5) model_space_matrix_0: vec4<f32>,
    @location(6) model_space_matrix_1: vec4<f32>,
    @location(7) model_space_matrix_2: vec4<f32>,
    @location(8) model_space_matrix_3: vec4<f32>,
    @location(9) normal_space_matrix_0: vec3<f32>,
    @location(10) normal_space_matrix_1: vec3<f32>,
    @location(11) normal_space_matrix_2: vec3<f32>,
}

struct CameraUniform {
    position: vec4<f32>,
    view_projection_matrix: mat4x4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) texture_coordinates: vec2<f32>,
    @location(1) tangent_position: vec3<f32>,
    @location(2) tangent_light_position: vec3<f32>,
    @location(3) tangent_view_position: vec3<f32>,
};

struct AmbientLight {
    color: vec3<f32>,
    strength: f32,
};

struct PointLight {
    color: vec4<f32>,
    position: vec4<f32>,
    strength: f32,
    enabled: u32,
}

// --- Bindings ---

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;

@group(0) @binding(1)
var s_diffuse: sampler;

@group(0) @binding(2)
var t_normal: texture_2d<f32>;

@group(0) @binding(3)
var s_normal: sampler;

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@group(2) @binding(0) 
var<uniform> ambient_light: AmbientLight;

// Point Lights
@group(3) @binding(0) 
var<uniform> point_light: PointLight;

// --- Vertex ---

@vertex
fn vs_main(
    vertex_point: VertexPoint,
    instance: InstanceUniform,
) -> VertexOutput {
    let model_space_matrix = mat4x4<f32>(
        instance.model_space_matrix_0,
        instance.model_space_matrix_1,
        instance.model_space_matrix_2,
        instance.model_space_matrix_3,
    );

    let normal_matrix = mat3x3<f32>(
        instance.normal_space_matrix_0,
        instance.normal_space_matrix_1,
        instance.normal_space_matrix_2,
    );

    // Make tangent matrix
    let world_normal = normalize(normal_matrix * vertex_point.normal_coordinates);
    let world_tangent = normalize(normal_matrix * vertex_point.tangent);
    let world_bitangent = normalize(normal_matrix * vertex_point.bitangent);
    let tangent_matrix = transpose(mat3x3<f32>(
        world_tangent,
        world_bitangent,
        world_normal
    ));

    let world_position = model_space_matrix * vec4<f32>(vertex_point.position_coordinates, 1.0);

    var out: VertexOutput;
    out.clip_position = camera.view_projection_matrix * world_position;
    out.texture_coordinates = vertex_point.texture_coordinates;
    out.tangent_position = tangent_matrix * world_position.xyz;
    out.tangent_view_position = tangent_matrix * camera.position.xyz;
    out.tangent_light_position = tangent_matrix * point_light.position.xyz;
    return out;
}

// --- Fragment ---

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // var result: vec4<f32>;

    // Get texel from texture
    let object_diffuse_map = textureSample(t_diffuse, s_diffuse, in.texture_coordinates);
    let object_normal_map = textureSample(t_normal, s_normal, in.texture_coordinates);

    // result = object_diffuse_map;

    // Ambient Light
    let ambient_color = ambient_light.color * ambient_light.strength;
    // result *= vec4<f32>(ambient_color, 1.0);

    // Point Light
    // if point_light.enabled == u32(1) {
        let tangent_normal = object_normal_map.xyz * 2.0 - 1.0;
        let light_dir = normalize(in.tangent_light_position - in.tangent_position);
        let view_dir = normalize(in.tangent_view_position - in.tangent_position);
        let half_dir = normalize(view_dir + light_dir);

        let diffuse_strength = max(dot(tangent_normal, light_dir), 0.0);
        let diffuse_color = point_light.color.xyz * diffuse_strength;
        // result *= vec4<f32>(diffuse_color, 1.0);

        let specular_strength = pow(max(dot(tangent_normal, half_dir), 0.0), 32.0);
        let specular_color = specular_strength * point_light.color.xyz;
        // result *= vec4<f32>(specular_color, 1.0);
    // }

    // return vec4<f32>(result);
    let result = (ambient_color + diffuse_color + specular_color) * object_diffuse_map.xyz;
    return vec4<f32>(result, object_diffuse_map.a);
}
