use gpu_connector::{ConnectorError, GPUConnector};
use logging::debug;
use wgpu::{
    core::instance::Surface, Adapter, Backends, Device, DeviceDescriptor, Features, Instance,
    InstanceDescriptor, Limits, PowerPreference, Queue, RequestAdapterOptions, Surface,
    SurfaceConfiguration, SurfaceError,
};
use winit::window::Window;

#[derive(Debug)]
pub struct WGPUConnector {
    instance: Instance,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    #[cfg(feature = "with_surface")]
    surface: Surface,
    #[cfg(feature = "with_surface")]
    surface_configuration: SurfaceConfiguration,
}

impl WGPUConnector {
    pub async fn new() -> EngineResultWGPU<Self> {
        let instance = Self::make_instance();
        debug!("Instance: {:#?}", instance);

        Self::from_instance(instance, None).await
    }

    pub async fn new_with_surface<'surface>(surface: &Surface<'surface>) -> EngineResultWGPU<Self> {
        let instance = Self::make_instance();
        debug!("Instance: {:#?}", instance);

        Self::from_instance(instance, Some(surface)).await
    }

    async fn from_instance<'surface>(
        instance: Instance,
        compatible_surface: Option<&Surface<'surface>>,
    ) -> EngineResultWGPU<Self> {
        let adapter = Self::make_adapter(&instance, compatible_surface).await?;
        let logical_device = Self::make_device_and_queue(&adapter).await?;

        Ok(Self {
            instance,
            adapter,
            logical_device,
        })
    }

    fn make_instance() -> Instance {
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            dx12_shader_compiler: Default::default(),
            ..Default::default()
        });
        debug!("Instance: {:#?}", instance);

        instance
    }

    async fn make_adapter<'surface>(
        instance: &Instance,
        compatible_surface: Option<&Surface<'surface>>,
    ) -> EngineResultWGPU<Adapter> {
        // Retrieve fitting adapter or construct error
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                compatible_surface: compatible_surface,
                force_fallback_adapter: false,
                power_preference: PowerPreference::HighPerformance,
            })
            .await
            .ok_or(EngineError::NoAdapters);

        // Print out debug information
        debug!("Adapter chosen: {:?}", adapter);

        adapter
    }

    async fn make_device_and_queue(adapter: &Adapter) -> EngineResultWGPU<LogicalDeviceWGPU> {
        // TODO: Parameterize
        let limits = Limits {
            max_bind_groups: 7,
            ..Default::default()
        };

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: Some("Main Device"),
                    required_features: Features::empty(),
                    required_limits: limits,
                },
                None,
            )
            .await
            .map_err(|_| EngineError::RequestDeviceError)?;

        debug!("Device: {:#?}", device);
        debug!("Queue: {:#?}", queue);

        let logical_device = LogicalDevice::new(device, queue);
        Ok(logical_device)
    }
}

// impl GPUConnector<
//     Instance,
//     Adapter,
//     Device,
//     Queue,
//     #[cfg(feature = "with_surface")] Surface,
//     #[cfg(feature = "with_surface")] SurfaceConfiguration,
//     #[cfg(feature = "with_surface")] SurfaceError,
//     #[cfg(feature = "with_surface")] Window> for WGPUConnector {
//     fn instance(&self) -> &Instance {
//         &self.instance
//     }

//     fn adapter(&self) -> &Adapter {
//         &self.adapter
//     }

//     fn logical_device(&self) -> &LogicalDeviceWGPU {
//         &self.logical_device
//     }
// }

impl GPUConnector<Instance, Adapter, Device, Queue, A, B> for WGPUConnector {
    fn new() -> Result<Self, ConnectorError>
    where
        Self: Sized,
    {
        todo!()
    }

    fn instance(&self) -> &Instance {
        todo!()
    }

    fn adapter(&self) -> &Adapter {
        todo!()
    }

    fn device<'a>(&'a self) -> &'a Device
    where
        Queue: 'a,
    {
        todo!()
    }

    fn queue<'a>(&'a self) -> &'a Queue
    where
        Device: 'a,
    {
        todo!()
    }

    fn surface<'a>(&'a self) -> &'a Surface {
        todo!()
    }

    fn surface_configuration<'a>(&'a self) -> &'a SurfaceConfiguration {
        todo!()
    }
}
