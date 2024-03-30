use crate::{error::ConnectorError, logging::*};
use wgpu::{
    Adapter, Backends, Device, DeviceDescriptor, Features, Instance, InstanceDescriptor, Limits,
    PowerPreference, Queue, RequestAdapterOptions, Surface, SurfaceConfiguration,
};
use winit::window::Window;

#[derive(Debug)]
pub struct GPUBackend<'a> {
    instance: Instance,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    surface: Option<Surface<'a>>,
    surface_configuration: Option<SurfaceConfiguration>,
}

impl<'a> GPUBackend<'a> {
    pub fn new(with_window: Option<&'a Window>) -> Result<Self, ConnectorError> {
        let instance = Self::make_instance();

        let mut surface: Option<Surface> = None;
        if let Some(window) = with_window {
            surface = Some(Self::make_surface(&instance, window)?);
        }

        let adapter = Self::make_adapter(&instance, surface.as_ref())?;

        let (device, queue) = Self::make_device_and_queue(&adapter)?;

        let mut surface_configuration: Option<SurfaceConfiguration> = None;
        if let Some(surface) = &surface {
            let window = with_window.expect("Window existed before, but is gone now");
            surface_configuration = surface.get_default_config(
                &adapter,
                window.inner_size().width,
                window.inner_size().height,
            );

            if surface_configuration.is_none() {
                warn!("Surface & Window exist, but configuration failed");
            }
        }

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            surface,
            surface_configuration,
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

    fn make_surface(
        instance: &Instance,
        from_window: &'a Window,
    ) -> Result<Surface<'a>, ConnectorError> {
        let surface = instance
            .create_surface(from_window)
            .map_err(|e| ConnectorError::SurfaceError(e));
        debug!("Surface: {:#?}", surface);

        surface
    }

    fn make_adapter<'surface>(
        instance: &Instance,
        compatible_surface: Option<&Surface<'surface>>,
    ) -> Result<Adapter, ConnectorError> {
        let adapter = pollster::block_on(async {
            instance
                .request_adapter(&RequestAdapterOptions {
                    compatible_surface: compatible_surface,
                    force_fallback_adapter: false,
                    power_preference: PowerPreference::HighPerformance,
                })
                .await
                .ok_or(ConnectorError::NoAdapters)
        });
        debug!("Adapter: {:#?}", adapter);

        adapter
    }

    fn make_device_and_queue(adapter: &Adapter) -> Result<(Device, Queue), ConnectorError> {
        // TODO: Parameterize
        let limits = Limits {
            max_bind_groups: 7,
            ..Default::default()
        };

        let (device, queue) = pollster::block_on(async {
            adapter
                .request_device(
                    &DeviceDescriptor {
                        label: Some("Main Device"),
                        required_features: Features::empty(),
                        required_limits: limits,
                    },
                    None,
                )
                .await
                .map_err(|_| ConnectorError::RequestDeviceError)
        })?;
        debug!("Device: {:#?}", device);
        debug!("Queue: {:#?}", queue);

        Ok((device, queue))
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

    pub fn surface(&self) -> &Option<Surface<'a>> {
        &self.surface
    }

    pub fn surface_configuration(&self) -> &Option<SurfaceConfiguration> {
        &self.surface_configuration
    }
}
