use std::ops::Deref;

use compute_connector_trait::ComputeConnectorTrait;
use compute_connector_wgpu::ComputeConnectorWGPU;
use engine_error::EngineError;
use engine_result_wgpu::EngineResultWGPU;
use graphic_connector_trait::GraphicConnectorTrait;
use logical_device::LogicalDevice;
use wgpu::{Adapter, Device, Instance, Queue, Surface, SurfaceConfiguration};

pub struct GraphicConnectorWGPU<'surface> {
    compute_connector: ComputeConnectorWGPU,
    surface: Surface<'surface>,
    surface_configuration: SurfaceConfiguration,
}

impl<'surface> GraphicConnectorWGPU<'surface> {
    pub async fn from_surface(
        surface: Surface<'surface>,
        size: (u32, u32),
    ) -> EngineResultWGPU<Self> {
        let compute_connector = ComputeConnectorWGPU::new_with_surface(&surface).await?;

        let surface_configuration = surface
            .get_default_config(compute_connector.adapter(), size.0, size.1)
            .ok_or(EngineError::CreateSurfaceError)?;
        surface.configure(compute_connector.device(), &surface_configuration);

        todo!()
    }

    pub async fn from_connector(
        compute_connector: ComputeConnectorWGPU,
        surface: Surface<'surface>,
        size: (u32, u32),
    ) -> EngineResultWGPU<Self> {
        let compute_connector = ComputeConnectorWGPU::new_with_surface(&surface).await?;

        let surface_configuration = surface
            .get_default_config(compute_connector.adapter(), size.0, size.1)
            .ok_or(EngineError::CreateSurfaceError)?;
        surface.configure(compute_connector.device(), &surface_configuration);

        todo!()
    }
}

impl<'surface> ComputeConnectorTrait<Instance, Adapter, Device, Queue>
    for GraphicConnectorWGPU<'surface>
{
    fn instance(&self) -> &Instance {
        self.compute_connector.instance()
    }

    fn adapter(&self) -> &Adapter {
        self.compute_connector.adapter()
    }

    fn logical_device(&self) -> &LogicalDevice<Device, Queue> {
        self.compute_connector.logical_device()
    }
}

impl<'surface>
    GraphicConnectorTrait<
        'surface,
        Instance,
        Adapter,
        Device,
        Queue,
        Surface<'surface>,
        SurfaceConfiguration,
    > for GraphicConnectorWGPU<'surface>
{
    fn surface(&'surface self) -> &'surface Surface<'surface> {
        &self.surface
    }

    fn surface_configuration(&self) -> &SurfaceConfiguration {
        &self.surface_configuration
    }

    fn set_surface_configuration(&mut self, surface_configuration: SurfaceConfiguration) {
        self.surface_configuration = surface_configuration;
    }
}

impl Deref for GraphicConnectorWGPU<'_> {
    type Target = ComputeConnectorWGPU;

    fn deref(&self) -> &Self::Target {
        &self.compute_connector
    }
}
