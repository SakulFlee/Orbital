# AGENTS.md

Operating guide for everyone working in this repository: human contributors and AI coding agents alike.
It is also the project's contributor contract.

If you are an AI agent, treat this file as your project instructions.
If you are a human contributor, the same rules apply to you.

---

## 1. Ownership and AI policy

AI tools are permitted and welcome.
Submitting code you have not inspected and understood is not.

- **You own what you submit.** The person who opens the pull request is the author and owner of the change, regardless of how it was produced.
- **Read and understand every line.** Do not paste generated code, patches, tests, or documentation that you cannot explain and maintain. If you cannot defend a line in review, remove it.
- **Verify that it works.** Run the build, formatter, linter, and test suite. Run or exercise the affected example when applicable. CI is a safety net, not a substitute for local verification.
- **No model attribution.** Do not mention AI models, providers, or versions in code, comments, documentation, commit messages, branch names, pull-request titles/descriptions, or changelogs. Do not add `Co-authored-by`, `Signed-off-by`, or “Generated with …” trailers for AI. AI tools must not be listed as authors, co-authors, owners, or contributors.
- **One responsible human.** The pull-request author is solely accountable for correctness, security, licensing, compatibility, and follow-up fixes. Asking someone else to review does not transfer that responsibility.
- **Keep the change deliberate.** Do not mix unrelated refactors, formatting churn, dependency updates, or generated files into a focused change. Do not commit secrets, credentials, or machine-specific paths.
- **Do not invent APIs.** Verify that functions, crates, and configuration actually exist in this tree and in the resolved dependency versions. Do not add dependencies without need and license review.
- **Respect licensing.** Only submit code you have the right to contribute. Check `LICENSE` and the license of any third-party code you adapt.
- **Agents must be honest.** Never report a command as passing unless it was run and passed. If something is blocked, say so and include the error.

In short: using AI as a tool is fine.
Publishing code you did not inspect, test, and take responsibility for is not.

---

## 2. Before you commit or open a pull request

From the repository root:

```bash
export SKIP_GLTF_EXPORT=1
cargo fmt --all -- --check
cargo clippy -- -D warnings
cargo test --no-fail-fast
```

- The forked iced crates are fetched automatically by Cargo from the `orbital-iced` git dependency; no submodule initialization is required.
- `SKIP_GLTF_EXPORT` is checked by **presence**, not value. It skips Blender-based asset export. Examples that need generated models (`pbr_grid`, `multi_module`) require the export to run once, or the generated `.glb` files to already exist.
- Forgejo CI runs `cargo fmt --all -- --check` and `cargo clippy -- -D warnings`. GitHub and TeamCity are looser. Use the strict Forgejo commands locally.
- Run the affected example(s) from the table in section 6.
- For changes to `Crates/orbital_cli` or its templates, test the CLI end to end: build it, scaffold a throwaway project, and exercise the affected `orbital build`/`orbital run` path.
- Review `git diff` yourself: no debug leftovers, secrets, generated assets, or unrelated files.
- Commit messages, pull-request metadata, and code must not contain AI/model attribution.

### Pull-request ownership checklist

- [ ] I have read and understood every line in this change.
- [ ] I can explain and maintain the change without relying on the tool that produced it.
- [ ] I ran formatting, clippy, and tests, and they pass.
- [ ] I ran or otherwise verified the affected example/app.
- [ ] The diff is focused and contains no unrelated or generated files.
- [ ] There is no AI/model attribution in commits, code, docs, or PR metadata.
- [ ] I accept ownership and responsibility for this change.

---

## 3. Repository overview

**Orbital** is a cross-platform real-time 3D rendering engine and framework written in Rust.
It renders PBR scenes (glTF import, image-based lighting, shadows) with **WGPU** and **WGSL** shaders.

- Canonical repository: [Forgejo](https://forgejo.sakul-flee.de/SakulFlee/Orbital).
- GitHub is a mirror. Open pull requests on Forgejo; if you can only use GitHub, treat a GitHub PR as a proposal to be ported to Forgejo before merge.
- New projects and platform builds go through the `orbital` CLI (`Crates/orbital_cli`); it is a primary user-facing part of the project.
- Edition: 2024. Workspace resolver: v2.
- License: see `LICENSE`. Do not rely on older documentation for licensing terms.

### History

The engine was rewritten several times; this explains some naming:

| Era | Language / API | Notes |
|-----|----------------|-------|
| Akimo (Java) | CPU 2D → LWJGL OpenGL/Vulkan | Original project name |
| Akimo (C++) | Vulkan bindings | Complete rewrite, same name |
| Orbital (Rust/Vulkan) | Rust + Vulkan bindings | First Rust iteration |
| **Orbital (Rust/WGPU/WGSL)** | **Current** | Cross-platform shaders via WGPU/WGSL |

---

## 4. Workspace layout

Workspace members are `Examples/*` and `Crates/*` (see `Cargo.toml`).

| Path | Purpose |
|------|---------|
| `Crates/orbital` | Facade crate. Re-exports engine modules (`twod`, `app`, `debug_render`, `ecs`, `ecs_bridge`, `file_manager`, `iced`, `importer`, `procgeo`, `renderer`, `resources`, `shader_preprocessor`, `text`, `touch_ui`, `ui`) plus `macros::main`, `cgmath`, `wgpu`, and `winit`. Owns the Blender `build.rs`. |
| `Crates/orbital_app` | `App` builder, `Module` trait, `ModuleRuntime`, core/game schedules, realization systems, render layers and overlays. |
| `Crates/ecs` (`orbital_ecs`) | ECS: world, entities, components, queries, systems, schedules, events. |
| `Crates/orbital_ecs_bridge` | Engine-specific ECS components/resources: camera, model, light, time, input snapshots, GPU resources, importer, iced events. |
| `Crates/orbital_core` | Caches, logging, MIP helpers, quaternion/projection utilities, `wgpu_util`. |
| `Crates/orbital_macros` | `#[orbital::main]` procedural macro generating desktop, Android, and iOS entry points. |
| `Crates/orbital_input` | Keyboard, mouse, touch, and gamepad input abstraction. |
| `Crates/orbital_file_manager` | Cross-platform asset/storage/cache I/O (desktop `Assets/`, Android APK assets, iOS bundle resources). |
| `Crates/orbital_math` | Transforms, modes, projections. |
| `Crates/orbital_camera`, `orbital_instance`, `orbital_vertex`, `orbital_mesh`, `orbital_model`, `orbital_light`, `orbital_cull`, `orbital_shadow`, `orbital_texture`, `orbital_material_shader` | Geometry, scene, and GPU resource crates. |
| `Crates/orbital_ibl_brdf`, `orbital_world_environment` | BRDF LUT and HDR/IBL environment handling. |
| `Crates/orbital_procgeo` | Procedural shapes and RON scene descriptions (`SceneBuilder`). |
| `Crates/orbital_resources` | Compatibility re-export shim only; real types live in the fine-grained crates. |
| `Crates/orbital_shader_preprocessor` | Named WGSL `ShaderNode` registry and `ShaderBuilder`. |
| `Crates/orbital_shader_core` | `ShaderDescriptor`, variables, bind-group and pipeline plumbing. |
| `Crates/orbital_shader_engine`, `orbital_shader_math`, `orbital_shader_pbr`, `orbital_shader_debug`, `orbital_shader_2d` | Reusable WGSL node libraries for engine, math, PBR, debug, and 2D/UI/text. |
| `Crates/orbital_renderer` | Main `Renderer` (shadow pass, skybox, models, timestamps) plus `Renderer2D`, `TextRenderer`, `UiRenderer`. |
| `Crates/orbital_2d` | 2D camera, vertices, shape descriptors, batching. |
| `Crates/orbital_text` | Fontdue-based font data, SDF atlas packing, 2D/billboard text mesh generation. |
| `Crates/orbital_ui` | ECS UI components/widgets, layout, hit testing, UI events. |
| `Crates/orbital_iced` | Integration crate for iced panels (`IcedBridgeModule`, `IcedLayerRenderer`, `IcedState`). |
| `Crates/orbital_touch_ui`, `orbital_debug_render` | Touch controls and debug overlays. |
| `Crates/orbital_importer_gltf` | glTF 2.0 importer with `KHR_lights_punctual`, async rayon tasks. |
| `Crates/orbital_cli` | `orbital-cli` package with the `orbital` binary. Project scaffolding (`init`, `init-android`), templates, desktop/Android/iOS build and run, and `Orbital.toml` handling. |
| `orbital-iced` (external) | Forked iced sub-crates (`iced`, `iced_core`, `iced_widget`, `iced_wgpu`, `iced_winit`, …), consumed as Cargo `git` dependencies from `SakulFlee/orbital-iced`. Not the same as `orbital_iced`. |
| `Examples/` | Official templates as runnable workspace example crates. |
| `Assets/` | Blender sources, exported glTF, HDR environments, RON scenes, loose shaders. |
| `CTestMod/` | C and committed WASM test, outside the Cargo workspace. |
| `wit/` | WebAssembly interface types. |
| `Images/` | README screenshots. |

---

## 5. Core architecture and conventions

### App / module pattern

- Entry point: `App::new().add_module(MyModule).liftoff(event_loop, AppSettings)`.
- A `Module` is a plugin: it receives `&mut World`, `&Device`, `&Queue` in `setup` and returns systems. It can also register render overlays and save/restore/clear state.
- `App` combines modules in registration order; `ModuleRuntime` owns the ECS `World`, schedules, renderer, and overlay renderers.
- Examples use `#[orbital::main]` on an `entrypoint(Result<EventLoop, EventLoopError>)`; the macro generates desktop, Android, and iOS entry points.

### Frame order

Per update: tick timers → process iced events → process deferred touch input → poll gamepads → snapshot input → run core schedule → poll importer → run game schedule with `TotalTime` → process engine events.

Per redraw: realize cameras → lights → environment → models → shadow stagger and CPU frustum cull → build bind groups and extract scene data → shadow pass → skybox → models → overlays sorted by `RenderLayer` → present.

### ECS

- Generational `Entity` handles; per-component-type sparse-set stores; type-erased resources behind `RwLock`.
- Components are blanket-implemented for `Any + Debug + Send + Sync`.
- Queries: `Query<(Read<T>, Write<U>), (With<X>, Without<Y>)>` with macro-generated tuple impls.
- Closure systems support component refs, `Res<T>`, `ResMut<T>`, and `Commands`; complex systems can implement `System` directly.
- `Schedule` preserves registration order and batches non-conflicting systems. The default executor currently runs batches sequentially.
- Interval systems: `System::desired_interval`, `Schedule::add_system_with_interval`, `add_low_priority_system` (1 s), and `with_interval`. The first tick always runs, missed ticks are dropped, and plain `Schedule::run` ignores intervals.
- Commands are deferred and flushed after schedule execution.
- Messaging uses ECS `Events<T>`, `EngineEvents`, `IcedEventQueue`, and importer channels.

### Resource realization

Camera, model, light, and environment resources follow descriptor → ECS component/dirty flag → realization system → GPU resource/cache.
Mesh, material, and descriptor caches are keyed by descriptor and use `orbital_core::cache::Cache` (5 s default TTL, `Arc` strong-count tracking).

### Shaders and rendering

- Material shaders are assembled from named WGSL nodes declared by `ShaderDescriptor::nodes`, optional raw source, and variables.
- The preprocessor resolves dependencies and builds the `MaterialShader` pipeline.
- The global registry receives `math`, `engine`, `pbr`, and `world_environment` libraries before module setup.
- Main pass order: shadow → skybox → models. Models use CPU-culled compacted instance buffers when available.
- 2D, text, and UI have separate renderer paths. Most WGSL is embedded in crates with `include_str!`; only `Assets/Shaders/shadow_depth.wgsl` is a loose shader asset.

### Import pipeline

`ImportQueueResource` holds glTF tasks. `ImporterResource` owns a rayon pool and result channel. `sys_poll_importer` drains results and spawns ECS entities for models, cameras, and lights. The importer supports external/local URIs, inline data URIs, and `KHR_lights_punctual`.

### UI and 2D

- `orbital_2d` provides shapes, batching, and `Camera2D`.
- `orbital_text` provides SDF glyph atlases and text meshes.
- `orbital_renderer` exposes `Renderer2D`, `TextRenderer`, and `UiRenderer`.
- `orbital_ui` provides ECS widgets, layout, hit testing, and events; it is not automatically driven by `ModuleRuntime`.
- `orbital_iced` is the production iced panel integration: events are processed before game systems and panels draw at `RenderLayer::UI`.
- `orbital_touch_ui` renders a virtual joystick overlay.

---

## 6. Assets, build scripts, and examples

### Assets

| Path | Contents |
|------|----------|
| `Assets/ModelFiles/*.blend` | Blender sources. |
| `Assets/Models/` | Exported glTF. Only `DamagedHelmet.glb` and `InstancingTest.glb` are committed; other exports are generated and gitignored. |
| `Assets/ModelScripts/` | `blender_gltf_export.py` and `pbr_grid.py`, used by `build.rs`. |
| `Assets/WorldEnvironments/*.hdr` | Committed HDR environment maps. |
| `Assets/Scenes/procgeo_demo.ron` | Procedural scene description. |
| `Assets/Shaders/shadow_depth.wgsl` | Loose shader asset. |
| `Examples/all-in-one/Assets/` | Minimal asset set used when running the all-in-one example from its directory. |

### `build.rs` and Blender

`Crates/orbital/build.rs` runs Blender headlessly when `SKIP_GLTF_EXPORT` is **not set**:

1. Generates `Assets/Models/PBR_Grid.glb` from `Assets/ModelScripts/pbr_grid.py`.
2. Exports every `Assets/ModelFiles/*.blend` to `Assets/Models/` via `blender_gltf_export.py`.

Failures panic. CI sets `SKIP_GLTF_EXPORT=true` so builds do not depend on Blender.
Set the variable locally to skip export; unset it when you need to regenerate models.

### Examples

Run from the example directory or from the repository root:

```bash
cd Examples/minimal && SKIP_GLTF_EXPORT=1 cargo run
cd Examples/all-in-one && SKIP_GLTF_EXPORT=1 cargo run
cd Examples/2d && SKIP_GLTF_EXPORT=1 cargo run

SKIP_GLTF_EXPORT=1 cargo run -p template_minimal
SKIP_GLTF_EXPORT=1 cargo run -p template_all_in_one
SKIP_GLTF_EXPORT=1 cargo run -p template_2d
```

| Directory | Package | Notes |
|-----------|---------|-------|
| `Examples/minimal` | `template_minimal` | Procedural scene; no asset files. |
| `Examples/all-in-one` | `template_all_in_one` | Scene, glTF, lighting, shadows, UI, and overlays. Ships its own minimal `Assets/`. |
| `Examples/2d` | `template_2d` | 2D shape rendering; no asset files. |

The template app sources are canonical in `Examples/`; the CLI embeds them
during generation. `Crates/orbital_cli/src/template/` only contains platform
scaffolding such as Gradle and Xcode files.

---

## 7. Tooling and CI

### Orbital CLI (`orbital`)

`Crates/orbital_cli` builds the `orbital` binary and is the main entry point for creating and building Orbital projects.

```bash
cargo build -p orbital-cli
cargo run -p orbital-cli -- --help

# Scaffold a project
orbital init my-game --template all-in-one

# Add Android support to an existing project
orbital init-android

# Build and run (desktop is the default)
orbital build
orbital build android --release
orbital run ios --device "<device>"
```

- `init` supports the `minimal`, `all-in-one`, and `2d` templates, and can point at the engine via `--engine-repo`, `--engine-branch`, or `--engine-path`.
- The CLI writes `Orbital.toml`, which configures the engine repository/branch plus Android and iOS packaging/deployment settings.
- Template app sources live in `Examples/`; the generators live in `Crates/orbital_cli/src/init/`. Platform scaffolding lives in `Crates/orbital_cli/src/template/`.
- The Nix shell installs a project-local `orbital` into `.cargo/bin` and provides `orbital-update` to refresh it.
- When changing CLI behavior or templates, test the generated project end to end, not only the CLI crate.

### Nix

`flake.nix` provides the primary dev environment: the Rust toolchain pinned by `rust-toolchain.toml` (1.99.1 with `cargo`, `clippy`, `rust-src`, `rustc`, `rustfmt`), `rust-analyzer`, Linux graphics/input libraries, cross-compilation shells, `cargo-flamegraph`, `cargo-criterion`, `pkg-config`, and Blender.
Enter with `nix-shell`, `nix develop`, or `direnv allow` (`.envrc` uses `use flake`).
When changing the channel in `rust-toolchain.toml`, update the manifest `sha256` passed to `fromToolchainFile` in `flake.nix`.

### Cargo configuration

- `.cargo/config.toml` defines linkers for x86_64/aarch64/arm Linux cross targets.
- Release: LTO, `opt-level=3`, one codegen unit, no debug/incremental.
- Dev: `opt-level=1`, thin LTO, 16 codegen units; `wgpu`, `winit`, `hashbrown`, and `cgmath` are bumped to `opt-level=3`.
- `[workspace.lints.*]` exists, but members currently do not opt in with `[lints] workspace = true`. Do not assume those lints are active.

### CI

| System | Files | Lint/test behavior |
|--------|-------|--------------------|
| Forgejo | `.forgejo/workflows/` | `cargo fmt --all -- --check`, `cargo clippy -- -D warnings` on PRs and pushes to `main`; PRs may auto-fix and push a `[ci]` commit. Tests run for all PRs: directly changed crates only, with the full suite for pushes to `main`, PRs targeting `release/**`, manual runs, and release tags. Forgejo write-isolates `pull_request` caches, so the full `main` run seeds the shared cache that PRs restore. |
| GitHub | `.github/workflows/main.yml` | `cargo fmt --all -- --check`, `cargo clippy -- -D warnings`, `cargo test --no-fail-fast`. Tests are skipped for normal PRs. |
| TeamCity | `.teamcity/settings.kts` | `cargo fmt --all -- --check`, `cargo clippy -- -D warnings`, `cargo test --no-fail-fast`. |

All CI systems and the Nix devshell use the toolchain pinned in `rust-toolchain.toml`.

Dependencies are auto-updated by Renovate (`renovate.json`).

### WASM

`NOTE-for-WASM.md` documents COOP/COEP headers and the `wasm-threads` feature.
Treat the current WASM status as unverified: `Crates/ecs/src/system/runner.rs` unconditionally rejects `wasm32`, and no WASM target is built in CI.

---

## 8. Gotchas

- **Two similarly named projects.** `Crates/orbital_iced` is the in-repo Orbital integration crate; `orbital-iced` is the external forked iced repo consumed as a Cargo `git` dependency.
- **`orbital_resources` is a shim.** It only re-exports types from the fine-grained resource crates.
- **There is no `orbital_shaders` crate.** Use the `orbital_shader_*` family.
- **`orbital_ecs` lives in `Crates/ecs`**, not `Crates/orbital_ecs`.
- **`SKIP_GLTF_EXPORT` checks presence.** `0`, `false`, and an empty value still skip export.
- **Generated assets are ignored.** Do not commit `PBR_Grid.glb`, `Cubes.glb`, or `TestScene.*`.
- **`App` state hooks.** `CombinedModule` forwards `setup` and `register_overlays`, but not per-module `save_state`/`restore_state`/`clear_state`.
- **CI coverage is uneven.** Linux is the primary target; Windows/macOS/Android/iOS paths exist but are not equally covered.

---

## 9. Key files

| File | Purpose |
|------|---------|
| `Cargo.toml` | Workspace members, dependencies, profiles, lints. |
| `rust-toolchain.toml` | Pinned Rust toolchain shared by local and CI. |
| `flake.nix` / `shell.nix` | Nix devshells and toolchain. |
| `.cargo/config.toml` | Cross-compilation linkers. |
| `Crates/orbital/src/lib.rs` | Facade re-exports. |
| `Crates/orbital/build.rs` | Blender → glTF export (`SKIP_GLTF_EXPORT`). |
| `Crates/orbital_cli/src/main.rs` | CLI commands and desktop/Android/iOS dispatch. |
| `Crates/orbital_cli/src/init/mod.rs` | Project scaffolding flow. |
| `Crates/orbital_cli/src/init/template.rs` | Generated project files (Cargo, devshell, env). |
| `Orbital.toml` | Engine, Android, and iOS project configuration for the CLI. |
| `Crates/orbital_app/src/module.rs` | `Module` lifecycle. |
| `Crates/orbital_app/src/module_runtime.rs` | Frame loop, schedules, realization, overlays. |
| `Crates/orbital_app/src/systems/realize.rs` | GPU realization for cameras, models, lights, environments. |
| `Crates/orbital_app/src/systems/import.rs` | Import queue polling and entity spawning. |
| `Crates/ecs/src/world.rs` | ECS world, entities, components, resources. |
| `Crates/ecs/src/system/schedule.rs` | Scheduling, conflict batching, intervals. |
| `Crates/ecs/src/query/query.rs` | Query API. |
| `Crates/orbital_ecs_bridge/src/components.rs` | Engine ECS components. |
| `Crates/orbital_ecs_bridge/src/resources.rs` | Engine ECS resources. |
| `Crates/orbital_shader_preprocessor/src/registry.rs` | WGSL node registry. |
| `Crates/orbital_shader_core/src/lib.rs` | `ShaderDescriptor` and pipeline plumbing. |
| `Crates/orbital_renderer/src/renderer.rs` | Pass ordering, models, overlays, 2D/UI/text. |
| `Crates/orbital_importer_gltf/src/lib.rs` | Async glTF importer. |
| `Crates/orbital_iced/src/bridge.rs` | Iced panel/module bridge. |
| `.forgejo/workflows/lint.yml` | Strict Forgejo fmt/clippy. |
| `.github/workflows/main.yml` | GitHub lint/test/release. |
| `.teamcity/settings.kts` | TeamCity configuration. |
| `Assets/Scenes/procgeo_demo.ron` | Procedural scene example. |
