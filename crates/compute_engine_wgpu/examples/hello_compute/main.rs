// Full credit of this goes to wgpu (https://github.com/gfx-rs/wgpu).
// This example was taken from their examples and mildly modified to work
// with our engine version.
// The original example is licensed under Apache-2.0 & MIT and
// can be found here:
// https://github.com/gfx-rs/wgpu/tree/trunk/examples/src/hello_compute

use std::borrow::Cow;

use compute_engine_wgpu::{ComputeEngineTrait, ComputeEngineWGPU};
use wgpu::util::DeviceExt;

#[tokio::main]
pub async fn main() {
    println!("Full credit of this goes to wgpu (https://github.com/gfx-rs/wgpu).");
    println!("This example was taken from their examples and mildly modified to work with our engine version.");
    println!("The original example is licensed under Apache-2.0 & MIT and can be found here:");
    println!("https://github.com/gfx-rs/wgpu/tree/trunk/examples/src/hello_compute");
    println!();

    let number_v: Vec<u32> = vec![1, 2, 3, 4];
    let numbers: &[u32] = &number_v;

    println!("Input: ");
    for (i, e) in number_v.iter().enumerate() {
        println!("#{}: {}", i, e);
    }
    println!();

    let compute_engine = ComputeEngineWGPU::new()
        .await
        .expect("Compute Engine startup failure!");

    // Loads the shader from WGSL
    let cs_module = compute_engine
        .device()
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

    // Gets the size in bytes of the buffer.
    let size = std::mem::size_of_val(&numbers) as wgpu::BufferAddress;

    // Instantiates buffer without data.
    // `usage` of buffer specifies how it can be used:
    //   `BufferUsages::MAP_READ` allows it to be read (outside the shader).
    //   `BufferUsages::COPY_DST` allows it to be the destination of the copy.
    let staging_buffer = compute_engine
        .device()
        .create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

    // Instantiates buffer with data (`numbers`).
    // Usage allowing the buffer to be:
    //   A storage buffer (can be bound within a bind group and thus available to a shader).
    //   The destination of a copy.
    //   The source of a copy.
    let storage_buffer =
        compute_engine
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Storage Buffer"),
                contents: bytemuck::cast_slice(&numbers),
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
            });

    // A bind group defines how buffers are accessed by shaders.
    // It is to WebGPU what a descriptor set is to Vulkan.
    // `binding` here refers to the `binding` of a buffer in the shader (`layout(set = 0, binding = 0) buffer`).

    // A pipeline specifies the operation of a shader

    // Instantiates the pipeline.
    let compute_pipeline =
        compute_engine
            .device()
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: None,
                layout: None,
                module: &cs_module,
                entry_point: "main",
            });

    // Instantiates the bind group, once again specifying the binding of buffers.
    let bind_group_layout = compute_pipeline.get_bind_group_layout(0);
    let bind_group = compute_engine
        .device()
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: storage_buffer.as_entire_binding(),
            }],
        });

    // A command encoder executes one or many pipelines.
    // It is to WebGPU what a command buffer is to Vulkan.
    let mut encoder = compute_engine
        .device()
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        cpass.set_pipeline(&compute_pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        cpass.insert_debug_marker("compute collatz iterations");
        cpass.dispatch_workgroups(numbers.len() as u32, 1, 1); // Number of cells to run, the (x,y,z) size of item being processed
    }
    // Sets adds copy operation to command encoder.
    // Will copy data from storage buffer on GPU to staging buffer on CPU.
    encoder.copy_buffer_to_buffer(&storage_buffer, 0, &staging_buffer, 0, size);

    // Submits command encoder for processing
    compute_engine.queue().submit(Some(encoder.finish()));

    // Note that we're not calling `.await` here.
    let buffer_slice = staging_buffer.slice(..);
    // Sets the buffer up for mapping, sending over the result of the mapping back to us when it is finished.
    let (sender, receiver) = flume::bounded(1);
    buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

    // Poll the device in a blocking manner so that our future resolves.
    // In an actual application, `compute_engine.device().poll(...)` should
    // be called in an event loop or on another thread.
    compute_engine
        .device()
        .poll(wgpu::Maintain::wait())
        .panic_on_timeout();

    // Awaits until `buffer_future` can be read from
    if let Ok(Ok(())) = receiver.recv_async().await {
        // Gets contents of buffer
        let data = buffer_slice.get_mapped_range();
        // Since contents are got in bytes, this converts these bytes back to u32
        let result: Vec<u32> = bytemuck::cast_slice(&data).to_vec();

        // With the current interface, we have to make sure all mapped views are
        // dropped before we unmap the buffer.
        drop(data);
        staging_buffer.unmap(); // Unmaps buffer from memory
                                // If you are familiar with C++ these 2 lines can be thought of similarly to:
                                //   delete myPointer;
                                //   myPointer = NULL;
                                // It effectively frees the memory

        // Returns data from buffer
        println!("Output: ");
        for (i, e) in result.iter().enumerate() {
            println!("#{}: {}", i, e);
        }
    } else {
        panic!("failed to run compute on gpu!")
    }
}
