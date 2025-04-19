use log::debug;
use smol::block_on;
use wgpu::{
    Adapter, Backends, Device, DeviceDescriptor, Instance, InstanceDescriptor, PowerPreference,
    Queue, RequestAdapterOptions,
};

pub async fn make_wgpu_connection_async() -> (Adapter, Device, Queue) {
    logging::init();

    debug!("{:#^88}", " WGPU Test Adapter ");
    debug!("# {: ^84} #", "!!! for testing only !!!");

    let instance = Instance::new(InstanceDescriptor {
        backends: Backends::all(),
        ..Default::default()
    });
    debug!("# {: <84} #", format!("Instance: {:?}", instance));

    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::None,
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await
        .expect("Failed to find any adapter");
    debug!("# {: <84} #", format!("Name: {}", adapter.get_info().name));
    debug!(
        "# {: <84} #",
        format!("Backend: {:?}", adapter.get_info().backend)
    );
    debug!(
        "# {: <84} #",
        format!("Device Type: {:?}", adapter.get_info().device_type)
    );
    debug!(
        "# {: <84} #",
        format!("Driver: {}", adapter.get_info().driver)
    );
    debug!(
        "# {: <84} #",
        format!("Driver Info: {}", adapter.get_info().driver_info)
    );

    let (device, queue) = adapter
        .request_device(
            &DeviceDescriptor {
                ..Default::default()
            },
            None,
        )
        .await
        .expect("Failed to create device");
    debug!("# {: <84} #", format!("Device: {:?}", device.features()));
    debug!("# {: <84} #", format!("Queue: {:?}", queue));

    debug!("{:#^88}", "");

    (adapter, device, queue)
}

pub fn make_wgpu_connection() -> (Adapter, Device, Queue) {
    block_on(async { make_wgpu_connection_async().await })
}

#[test]
fn test_make_connection() {
    let (_adapter, _device, _queue) = make_wgpu_connection();
}
