# Orbital Engine — Architecture & Reference Map

Starting-point map for this repository, intended to speed up future coding sessions.
Covers architecture, tooling, conventions, and the commands you'll reach for most often.

---

## 1. What This Is

**Orbital** is a cross-platform real-time **3D rendering engine & framework**, written in Rust.
It renders PBR scenes (glTF import, image-based lighting) using **[WGPU](https://wgpu.rs/)** as the graphics abstraction and **WGSL** shaders.

- License: **dual MIT + Apache-2.0** (every crate carries both headers). Attribution: Lukas Weber / SakulFlee (© 2022+).
- Home: [Forgejo](https://forgejo.sakul-flee.de/SakulFlee/Orbital) · GitHub mirror at `github.com/SakulFlee/Orbital`.
- Version: `0.1.0`, **edition 2024**, Cargo workspace resolver v2.

### Evolution (why things look the way they do)
The engine was rewritten several times; knowing this explains naming and structure:

| Era | Language / API | Notes |
|-----|----------------|-------|
| Akimo (Java) | CPU 2D → LWJGL OpenGL/Vulkan | Original project name |
| Akimo (C++) | Vulkan bindings | Complete rewrite, same name |
| Orbital (Rust/Vulkan) | Rust + Vulkan bindings | First Rust iteration |
| **Orbital (Rust/WGPU/WGSL)** | **Current** — moved off GLSL to WGSL for cross-platform shaders | This is what you're working on |

---

## 2. Workspace Layout

`Cargo.toml` members = `["Examples/*", "Crates/*", "Tools/*"]`. Three logical zones:

```
Orbital/
├── Crates/          # Engine library crates (the core product)
│   ├── orbital/            # Facade crate — re-exports everything + build.rs (Blender export)
│   ├── orbital_core/       # Math, cache, logging, macros, mip-leveling, wgpu_util
│   ├── orbital_app/        # App builder + module runtime + system scheduling + touch UI
│   ├── orbital_resources/  # Resource management (mesh/model/camera/light/shader/texture/…)
│   ├── orbital_ecs/        # The ECS core (entity/component/query/system/world) — name is "orbital_ecs"
│   ├── orbital_importer_gltf/  # GLTF 2.0 importer (+ KHR_lights_punctual)
│   ├── orbital_renderer/   # WGPU render pipeline
│   ├── orbital_input/      # Keyboard/mouse/gamepad input abstraction
│   ├── orbital_procgeo/    # Procedural geometry + scene (.ron) format
│   ├── orbital_shader_preprocessor/  # WGSL preprocessing
│   ├── orbital_file_manager/     # File/dir access (+ Android NDK)
│   ├── orbital_ecs_bridge/      # Bridges ECS ↔ engine resources
│   ├── orbital_debug_render/    # Debug overlay (wireframe, spheres, lights…)
│   └── orbital_touch_ui/        # Touch controls UI
├── Examples/        # Runnable demo apps (each = a small Bevy-style app)
├── Tools/           # orbital-cli — Android project scaffolding tool
├── Assets/          # Source models (.blend), exported .glb, shaders, HDR envs, scenes
├── wit/             # WebAssembly interface types (WIT files for WASM build)
├── Images/          # Render screenshots
├── Testing/, CTestMod/  # Non-Rust tests (C + wasm) — not part of the cargo workspace
└── _main/target/    # Leftover build dir (ignored-ish; git-tracked noise)
```

**~31,800 lines of Rust** across `Crates` and `Examples`.

---

## 3. Core Architecture & Conventions

### 3.1 Bevy-inspired App / Module pattern (`orbital_app`)
- `App::new().add_module(MyModule).liftoff(event_loop, settings)` — the entry point for every app.
- Modules are plugins: each contributes **ECS entities, resources, and systems**.
- All module systems are merged into a single game schedule (see `core_schedule.rs`).
- Examples use `make_desktop_main!(entrypoint);` + an `entrypoint(event_loop)` fn.

### 3.2 ECS core (`orbital_ecs`) — the heart of the world
- **Entity / Component**: components stored in typed stores (`component/store.rs`, `world_store.rs`).
- **Query system**: heavily macro-driven (`query/macros/…`) for compile-time-typed, zero-cost queries.
- **Systems**: scheduled work (`system/{runner,schedule,executor,param,commands,merge}.rs`); systems can read/write resources and access the world.
- **World** ties entities + components + resources together; `IntoSystem` trait adapts closures to systems.
- **Messaging**: elements communicate via a message-passing bus (tag-based fan-out), *not* shared memory — keeps coupling loose and scales well.

### 3.3 Resource lifecycle (`orbital_resources`)
Resources follow an explicit lifecycle: **creation → realization → caching → cleanup**. Key subsystems:
- `mesh/`, `model/` — geometry & mesh descriptors/caches
- `camera/` (descriptor, frustum, mode), `light/`, `instance/`
- `material_shader/`, `pbr_material_shader/` — PBR material + shader descriptor caches
- `texture/`, `buffer/`, `vertex/`, `transform/`, `shadow/`, `projection/`, `cull/`
- `ibl_brdf/` (+ bundled WGSL) — image-based lighting / BRDF
- `world_environment/` — HDR sky cube generation (bundled `.wgsl` helpers)

### 3.4 Rendering (`orbital_renderer`) + shaders (`Assets/Shaders`, `orbital_resources/shader/*.wgsl`)
- WGPU render pipeline; WGSL shaders are preprocessed by `orbital_shader_preprocessor`.
- Bundled shaders: `default.wgsl`, `pbr.wgsl`, `instance_cull.wgsl`, `shadow_depth.wgsl`, `wireframe.wgsl`, `test.wgsl`.

### 3.5 Import pipeline (`orbital_importer_gltf`)
Full GLTF 2.0 import: async task-based loading, type-specific importers, texture/image decoding (via `image`), error/result types. Supports KHR_lights_punctual.

---

## 4. Assets Pipeline

```
Assets/
├── ModelFiles/*.blend        # Source models (Blender) — NOT committed as .glb
├── Models/*.glb              # Exported glTF binaries (committed: DamagedHelmet, InstancingTest)
│   └── TestScene.{bin,gltf}  # procgeo scene test data
├── ModelScripts/             # Blender export scripts
│   ├── blender_gltf_export.py   # .blend → .glb exporter (used by build.rs)
│   └── pbr_grid.py              # generates the PBR grid model
├── Shaders/*.wgsl            # Shader sources
├── Scenes/procgeo_demo.ron   # Procedural scene description (RON format)
└── WorldEnvironments/*.hdr   # IBL environment maps (Kloppenheim, LonelyRoad, PhotoStudio)
```

**⚠️ Build-time model export:** `Crates/orbital/build.rs` runs **Blender in headless mode** to:
1. Generate the PBR grid `.glb` (`blender_pbr_grid()`).
2. Export every `Assets/ModelFiles/*.blend` → `Assets/Models/` (`blender_model_files()`).

It is gated by the **`SKIP_GLTF_EXPORT`** env var — set it to skip Blender entirely (CI does this so builds don't depend on a local Blender install). If export runs and fails, `build.rs` panics.

---

## 5. Tooling & Build System

### Nix devShell (`flake.nix`)
Primary environment. Provides:
- **Fenix Nightly** Rust toolchain with components: `cargo`, `clippy`, `rust-src`, `rustc`, `rustfmt`, `rust-analyzer`.
- Cross-compilation targets: `aarch64-unknown-linux-gnu`, `x86_64-pc-windows-gnu` (via `combine`).
- Linux build inputs: glibc dev, systemd, Wayland, libxkbcommon, vulkan-loader, mesa.
- Tools: `gcc`, `cargo-flamegraph`, `cargo-criterion`, `pkg-config`, `blender`.

Enter it with **`nix-shell`** (or `direnv load`, since `.envrc` = `use flake`). The shellHook sets up cross-linkers and `LD_LIBRARY_PATH`.

### Cargo config / linker setup (`.cargo/config.toml`)
Custom linkers for cross targets:
```toml
[target.x86_64-unknown-linux-gnu]  linker = "x86_64-linux-gnu-gcc"
[target.aarch64-unknown-linux-gnu] linker = "aarch64-linux-gnu-gcc"
… (armv7, arm variants)
```

### Profile tuning (`Cargo.toml`)
- **release**: `lto=true`, `opt-level=3`, `codegen-units=1` — fully optimized.
- **dev**: `opt-level=1`, `codegen-units=16`; the heavy deps (`wgpu`, `winit`, `hashbrown`, `cgmath`) are bumped to `opt-level=3` in dev for faster iteration.

### CI / CD
Three systems:
| System | Where | What it does |
|--------|-------|--------------|
| **GitHub Actions** | `.github/workflows/main.yml` | `lint` (clippy + `fmt --check`) → `test` (`cargo test --no-fail-fast`) → on tag, matrix builds for linux/mac/windows, zips artifacts (+ `Assets/`) → GitHub release. Uses `SKIP_GLTF_EXPORT=true`. |
| **JetBrains TeamCity** | `.teamcity/{pom.xml,settings.kts}` | Alternative JetBrains CI config. |
| **Forgejo workflows** | `.forgejo/workflows`, `.forgejo/ci` | Self-hosted git host (forgejo.sakul-flee.de) specific automation. |

Dependency auto-updates: `renovate.json` (`config:recommended`).

### WASM support (`NOTE-for-WASM.md`)
- Uses **rayon** via `wasm-bindgen-rayon` (Web Workers + `SharedArrayBuffer`).
- Web server **must** send `Cross-Origin-Opener-Policy: same-origin` and `Cross-Origin-Embedder-Policy: require-corp`.
- Requires the **`wasm-threads`** feature on `orbital_ecs`, or WASM compilation fails at build time.
- WIT interface types live in `wit/` (`orbital.wit`, `module.wit`, `command.wit`).

---

## 6. Examples (how to run them)

Each example is a tiny app: a `lib.rs` (library, e.g. `test_pbr_grid`) + `main.rs` that calls the engine entrypoint. Binaries are named with a `test_…` prefix.

| Example | Run command | What it shows |
|---------|-------------|---------------|
| pbr_grid | `cargo run -p pbr_grid --bin test_pbr_grid_desktop` | PBR material grid (needs Blender export) |
| gltf_pbr_damaged_helmet | `cargo run -p gltf_pbr_damaged_helmet --bin …` | GLTF PBR helmet import |
| instancing | `…/instancing --bin …` | Instanced rendering |
| procgeo_scene | `…/procgeo_scene --bin …` | Procedural geometry scene (.ron) |
| roll_camera | `…/roll_camera --bin …` | Camera controls |
| skybox | `…/skybox --bin test_skybox_desktop` | Environment/skybox mapping |
| multi_module | `…/multi_module --bin …` | Module system demo (Camera + Model modules, DebugModule, TouchUi) |

> Note: the first build of a `.blend`-dependent example will invoke Blender via `build.rs`. Use `SKIP_GLTF_EXPORT=1 cargo run …` to skip that.

---

## 7. Quickstart Commands

```bash
# Environment
nix-shell                                   # or: direnv load
direnv load                                 # if direnv is set up

# Build / test / lint (use SKIP_GLTF_EXPORT=1 to avoid needing Blender)
SKIP_GLTF_EXPORT=1 cargo build --release
SKIP_GLTF_EXPORT=1 cargo test --no-fail-fast
cargo clippy
cargo fmt --all --check

# Run an example
SKIP_GLTF_EXPORT=1 cargo run -p pbr_grid --bin test_pbr_grid_desktop

# Cross-compile (targets defined in .cargo/config.toml)
cargo build --release --target aarch64-unknown-linux-gnu
```

### Common environment requirements
`pkg-config`, `libudev-dev`, `libwayland-dev`, `libxkbcommon-dev`, `libvulkan-dev`, `mesa-vulkan-drivers`, and **Blender** (only if not using `SKIP_GLTF_EXPORT`). On Linux the Nix shell provides these; on plain CI they're installed via apt.

---

## 8. Gotchas & Notes for Future Sessions

- **`build.rs` is heavy**: it shells out to Blender and can panic on failure — always set `SKIP_GLTF_EXPORT=1` in scripts/CI unless you want the export step.
- **Pre-exported models are committed** (`Assets/Models/DamagedHelmet.glb`, `InstancingTest.glb`) but regenerated `.blend` exports are gitignored (see `.gitignore`).
- **Dual-license headers**: adding a new crate requires both MIT and Apache-2.0 license headers.
- **ECS macros are the trickiest part** (`orbital_ecs/src/query/macros/…`) — read `query.rs` + the macro files when working on queries.
- **Resource caches** live in `orbital_resources` (mesh/material_shader/model/etc.) and are keyed by descriptor; changing a descriptor's fields can invalidate caching assumptions.
- **`_main/target/`** is stray build output — not part of normal builds, git-tracked noise.
- **Testing/** and **CTestMod/** hold C + wasm tests outside the Rust workspace — run manually, not via `cargo test`.
- The working tree is currently clean on branch `main`; game logs (`game-0/1/2.log`) show runtime logging at DEBUG level (logger init line: `orbital_core::logging`).

---

## 9. Key Files Cheat Sheet

| File | Purpose |
|------|---------|
| `README.md` | Public-facing overview, features, history |
| `Cargo.toml` | Workspace + shared deps + profile tuning |
| `flake.nix` / `shell.nix` | Dev environment (Nix/Fenix) |
| `.cargo/config.toml` | Cross-compilation linkers |
| `Orbital.toml` | Engine repo/Android build config for `orbital-cli` |
| `NOTE-for-WASM.md` | WASM threading + server header requirements |
| `Crates/orbital/build.rs` | Blender → glTF model export (gated by SKIP_GLTF_EXPORT) |
| `.github/workflows/main.yml` | CI: lint/test/matrix build/release |
| `Assets/Shaders/*.wgsl` | Shader sources used across the engine |
| `wit/*.wit` | WebAssembly interface types |
