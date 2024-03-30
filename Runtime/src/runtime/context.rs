use crate::surface_wrapper::SurfaceWrapper;
use log::info;
use wgpu::{
    util::{
        backend_bits_from_env, dx12_shader_compiler_from_env, gles_minor_version_from_env,
        initialize_adapter_from_env_or_default,
    },
    Adapter, Device, DeviceDescriptor, Features, Instance, InstanceDescriptor, InstanceFlags,
    Limits, Queue,
};

pub struct Context {
    instance: Instance,
    adapter: Adapter,
    device: Device,
    queue: Queue,
}

impl Context {
    pub async fn init(surface: &mut SurfaceWrapper) -> Self {
        info!("Initializing WGPU ...");

        let backends = backend_bits_from_env().unwrap_or_default();
        let dx12_shader_compiler = dx12_shader_compiler_from_env().unwrap_or_default();
        let gles_minor_version = gles_minor_version_from_env().unwrap_or_default();

        let instance = Instance::new(InstanceDescriptor {
            backends,
            flags: InstanceFlags::from_build_config().with_env(),
            dx12_shader_compiler,
            gles_minor_version,
        });

        let adapter = initialize_adapter_from_env_or_default(&instance, surface.get())
            .await
            .expect("No suitable GPU adapters found!");
        let adapter_info = adapter.get_info();
        info!("Adapter: {} ({:#?})", adapter_info.name, adapter_info);

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    required_features: Features::default(),
                    required_limits: Limits::default(),
                },
                None,
            )
            .await
            .expect("Unable to find suitable GPU device!");

        Self {
            instance,
            adapter,
            device,
            queue,
        }
    }

    pub fn instance(&self) -> &Instance {
        &self.instance
    }

    pub fn adapter(&self) -> &Adapter {
        &self.adapter
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn queue(&self) -> &Queue {
        &self.queue
    }
}
