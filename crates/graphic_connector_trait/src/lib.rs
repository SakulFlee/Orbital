use compute_connector_trait::ComputeConnectorTrait;

pub trait GraphicConnectorTrait<
    'surface,
    Instance,
    Adapter,
    Device,
    Queue,
    Surface,
    SurfaceConfiguration,
>: ComputeConnectorTrait<Instance, Adapter, Device, Queue>
{
    fn surface(&'surface self) -> &'surface Surface;

    fn surface_configuration(&self) -> &SurfaceConfiguration;

    fn set_surface_configuration(&mut self, surface_configuration: SurfaceConfiguration);
}
