# WGPU Test Adapter

This is a test adapter crate used solely for running Rust/Cargo tests.  
Only ever use this for testing purposes and never for anything else!

## Continues Integration & Software Rendering

> [!warning]
> This section is Linux-only, as most CIs run on Linux in some shape or form.
> Windows and macOS based CIs should have access to a GPU, be it virtual, in software, or native.

Most testing environments don't have access to a dedicated or integrated GPU.
To get around this limitation, we can make use of **Software Rendering**.
However, _Software Rendering_ is commonly painfully slow as it usually are performing _CPU-Emulated Rendering_.

> [!info]
> If you have access to the machine of your testing environment, you might be able to passthrough a integrated or dedicated GPU into the test environment, or, create a virtual GPU to be used.

To make sure a _Software Rendering Adapter_ (i.e. a _Fallback Device_) can be found by WGPU, make sure you have **MESA** and **LibGL** installed!  
On Debian-like systems you can do so via:

```bash
apt-get update && apt-get install -y mesa-utils libgl1-mesa-dev
```

> [!notice]
> An dedicated or integrated, or even virtual, GPU will always be chosen over a Software Rendering adapter!  
> The order of choice _should_ be: Dedicated -> Integrated -> Virtual -> Software -> Dummy -> None

### Docker

You can even use this inside Docker!

```bash
docker run --rm -it -v "${PWD}:/app" rust:latest bash -c 'cd /app && apt-get update && apt-get install -y mesa-utils libgl1-mesa-dev && cargo test --package wgpu_test_adapter'
```

## Adapter info

When using `cargo test`, you can parse `-- --nocapture` to enable logging of adapter details whenever the adapter is called:

```bash
cargo test (--package <package>) -- --nocapture
```

> [!note]  
> The `--` is important as we are passing the argument `--nocapture` to the **test binary**, not `cargo test`!

Example Output:

```log
(...)
running 1 test
Adapter: Adapter { context: ContextWgpuCore { type: "Native" }, data: Any { .. } }
Device: Device { context: ContextWgpuCore { type: "Native" }, data: Any { .. } }
Queue: Queue { context: ContextWgpuCore { type: "Native" }, data: Any { .. } }
---
Name: "llvmpipe (LLVM 15.0.6, 256 bits)"
Backend: Gl
Device Type: Cpu
Driver: ""
Driver Info: "4.5 (Core Profile) Mesa 22.3.6"
test test_fn ... ok
(...)
```
