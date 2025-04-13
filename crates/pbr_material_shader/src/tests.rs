use material_shader::{MaterialShader, MaterialShaderDescriptor, VertexStageLayout};
use shader::ShaderSource;
use wgpu::{Face, FrontFace, PolygonMode, PrimitiveTopology};

use crate::{PBRMaterial, PBRMaterialDescriptor, PBRMaterialShaderDescriptor};

#[test]
fn default() {
    let _pbr_material = PBRMaterial::default();
}

#[test]
fn alias_pbr_material() {
    let _pbr_material = PBRMaterial::default();
}

#[test]
fn alias_pbr_material_descriptor() {
    let _pbr_material = PBRMaterialDescriptor::default();
}

#[test]
fn alias_pbr_material_shader_descriptor() {
    let _pbr_material = PBRMaterialShaderDescriptor::default();
}

#[test]
fn default_conversion_to_material_shader() {
    let pbr_material = PBRMaterial::default();
    let _material_shader: MaterialShaderDescriptor = pbr_material.into();
}

#[test]
fn default_conversion_to_material_shader_check_variable_count() {
    let pbr_material = PBRMaterial::default();
    let material_shader: MaterialShaderDescriptor = pbr_material.into();

    assert_eq!(7, material_shader.variables.len());
}

#[test]
fn default_conversion_to_material_shader_check_name_persistence() {
    const NAME: &'static str = "Test";

    let mut pbr_material = PBRMaterial::default();
    pbr_material.name = Some(NAME);

    let material_shader: MaterialShaderDescriptor = pbr_material.into();
    assert_eq!(NAME, material_shader.name.expect("Name missing"));
}

#[test]
fn default_conversion_to_material_shader_check_shader_source_persistence() {
    const SHADER_SOURCE: ShaderSource = ShaderSource::String("Testing");

    let mut pbr_material = PBRMaterial::default();
    pbr_material.custom_material_shader = Some(MaterialShaderDescriptor {
        shader_source: SHADER_SOURCE,
        ..Default::default()
    });

    let material_shader: MaterialShaderDescriptor = pbr_material.into();
    assert_eq!(SHADER_SOURCE, material_shader.shader_source);
}

#[test]
fn default_conversion_to_material_shader_check_entrypoint_vertex_persistence() {
    const ENTRYPOINT_VERTEX: &'static str = "Testing";

    let mut pbr_material = PBRMaterial::default();
    pbr_material.custom_material_shader = Some(MaterialShaderDescriptor {
        entrypoint_vertex: ENTRYPOINT_VERTEX,
        ..Default::default()
    });

    let material_shader: MaterialShaderDescriptor = pbr_material.into();
    assert_eq!(ENTRYPOINT_VERTEX, material_shader.entrypoint_vertex);
}

#[test]
fn default_conversion_to_material_shader_check_entrypoint_fragment_persistence() {
    const ENTRYPOINT_FRAGMENT: &'static str = "Testing";

    let mut pbr_material = PBRMaterial::default();
    pbr_material.custom_material_shader = Some(MaterialShaderDescriptor {
        entrypoint_fragment: ENTRYPOINT_FRAGMENT,
        ..Default::default()
    });

    let material_shader: MaterialShaderDescriptor = pbr_material.into();
    assert_eq!(ENTRYPOINT_FRAGMENT, material_shader.entrypoint_fragment);
}

#[test]
fn default_conversion_to_material_shader_check_vertex_stage_layouts_persistence() {
    let vertex_stage_layouts: Vec<VertexStageLayout> = vec![
        VertexStageLayout::SimpleVertexData,
        VertexStageLayout::SimpleVertexData,
        VertexStageLayout::SimpleVertexData,
        VertexStageLayout::SimpleVertexData,
    ];

    let mut pbr_material = PBRMaterial::default();
    pbr_material.custom_material_shader = Some(MaterialShaderDescriptor {
        vertex_stage_layouts: vertex_stage_layouts.clone(),
        ..Default::default()
    });

    let material_shader: MaterialShaderDescriptor = pbr_material.into();
    assert_eq!(
        vertex_stage_layouts.len(),
        material_shader.vertex_stage_layouts.len()
    );
    assert_eq!(vertex_stage_layouts, material_shader.vertex_stage_layouts);
}

#[test]
fn default_conversion_to_material_shader_check_primitive_topology_persistence() {
    const PRIMITIVE_TOPOLOGY: PrimitiveTopology = PrimitiveTopology::PointList;

    let mut pbr_material = PBRMaterial::default();
    pbr_material.custom_material_shader = Some(MaterialShaderDescriptor {
        primitive_topology: PRIMITIVE_TOPOLOGY,
        ..Default::default()
    });

    let material_shader: MaterialShaderDescriptor = pbr_material.into();
    assert_eq!(PRIMITIVE_TOPOLOGY, material_shader.primitive_topology);
}

#[test]
fn default_conversion_to_material_shader_check_front_face_order_persistence() {
    const FRONT_FACE_ORDER: FrontFace = FrontFace::Ccw;

    let mut pbr_material = PBRMaterial::default();
    pbr_material.custom_material_shader = Some(MaterialShaderDescriptor {
        front_face_order: FRONT_FACE_ORDER,
        ..Default::default()
    });

    let material_shader: MaterialShaderDescriptor = pbr_material.into();
    assert_eq!(FRONT_FACE_ORDER, material_shader.front_face_order);
}

#[test]
fn default_conversion_to_material_shader_check_cull_mode_persistence() {
    const CULL_MODE: Option<Face> = Some(Face::Front);

    let mut pbr_material = PBRMaterial::default();
    pbr_material.custom_material_shader = Some(MaterialShaderDescriptor {
        cull_mode: CULL_MODE,
        ..Default::default()
    });

    let material_shader: MaterialShaderDescriptor = pbr_material.into();
    assert_eq!(CULL_MODE, material_shader.cull_mode);
}

#[test]
fn default_conversion_to_material_shader_check_polygon_mode_persistence() {
    const POLYGON_MODE: PolygonMode = PolygonMode::Point;

    let mut pbr_material = PBRMaterial::default();
    pbr_material.custom_material_shader = Some(MaterialShaderDescriptor {
        polygon_mode: POLYGON_MODE,
        ..Default::default()
    });

    let material_shader: MaterialShaderDescriptor = pbr_material.into();
    assert_eq!(POLYGON_MODE, material_shader.polygon_mode);
}

#[test]
fn default_conversion_to_material_shader_check_depth_stencil_persistence() {
    const DEPTH_STENCIL: bool = false;

    let mut pbr_material = PBRMaterial::default();
    pbr_material.custom_material_shader = Some(MaterialShaderDescriptor {
        depth_stencil: DEPTH_STENCIL,
        ..Default::default()
    });

    let material_shader_descriptor: MaterialShaderDescriptor = pbr_material.into();
    assert_eq!(DEPTH_STENCIL, material_shader_descriptor.depth_stencil);
}
