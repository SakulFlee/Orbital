use log::warn;
use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BlendState, ColorTargetState, ColorWrites, Face,
    FragmentState, FrontFace, MultisampleState, PipelineLayoutDescriptor, PolygonMode,
    PrimitiveState, PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor, TextureFormat,
    VertexState,
};

use crate::{runtime::Context, shader::Shader};

use super::Vertex;

// TODO: Depth Stencil/Testing

pub struct PipelineBuilder<'a> {
    shader: Shader,
    label: Option<String>,
    bind_group_descriptions: Vec<BindGroupLayoutDescriptor<'a>>,
    primitive_topology: Option<PrimitiveTopology>,
    front_face_order: Option<FrontFace>,
    cull_mode: Option<Option<Face>>,
    polygon_mode: Option<PolygonMode>,
}

impl<'a> PipelineBuilder<'a> {
    pub fn new(shader: Shader) -> Self {
        Self {
            shader,
            label: None,
            bind_group_descriptions: Vec::new(),
            primitive_topology: None,
            front_face_order: None,
            cull_mode: None,
            polygon_mode: None,
        }
    }

    pub fn build(
        self,
        context: &Context,
        surface_texture_format: Option<TextureFormat>,
    ) -> RenderPipeline {
        let mut bind_groups = Vec::<BindGroupLayout>::new();
        for bind_group_desc in self.bind_group_descriptions {
            bind_groups.push(context.device().create_bind_group_layout(&bind_group_desc));
        }

        let layout = context
            .device()
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: self
                    .label
                    .as_ref()
                    .map(|x| format!("{x} [Layout]"))
                    .as_deref(),
                bind_group_layouts: &bind_groups
                    .iter()
                    .collect::<Vec<&BindGroupLayout>>()
                    .as_slice(),
                push_constant_ranges: &[],
            });

        let vertex_state = VertexState {
            module: &self.shader.module(),
            entry_point: &self.shader.entrypoint_vertex(),
            buffers: &[Vertex::descriptor()],
        };

        let mut color_target_state: Vec<Option<ColorTargetState>> = Vec::new();
        let mut fragment_state: Option<FragmentState> = None;
        if let Some(entrypoint) = self.shader.entrypoint_fragment() {
            if let Some(surface_format) = surface_texture_format {
                color_target_state.push(Some(ColorTargetState {
                    format: surface_format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                }));
            }

            fragment_state = Some(FragmentState {
                module: &self.shader.module(),
                entry_point: &entrypoint,
                targets: &color_target_state,
            });
        }

        let primitive_state = PrimitiveState {
            topology: self
                .primitive_topology
                .unwrap_or(PrimitiveTopology::TriangleList),
            strip_index_format: None,
            front_face: self.front_face_order.unwrap_or(FrontFace::Ccw),
            cull_mode: self.cull_mode.unwrap_or(Some(Face::Back)),
            unclipped_depth: false,
            polygon_mode: self.polygon_mode.unwrap_or(PolygonMode::Fill),
            conservative: false,
        };

        let multisample = MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        };

        context
            .device()
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: self.label.as_deref(),
                layout: Some(&layout),
                vertex: vertex_state,
                fragment: fragment_state,
                primitive: primitive_state,
                depth_stencil: None,
                multisample,
                multiview: None,
            })
    }

    pub fn with_defaults(self) -> Self {
        self.with_primitive_topology(PrimitiveTopology::TriangleList)
            .with_front_face_order(FrontFace::Ccw)
            .with_cull_mode(Some(Face::Back))
            .with_polygon_mode(PolygonMode::Fill)
    }

    pub fn with_label(mut self, label: &str) -> Self {
        if self.label.is_some() {
            warn!("Label has already been set and will be OVERWRITTEN!");
        }

        self.label = Some(label.into());
        self
    }

    pub fn with_primitive_topology(mut self, topology: PrimitiveTopology) -> Self {
        if self.primitive_topology.is_some() {
            warn!("Primitive topology has already been set and will be OVERWRITTEN!");
        }

        self.primitive_topology = Some(topology);
        self
    }

    pub fn with_front_face_order(mut self, face: FrontFace) -> Self {
        if self.front_face_order.is_some() {
            warn!("Front face order has already been set and will be OVERWRITTEN!");
        }

        self.front_face_order = Some(face);
        self
    }

    pub fn with_cull_mode(mut self, face: Option<Face>) -> Self {
        if self.cull_mode.is_some() {
            warn!("Cull mode has already been set and will be OVERWRITTEN!");
        }

        self.cull_mode = Some(face);
        self
    }

    pub fn with_polygon_mode(mut self, mode: PolygonMode) -> Self {
        if self.polygon_mode.is_some() {
            warn!("Polygon mode has already been set and will be OVERWRITTEN!");
        }

        self.polygon_mode = Some(mode);
        self
    }

    pub fn with_binding_group(mut self, descriptor: BindGroupLayoutDescriptor<'a>) -> Self {
        self.bind_group_descriptions.push(descriptor);
        self
    }
}
