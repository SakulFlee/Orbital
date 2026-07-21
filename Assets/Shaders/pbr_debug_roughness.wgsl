struct CameraUniform {
    position: vec3<f32>,
    view_projection_matrix: mat4x4<f32>,
    perspective_view_projection_matrix: mat4x4<f32>,
    view_projection_transposed: mat4x4<f32>,
    perspective_projection_invert: mat4x4<f32>,
    global_gamma: f32,
}

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
    @location(0) uv: vec2<f32>,
}

struct PBRFactors {
    albedo_factor: vec3<f32>,
    metallic_factor: f32,
    roughness_factor: f32,
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;

@group(1) @binding(6) var roughness_texture: texture_2d<f32>;
@group(1) @binding(7) var roughness_sampler: sampler;

@group(1) @binding(12) var<uniform> pbr_factors: PBRFactors;

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

    let world_position = model_space_matrix * vec4<f32>(vertex.position, 1.0);

    var out: FragmentData;
    out.position = camera.perspective_view_projection_matrix * world_position;
    out.uv = vertex.uv;

    return out;
}

@fragment
fn entrypoint_fragment(in: FragmentData) -> @location(0) vec4<f32> {
    // Test 1: constant red — if visible, shader is loaded and culling works
    return vec4<f32>(1.0, 0.0, 1.0, 1.0);
}
