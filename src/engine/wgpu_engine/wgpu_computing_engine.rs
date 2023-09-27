use wgpu::{
    Adapter, Backend, Backends, DeviceDescriptor, Features, Instance, InstanceDescriptor, Limits,
};

use crate::engine::{EngineError, EngineResult, LogicalDevice, TComputingEngine};

pub struct WGPUComputingEngine {
    instance: Instance,
    adapter: Adapter,
    logical_device: LogicalDevice,
}

impl WGPUComputingEngine {
    pub fn new() -> EngineResult<Self> {
        Self::new_with_score_function(|_| 0)
    }

    pub fn new_with_score_function(
        score_adapter: impl Fn(&Adapter) -> i32,
    ) -> Result<Self, EngineError> {
        let instance = Self::make_instance();
        log::debug!("Instance: {:#?}", instance);

        Self::from_instance(instance, score_adapter)
    }

    pub fn from_instance(
        instance: Instance,
        score_adapter: impl Fn(&Adapter) -> i32,
    ) -> Result<Self, EngineError> {
        let adapter = Self::make_adapter(&instance, score_adapter)?;
        let logical_device = Self::make_device_and_queue(&adapter)?;

        Ok(Self {
            instance,
            adapter,
            logical_device,
        })
    }

    pub(crate) fn make_instance() -> Instance {
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            dx12_shader_compiler: Default::default(),
        });
        log::debug!("Instance: {:#?}", instance);

        instance
    }

    fn make_adapter(
        instance: &Instance,
        score_adapter: impl Fn(&Adapter) -> i32,
    ) -> EngineResult<Adapter> {
        // Rank all adapters
        let mut adapters = Self::rank_adapters(instance, score_adapter);

        // Print out debug information
        log::debug!("The following adapters are compatible:");
        for (i, (adapter, score)) in adapters.iter().enumerate() {
            log::debug!("#{}, Score: {} - {:?}", i, score, adapter.get_info());
        }

        // Pick the last adapter.
        // After scoring and sorting, the highest score should be the
        // best option
        let (chosen_adapter, chosen_score) = adapters.pop().ok_or(EngineError::NoAdapters)?;
        log::info!(
            "Selected Adapter '{:?}' with a score of {}",
            chosen_adapter.get_info(),
            chosen_score
        );

        Ok(chosen_adapter)
    }

    fn rank_adapters(
        instance: &Instance,
        score_adapter: impl Fn(&Adapter) -> i32,
    ) -> Vec<(Adapter, i32)> {
        let mut adapters: Vec<(Adapter, i32)> = instance
            .enumerate_adapters(Backends::all())
            .map(|x| {
                fn score_type(adapter: &Adapter) -> i32 {
                    match adapter.get_info().device_type {
                        wgpu::DeviceType::DiscreteGpu => 5000,
                        wgpu::DeviceType::IntegratedGpu => 2500,
                        wgpu::DeviceType::VirtualGpu => 1000,
                        wgpu::DeviceType::Cpu => 0,
                        wgpu::DeviceType::Other => i32::MIN,
                    }
                }

                fn score_backend(adapter: &Adapter) -> i32 {
                    match adapter.get_info().backend {
                        // Supported and preferred on Windows & Xbox
                        Backend::Dx12 => 100,
                        // Supported and preferred on macOS
                        Backend::Metal => 100,
                        // Universally supported, acting as a "modern fallback"
                        Backend::Vulkan => 50,
                        // Supported on Windows, acting as a "windows fallback"
                        Backend::Dx11 => 25,
                        // Supported only inside Browsers where no other
                        // option is present
                        Backend::BrowserWebGpu => 100,
                        // Old universal backend, acting as a last-resort
                        Backend::Gl => 0,
                        Backend::Empty => i32::MIN, // never hit, see above
                    }
                }

                let score = score_type(&x) + score_backend(&x) + score_adapter(&x);

                (x, score)
            })
            .collect();
        adapters.sort_by_cached_key(|x| x.1);
        adapters
    }

    fn make_device_and_queue(adapter: &Adapter) -> EngineResult<LogicalDevice> {
        // TODO: Parameterize
        let limits = Limits {
            max_bind_groups: 7,
            ..Default::default()
        };

        let (device, queue) = pollster::block_on(adapter.request_device(
            &DeviceDescriptor {
                label: Some("Main Device"),
                features: Features::empty(),
                limits,
            },
            None,
        ))
        .map_err(|_| EngineError::RequestDeviceError)?;

        log::debug!("Device: {:#?}", device);
        log::debug!("Queue: {:#?}", queue);

        let logical_device = LogicalDevice::new(device, queue);
        Ok(logical_device)
    }
}

impl TComputingEngine for WGPUComputingEngine {
    fn get_instance(&self) -> &Instance {
        &self.instance
    }

    fn get_adapter(&self) -> &Adapter {
        &self.adapter
    }

    fn get_logical_device(&self) -> &LogicalDevice {
        &self.logical_device
    }
}
