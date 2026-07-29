# Known Issue: Windows Hang on AMD Hybrid-Graphics Systems

## Description

On Windows machines with **AMD GPUs** (or AMD + NVIDIA dual-GPU laptops), the engine hangs
during WGPU adapter initialization. The window opens but freezes immediately; no rendering occurs.

**Root cause:** The implicit Vulkan layer `VK_LAYER_AMD_switchable_graphics` (shipped with AMD
GPU drivers) has a known bug in its `vkEnumeratePhysicalDevices` implementation. When the Vulkan
loader calls into this layer, it either hangs indefinitely or returns `VK_INCOMPLETE` in a loop.
This affects both single-AMD and AMD+NVIDIA hybrid configurations.

The log typically shows a warning like:

```
Layer VK_LAYER_AMD_switchable_graphics uses API version 1.2 which is older than the
application specified API version of 1.3. May cause issues.
```

followed by a hang after `Surface: ...` is printed.

**References:**
- https://github.com/KhronosGroup/Vulkan-Loader/issues/1165
- https://github.com/GPUOpen-Drivers/AMDVLK/issues/195
- https://github.com/GPUOpen-Drivers/AMDVLK/issues/196
- NVIDIA KB article: https://nvidia.custhelp.com/app/answers/detail/a_id/5182

---

## Workarounds

### Option 1: Force DX12 via environment variable (recommended)

Windows' native DX12 backend is unaffected. Select it at runtime:

```sh
set WGPU_BACKEND=dx12
cargo run
```

The engine uses `Backends::from_env()` which reads `WGPU_BACKEND` before falling back to
all available backends.

### Option 2: Disable the AMD Vulkan layer (keeps Vulkan available)

The AMD layer provides an official disable mechanism via environment variable:

```sh
set DISABLE_LAYER_AMD_SWITCHABLE_GRAPHICS_1=1
cargo run
```

This keeps Vulkan available for non-AMD GPUs (e.g. NVIDIA) while working around the
AMD-specific bug.

**Note:** The engine automatically sets `DISABLE_LAYER_AMD_SWITCHABLE_GRAPHICS_1=1` on
Windows startup (`context.rs:make_instance`). You only need to set it manually if the
automatic approach does not work in your environment.

### Option 3: Update AMD GPU drivers

The bug exists in older versions of the `VK_LAYER_AMD_switchable_graphics` layer (API v1.2).
Updating to the latest AMD Adrenalin drivers may resolve the issue. However, some systems
with OEM-customised drivers cannot update independently.

---

## Status

- This is a **driver-layer bug**, not an engine bug.
- The engine sets `DISABLE_LAYER_AMD_SWITCHABLE_GRAPHICS_1=1` automatically on Windows
  at startup. No manual env var is needed in most cases.
- If the hang still occurs, manually set `WGPU_BACKEND=dx12` as a fallback.
- The engine will log all available adapters and the selected adapter on startup,
  making it easy to verify which backend is in use.
- The automatic workaround can be removed once the AMD layer is fixed in widespread
  drivers.