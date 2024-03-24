use akimo_project::{
    app::App,
    error::RuntimeError,
    logging::*,
    runtime::{Runtime, RuntimeSettings},
};
use wgpu::{
    Adapter, Color, CommandEncoderDescriptor, Device, FragmentState, LoadOp, MultisampleState,
    Operations, PipelineLayout, PipelineLayoutDescriptor, PrimitiveState, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    ShaderModule, ShaderModuleDescriptor, ShaderSource, StoreOp, SurfaceConfiguration, TextureView,
    VertexState,
};
use winit::event::WindowEvent;

pub struct TriangleApp {
    shader: ShaderModule,
    pipeline_layout: PipelineLayout,
    pipeline: RenderPipeline,
}

impl App for TriangleApp {
    fn init(
        config: &SurfaceConfiguration,
        adapter: &Adapter,
        device: &Device,
        queue: &Queue,
    ) -> Self
    where
        Self: Sized,
    {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
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
            shader,
            pipeline_layout,
            pipeline,
        }
    }

    fn resize(&mut self, config: &SurfaceConfiguration, device: &Device, queue: &Queue) {
        // TODO ... uhhh....
        todo!()
    }

    fn update(&mut self, _event: WindowEvent) {
        // Nothing :)
    }

    fn render(&mut self, view: &TextureView, device: &Device, queue: &Queue) {
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::GREEN),
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

        queue.submit(Some(encoder.finish()));
    }
}

fn main() -> Result<(), RuntimeError> {
    pollster::block_on(Runtime::liftoff::<TriangleApp>(RuntimeSettings::default()))
}
