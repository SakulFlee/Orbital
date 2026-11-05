//! World-space SDF text built on the billboard component.

use std::sync::Arc;

use cgmath::{Vector2, Vector3};
use orbital_ecs_bridge::{EditableField, EditableFields, EditableValue};
use orbital_material_shader::{
    AlphaMode, MaterialShaderDescriptor, RenderQueue, VertexStageLayout,
};
use orbital_mesh::MeshDescriptor;
use orbital_shader_core::VariableType;
use orbital_text::{DefaultFont, FontData, SdfAtlas, TextConfig, generate_text_mesh};
use orbital_texture::{BufferDescriptor, FilterMode, TextureDescriptor, TextureSize};
use orbital_vertex::Vertex;
use wgpu::{
    BlendState, CompareFunction, SamplerBindingType, TextureDimension, TextureFormat,
    TextureSampleType, TextureUsages, TextureViewDimension,
};

use crate::Billboard;

const TEXT_NODES: &[&str] = &["pbr_vertex"];
const TEXT_WGSL: &str = include_str!("billboard_text.wgsl");

/// Editable source data for a world-space text billboard.
///
/// The rendered [`Billboard`] contains the generated glyph mesh and atlas
/// material; this component keeps the authoring values so the ECS inspector
/// can edit the text without losing them. [`Text::dirty`] tells the billboard
/// module when to rebuild the generated mesh.
#[derive(Debug, Clone)]
pub struct Text {
    /// The text to render.
    pub value: String,
    /// Font size in atlas pixels.
    pub font_size: f32,
    /// Text colour.
    pub color: [f32; 4],
    /// Optional background colour.
    pub background: Option<[f32; 4]>,
    /// World size of one font pixel.
    pub world_scale: f32,
    /// Whether the text uses the opaque or blended render path.
    pub alpha_mode: AlphaMode,
    /// Set when the source values changed and the glyph mesh must rebuild.
    dirty: bool,
}

impl Text {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            font_size: 128.0,
            color: [1.0, 1.0, 1.0, 1.0],
            background: None,
            world_scale: 0.00225,
            alpha_mode: AlphaMode::Blend,
            dirty: true,
        }
    }

    pub fn with_font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self.mark_dirty();
        self
    }

    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self.mark_dirty();
        self
    }

    pub fn with_background(mut self, background: Option<[f32; 4]>) -> Self {
        self.background = background;
        self.mark_dirty();
        self
    }

    pub fn with_world_scale(mut self, world_scale: f32) -> Self {
        self.world_scale = world_scale;
        self.mark_dirty();
        self
    }

    pub fn with_alpha_mode(mut self, alpha_mode: AlphaMode) -> Self {
        self.alpha_mode = alpha_mode;
        self.mark_dirty();
        self
    }

    pub(crate) fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub(crate) fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Builds the generated billboard content. Position, size and mode are
    /// owned by the entity's [`Billboard`] and preserved by the rebuild system.
    pub(crate) fn build_billboard(&self) -> Billboard {
        let mut font = DefaultFont::load();
        let mut billboard = text_billboard_with_background(
            &self.value,
            &mut font,
            &TextConfig {
                font_size: self.font_size.max(1.0),
                color: self.color,
                ..TextConfig::default()
            },
            self.world_scale.max(0.0001),
            None,
            self.background,
            self.alpha_mode,
        );
        // The generated helper places the billboard at the origin; the
        // existing Billboard supplies the transform.
        billboard.position = cgmath::Point3::new(0.0, 0.0, 0.0);
        billboard
    }
}

impl EditableFields for Text {
    fn editable_fields(&self) -> Vec<EditableField> {
        vec![
            EditableField {
                id: "text".to_string(),
                label: "text".to_string(),
                value: self.value.clone(),
            },
            EditableField {
                id: "font_size".to_string(),
                label: "font_size".to_string(),
                value: self.font_size.to_edit_string(),
            },
            EditableField {
                id: "color".to_string(),
                label: "color (RGBA)".to_string(),
                value: color_text(self.color),
            },
            EditableField {
                id: "world_scale".to_string(),
                label: "world_scale".to_string(),
                value: self.world_scale.to_edit_string(),
            },
        ]
    }

    fn set_editable_field(&mut self, id: &str, input: &str) -> Result<(), String> {
        match id {
            "text" => self.value = input.to_string(),
            "font_size" => self.font_size.set_from_str(input)?,
            "color" => self.color = parse_color(input)?,
            "world_scale" => self.world_scale.set_from_str(input)?,
            _ => return Err(format!("unknown text field `{id}`")),
        }
        self.mark_dirty();
        Ok(())
    }
}

fn color_text(color: [f32; 4]) -> String {
    color
        .iter()
        .map(|component| format!("{component:?}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_color(input: &str) -> Result<[f32; 4], String> {
    let values = input
        .split([' ', ','])
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| {
            part.parse::<f32>()
                .map_err(|_| format!("`{part}` is not a number"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    if values.len() != 4 {
        return Err(format!(
            "expected 4 colour components, got {}",
            values.len()
        ));
    }

    Ok([values[0], values[1], values[2], values[3]])
}

/// Builds a world-space SDF text billboard.
///
/// `world_scale` is the world size of one font pixel. `outline` is
/// `(color, sdf_width)`, where the width is in normalized SDF units.
pub fn text_billboard(
    text: &str,
    font: &mut FontData,
    config: &TextConfig,
    world_scale: f32,
    outline: Option<([f32; 4], f32)>,
    alpha_mode: AlphaMode,
) -> Billboard {
    text_billboard_with_background(text, font, config, world_scale, outline, None, alpha_mode)
}

/// Like [`text_billboard`], with an optional background color.
pub fn text_billboard_with_background(
    text: &str,
    font: &mut FontData,
    config: &TextConfig,
    world_scale: f32,
    outline: Option<([f32; 4], f32)>,
    background: Option<[f32; 4]>,
    alpha_mode: AlphaMode,
) -> Billboard {
    let characters = if text.is_empty() {
        " ".to_string()
    } else {
        text.chars().collect()
    };
    let atlas = SdfAtlas::build_atlas(
        font,
        config.font_size.max(1.0) as u32,
        &characters,
        8.0,
        1024,
        1024,
    );
    let vertices_2d = generate_text_mesh(text, font, config, Some(&atlas));

    let mut min = Vector2::new(f32::MAX, f32::MAX);
    let mut max = Vector2::new(f32::MIN, f32::MIN);
    for vertex in &vertices_2d {
        min.x = min.x.min(vertex.position[0]);
        min.y = min.y.min(vertex.position[1]);
        max.x = max.x.max(vertex.position[0]);
        max.y = max.y.max(vertex.position[1]);
    }
    let center = (min + max) * 0.5;

    let normal = Vector3::unit_z();
    let tangent = Vector3::unit_x();
    let bitangent = Vector3::unit_y();
    let background_vertices = if background.is_some() && !vertices_2d.is_empty() {
        4
    } else {
        0
    };
    let mut vertices = Vec::with_capacity(vertices_2d.len() + background_vertices);
    let mut indices = Vec::with_capacity(vertices_2d.len() + background_vertices);

    if background.is_some() && !vertices_2d.is_empty() {
        let padding = config.font_size * 0.2;
        let x0 = min.x - padding - center.x;
        let x1 = max.x + padding - center.x;
        let y0 = -(max.y + padding - center.y);
        let y1 = -(min.y - padding - center.y);

        vertices.extend([
            Vertex::new_with_bitangent(
                Vector3::new(x0, y0, 0.0),
                normal,
                tangent,
                bitangent,
                Vector2::new(2.0, 2.0),
            ),
            Vertex::new_with_bitangent(
                Vector3::new(x1, y0, 0.0),
                normal,
                tangent,
                bitangent,
                Vector2::new(2.0, 2.0),
            ),
            Vertex::new_with_bitangent(
                Vector3::new(x1, y1, 0.0),
                normal,
                tangent,
                bitangent,
                Vector2::new(2.0, 2.0),
            ),
            Vertex::new_with_bitangent(
                Vector3::new(x0, y1, 0.0),
                normal,
                tangent,
                bitangent,
                Vector2::new(2.0, 2.0),
            ),
        ]);
        indices.extend([0, 1, 2, 0, 2, 3]);
    }

    let glyph_start = vertices.len() as u32;
    for vertex in &vertices_2d {
        vertices.push(Vertex::new_with_bitangent(
            Vector3::new(
                vertex.position[0] - center.x,
                -(vertex.position[1] - center.y),
                0.0,
            ),
            normal,
            tangent,
            bitangent,
            Vector2::new(vertex.texcoord[0], vertex.texcoord[1]),
        ));
    }
    indices.extend(glyph_start..glyph_start + vertices_2d.len() as u32);

    let texture = TextureDescriptor::Data {
        pixels: atlas.data,
        size: TextureSize {
            width: atlas.width,
            height: atlas.height,
            ..Default::default()
        },
        usages: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        format: TextureFormat::Rgba8Unorm,
        texture_dimension: TextureDimension::D2,
        texture_view_dimension: TextureViewDimension::D2,
        filter_mode: FilterMode::linear(),
    };

    let material = text_material(texture, config.color, outline, background, alpha_mode);
    Billboard::from_material(
        Vector2::new(world_scale.max(0.0001), world_scale.max(0.0001)),
        material,
    )
    .with_mesh(Arc::new(MeshDescriptor::new(vertices, indices)))
}

fn text_material(
    texture: TextureDescriptor,
    color: [f32; 4],
    outline: Option<([f32; 4], f32)>,
    background: Option<[f32; 4]>,
    alpha_mode: AlphaMode,
) -> Arc<MaterialShaderDescriptor> {
    let (outline_color, outline_width) = outline.unwrap_or(([0.0, 0.0, 0.0, 0.0], 0.0));
    let background_color = background.unwrap_or([0.0, 0.0, 0.0, 0.0]);
    let cutoff = alpha_mode.cutoff().unwrap_or(0.0);

    let mut params = [0u8; 64];
    write_color(&mut params[0..16], color);
    write_color(&mut params[16..32], outline_color);
    write_color(&mut params[32..48], background_color);
    params[48..52].copy_from_slice(&outline_width.to_le_bytes());
    params[52..56].copy_from_slice(&0.02f32.to_le_bytes()); // smoothing
    params[56..60].copy_from_slice(&cutoff.to_le_bytes());

    let (blend, depth_write, render_queue) = if alpha_mode == AlphaMode::Blend {
        (BlendState::ALPHA_BLENDING, false, RenderQueue::Transparent)
    } else {
        (BlendState::REPLACE, true, RenderQueue::Opaque)
    };

    Arc::new(MaterialShaderDescriptor {
        name: Some("Billboard Text Material".into()),
        nodes: TEXT_NODES,
        raw_source: Some(TEXT_WGSL.into()),
        variables: vec![
            VariableType::Texture {
                descriptor: Box::new(texture),
                sample_type: TextureSampleType::Float { filterable: true },
                sampler_binding_type: SamplerBindingType::Filtering,
            },
            VariableType::Buffer(BufferDescriptor {
                data: params.to_vec(),
                ..Default::default()
            }),
        ],
        vertex_stage_layouts: Some(vec![
            VertexStageLayout::ComplexVertexData,
            VertexStageLayout::InstanceData,
        ]),
        cull_mode: None,
        alpha_mode,
        blend,
        depth_write,
        depth_compare: CompareFunction::Less,
        render_queue,
        cast_shadows: false,
        receive_shadows: false,
        ..MaterialShaderDescriptor::default()
    })
}

fn write_color(target: &mut [u8], color: [f32; 4]) {
    for (index, value) in color.iter().enumerate() {
        target[index * 4..index * 4 + 4].copy_from_slice(&value.to_le_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use orbital_text::DefaultFont;

    #[test]
    fn text_billboard_builds_mesh_and_transparent_material() {
        let mut font = DefaultFont::load();
        let config = TextConfig {
            font_size: 24.0,
            color: [1.0, 0.9, 0.8, 1.0],
            ..Default::default()
        };

        let billboard = text_billboard(
            "Orbital",
            &mut font,
            &config,
            0.01,
            Some(([0.0, 0.0, 0.0, 0.8], 0.1)),
            AlphaMode::Blend,
        );

        let mesh = billboard.mesh.as_ref().expect("text mesh");
        assert!(!mesh.vertices.is_empty());
        assert_eq!(mesh.vertices.len(), mesh.indices.len());
        assert_eq!(billboard.material.render_queue, RenderQueue::Transparent);
        assert!(!billboard.material.depth_write);
        assert!(billboard.material.entrypoint_fragment_gbuffer.is_none());
    }

    #[test]
    fn text_source_fields_are_editable_and_mark_the_mesh_dirty() {
        let mut text = Text::new("Room 1")
            .with_font_size(64.0)
            .with_color([1.0, 0.5, 0.25, 1.0])
            .with_world_scale(0.01);
        text.clear_dirty();

        let ids = text
            .editable_fields()
            .into_iter()
            .map(|field| field.id)
            .collect::<Vec<_>>();
        assert_eq!(ids, ["text", "font_size", "color", "world_scale"]);

        text.set_editable_field("text", "Metallic Materials")
            .expect("text edit failed");
        assert_eq!(text.value, "Metallic Materials");
        assert!(text.is_dirty());

        text.set_editable_field("color", "0.1 0.2 0.3 1.0")
            .expect("colour edit failed");
        assert_eq!(text.color, [0.1, 0.2, 0.3, 1.0]);
        assert!(text.set_editable_field("color", "1 2 3").is_err());
    }

    #[test]
    fn rebuilding_text_preserves_generated_content() {
        let text = Text::new("Orbital").with_font_size(32.0);
        let billboard = text.build_billboard();
        assert!(billboard.mesh.is_some());
        assert!(billboard.material.entrypoint_fragment_gbuffer.is_none());
    }
}
