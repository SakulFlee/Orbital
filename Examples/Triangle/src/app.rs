use akimo_runtime::{
    runtime::{App, Context},
    wgpu::{
        Color, CommandEncoderDescriptor, FragmentState, LoadOp, MultisampleState, Operations,
        PipelineLayoutDescriptor, PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor,
        RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, StoreOp,
        SurfaceConfiguration, TextureView, VertexState,
    },
};

pub struct TriangleApp {
    // Note: Curiously, the following two variables don't have to be stored if
    // we are just referencing them. Uncomment the below and the end of
    // Self::init if you need access to them :)
    //
    // shader: ShaderModule,
    // pipeline_layout: PipelineLayout,
    pipeline: RenderPipeline,
}

impl App for TriangleApp {
    fn init(config: &SurfaceConfiguration, context: &Context) -> Self
    where
        Self: Sized,
    {
        let shader = context
            .device()
            .create_shader_module(ShaderModuleDescriptor {
                label: None,
                source: ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            });

        let pipeline_layout = context
            .device()
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let pipeline = context
            .device()
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                fragment: Some(FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(config.format.into())],
                }),
                primitive: PrimitiveState::default(),
                depth_stencil: None,
                multisample: MultisampleState::default(),
                multiview: None,
            });

        Self {
            // Note: Check variable description in struct declaration!
            //
            // shader,
            // pipeline_layout,
            pipeline,
        }
    }

    fn resize(&mut self, _config: &SurfaceConfiguration, _context: &Context) {
        // Nothing needed for this example!
        // Later, this should be used to update the uniform buffer matrix.
    }

    fn update(&mut self) {
        // Nothing needed for this example!
        // All events that we care about are already taken care of.
    }

    fn render(&mut self, view: &TextureView, context: &Context) {
        let mut encoder = context
            .device()
            .create_command_encoder(&CommandEncoderDescriptor { label: None });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_pipeline(&self.pipeline);
            render_pass.draw(0..3, 0..1);
        }

        context.queue().submit(Some(encoder.finish()));
    }
}
