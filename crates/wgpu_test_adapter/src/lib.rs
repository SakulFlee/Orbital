use smol::block_on;
use wgpu::{
    Adapter, Backends, Device, DeviceDescriptor, Instance, InstanceDescriptor, PowerPreference,
    Queue, RequestAdapterOptions,
};

pub async fn make_wgpu_connection_async() -> (Adapter, Device, Queue) {
    let instance = Instance::new(InstanceDescriptor {
        backends: Backends::all(),
        ..Default::default()
    });
    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::None,
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await
        .expect("Failed to find any adapter");

    let (device, queue) = adapter
        .request_device(
            &DeviceDescriptor {
                ..Default::default()
            },
            None,
        )
        .await
        .expect("Failed to create device");

    println!("{:#^80}", " WGPU Test Adapter ");
    println!("# {: ^76} #", "!!! for testing only !!!");
    println!("# {: <76} #", format!("Name: {}", adapter.get_info().name));
    println!(
        "# {: <76} #",
        format!("Backend: {:?}", adapter.get_info().backend)
    );
    println!(
        "# {: <76} #",
        format!("Device Type: {:?}", adapter.get_info().device_type)
    );
    println!(
        "# {: <76} #",
        format!("Driver: {}", adapter.get_info().driver)
    );
    println!(
        "# {: <76} #",
        format!("Driver Info: {}", adapter.get_info().driver_info)
    );
    println!("{:#^80}", "");

    (adapter, device, queue)
}

pub fn make_wgpu_connection() -> (Adapter, Device, Queue) {
    block_on(async { make_wgpu_connection_async().await })
}

#[test]
fn test_make_connection() {
    let (_adapter, _device, _queue) = make_wgpu_connection();
}
