struct CameraUniform {
    view_proj: mat4x4<f32>,
}

struct AmbientLight {
    color: vec3<f32>,
}
@group(2) @binding(0) 
var<uniform> ambient_light: AmbientLight;

struct PointLight {
    position: vec3<f32>,
    color: vec3<f32>,
}
@group(3) @binding(0) 
var<uniform> point_light: PointLight;

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
}

// Internal struct for Vertex Output data
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
};

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;

@group(0) @binding(1)
var s_diffuse: sampler;

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

// Vertex Main
// 
// Gets called per vertex in the queue.
// Should position the vertex on the screen.
@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    var out: VertexOutput;

    out.tex_coords = model.tex_coords;

    out.world_normal = model.normal;

    var world_position: vec4<f32> = model_matrix * vec4<f32>(model.position, 1.0);
    out.world_position = world_position.xyz;

    out.clip_position = camera.view_proj * world_position;

    return out;
}

// Fragment Main
//
// Gets called per pixel and vertex and returns a colour for that pixel.
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Texture
    let object_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);

    // Ambient Light
    let ambient_strength = 0.1;
    let ambient_color = ambient_light.color * ambient_strength;

    // Point Light
    let light_dir = normalize(point_light.position - in.world_position);

    let diffuse_strength = max(dot(in.world_normal, light_dir), 0.0);
    let diffuse_color = point_light.color * diffuse_strength;

    // Combine lights and textures!
    let result = (ambient_color + diffuse_color) * object_color.xyz;

    return vec4<f32>(result, object_color.a);
}
