use engine_error::EngineError;
use engine_result_wgpu::EngineResultWGPU;
use logging::debug;
use logical_device::LogicalDevice;
use logical_device_wgpu::LogicalDeviceWGPU;
use wgpu::{
    Adapter, Backends, Device, DeviceDescriptor, Features, Instance, InstanceDescriptor, Limits,
    PowerPreference, Queue, RequestAdapterOptions,
};

pub use compute_connector_trait::ComputeConnectorTrait;

#[derive(Debug)]
pub struct ComputeConnectorWGPU {
    instance: Instance,
    adapter: Adapter,
    logical_device: LogicalDeviceWGPU,
}

impl ComputeConnectorWGPU {
    pub async fn new() -> EngineResultWGPU<Self> {
        let instance = Self::make_instance();
        debug!("Instance: {:#?}", instance);

        Self::from_instance(instance).await
    }

    pub async fn from_instance(instance: Instance) -> EngineResultWGPU<Self> {
        let adapter = Self::make_adapter(&instance).await?;
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

    async fn make_adapter(instance: &Instance) -> EngineResultWGPU<Adapter> {
        // Retrieve fitting adapter or construct error
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                compatible_surface: None,
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

impl ComputeConnectorTrait<Instance, Adapter, Device, Queue> for ComputeConnectorWGPU {
    fn instance(&self) -> &Instance {
        &self.instance
    }

    fn adapter(&self) -> &Adapter {
        &self.adapter
    }

    fn logical_device(&self) -> &LogicalDeviceWGPU {
        &self.logical_device
    }
}
