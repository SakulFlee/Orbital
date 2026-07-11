# ECS Full Migration Plan — Orbital Engine (Expanded)

## Scope

Remove `App`, `StandardApp`, `Element`, `ElementStore`, and the entire Element event system. Replace with `Module` pattern (like Bevy Plugins). Migrate ALL descriptor stores (Camera, Model, Light, Environment) to ECS components with external GPU realization caches. The glTF importer becomes an ECS system.

---

## New Architecture

### Module System (replaces App + Element)

```rust
pub trait Module: Send + Sync {
    /// Called once at startup. Returns systems to run each frame.
    fn setup(ecs: &mut World, device: &Device, queue: &Queue) -> Vec<Box<dyn System>>;
}
```

- Multiple modules can coexist — each contributes systems to the game schedule
- No `on_update`, no `on_render`, no `AppEvent` — everything is an ECS system
- The runtime owns the ECS world, core schedule, realize schedule, and game schedule

### Runtime Flow

```
AppRuntime::liftoff(module)
  ├── Create ECS world, core_schedule, realize_schedule
  ├── Call module.setup() → Vec<System> → build game_schedule
  └── Run event loop

Each frame (RedrawRequested):
  ├── Timer tick → DeltaTime, InputSnapshot resources
  ├── core_schedule.run(ecs)      — timing, frame counter
  ├── game_schedule.run(ecs)      — user systems (movement, AI, etc.)
  ├── realize_schedule.run(ecs)   — dirty → GPU cache sync
  └── render(ecs, device, queue)  — build bind group, submit
```

### Engine Events (replaces AppEvent)

```rust
pub enum EngineEvent {
    CursorGrabbed(bool),
    CursorVisible(bool),
    CursorPosition(cgmath::Vector2<f64>),
    RequestClose,
    ForceClose { exit_code: i32 },
    RequestRedraw,
}
```

Stored in an `EngineEvents` resource. Processed by the runtime after schedules run. No more `AppEvent` routing through trait methods.

### Entity Archetypes

**Camera:**
```
Entity
  ├── Position(Point3<f32>)
  ├── Rotation(Quaternion<f32>)
  ├── CameraDescriptorEcs { label, aspect, fovy, near, far, global_gamma }
  ├── CameraControllerConfig { speed, sensitivity, mouse_input, button_input, ... }
  ├── CameraRealization(Arc<RwLock<Camera>>)
  └── CameraDirty(bool)
```

**Model:**
```
Entity
  ├── Position(Point3<f32>)
  ├── Rotation(Quaternion<f32>)
  ├── ModelDescriptorEcs { label, mesh: Arc<MeshDescriptor>, materials: Vec<Arc<MaterialShaderDescriptor>> }
  ├── ModelInstances(HashMap<Ulid, Transform>)  // instanced rendering
  ├── ModelRealization(Arc<Model>)
  └── ModelDirty(bool)
```

**Light:**
```
Entity
  ├── Position(Point3<f32>)
  ├── Rotation(Quaternion<f32>)        // for directional/spot
  ├── LightDescriptorEcs { light_type, color, ... }
  └── LightDirty(bool)
```
Note: Light GPU buffer is unified (all lights in one buffer). Realization builds the buffer from all dirty lights. Placeholder implementation for now — full rework planned later.

**Environment:**
```
Resource (singleton)
  └── WorldEnvironmentDescriptor
```
Realized by `realize_schedule` into `WorldEnvironment` GPU textures.

### GPU Realization Pattern

```
Descriptor (ECS component)  →  Realization (ECS component)  →  GPU cache
     cheap, clonable              Arc<RwLock<T>> link            heavy GPU state
```

For Camera: `CameraRealization(Arc<RwLock<Camera>>)` — lives on entity
For Model: `ModelRealization(Arc<Model>)` — lives on entity  
For Light: Unified buffer rebuilt from all `LightDescriptorEcs` components
For Environment: `WorldEnvironment` stored as ECS resource after realization

---

## TODO List

### Phase 4: Module Runtime (replace App/StandardApp)

- [ ] **4a** Create `EngineEvent` enum and `EngineEvents` resource in `orbital_ecs_bridge`
- [ ] **4b** Refactor `Module` trait: `setup()` returns `Vec<Box<dyn System>>` (not Schedule)
- [ ] **4c** Create `ModuleApp` struct in `orbital_app` — wraps a Module, implements the runtime loop
  - Owns: `ecs_world`, `core_schedule`, `realize_schedule`, `game_schedule`
  - `liftoff(event_loop, settings, module)` — creates runtime, calls module.setup(), runs event loop
  - `update()`: core_schedule → game_schedule → realize_schedule → process EngineEvents
  - `redraw()`: realize_cameras, build bind group, render
- [ ] **4d** Remove `App` trait, `StandardApp`, and `ElementStore` from `orbital_app`
- [ ] **4e** Move cursor grab/hide processing into runtime (reads `EngineEvents` resource)
- [ ] **4f** Wire `EngineEvent::RequestClose` → exit in runtime
- [ ] **4g** Update `orbital_app/src/lib.rs` — remove old exports, export new types

### Phase 5a: Model ECS Migration

- [ ] **5a.1** Add `ModelDescriptorEcs` component to bridge (mesh, materials)
- [ ] **5a.2** Add `ModelRealization(Arc<Model>)` component to bridge
- [ ] **5a.3** Add `ModelDirty(bool)` component to bridge
- [ ] **5a.4** Add `ModelInstances(HashMap<Ulid, Transform>)` component for instancing
- [ ] **5a.5** Create `realize_models(ecs, device, queue, mesh_cache, material_cache, surface_format)` function — dirty → GPU model creation
- [ ] **5a.6** Create `sys_sync_model_transforms` — reads Position/Rotation, updates ModelDescriptorEcs transforms
- [ ] **5a.7** Add `MeshCache` and `MaterialShaderCache` as ECS resources (currently in World)
- [ ] **5a.8** Remove `ModelStore` from `orbital_world::World`

### Phase 5b: Light ECS Migration

- [ ] **5b.1** Add `LightDescriptorEcs` component to bridge (light_type, color, position, direction)
- [ ] **5b.2** Add `LightDirty(bool)` component to bridge
- [ ] **5b.3** Create `realize_lights(ecs, device, queue)` — rebuilds unified light buffer from all LightDescriptorEcs
- [ ] **5b.4** Add `LightBuffer(wgpu::Buffer)` as ECS resource
- [ ] **5b.5** Remove `LightStore` from `orbital_world::World`

### Phase 5c: Environment ECS Migration

- [ ] **5c.1** Add `WorldEnvironmentDescriptor` as ECS resource (singleton)
- [ ] **5c.2** Create `realize_environment(ecs, device, queue, surface_format)` — descriptor → IBL textures
- [ ] **5c.3** Add `WorldEnvironmentGpu(Option<Arc<WorldEnvironment>>)` as ECS resource
- [ ] **5c.4** Remove `EnvironmentStore` from `orbital_world::World`

### Phase 5d: Import ECS Migration

- [ ] **5d.1** Add `ImportQueue(Vec<ImportTask>)` as ECS resource
- [ ] **5d.2** Create `sys_poll_importer(ecs, device, queue)` — drains ImportQueue, runs importer, spawns entities for resulting models/cameras
- [ ] **5d.3** Add `Importer` as ECS resource (owned by the system)
- [ ] **5d.4** Remove `WorldEvent::Import` handling from old World

### Phase 5e: Bind Group from ECS

- [ ] **5e.1** Update `recreate_bind_group_with_camera_buffer` → `recreate_bind_group_from_ecs(ecs, device, queue)` — reads all GPU state from ECS resources
- [ ] **5e.2** Remove old `World::prepare_render()` and `World::recreate_bind_group()`
- [ ] **5e.3** Remove `orbital_world::World` entirely (or reduce to just bind group helper)

### Phase 5f: Example Migration

- [ ] **5f.1** Migrate `roll_camera` — Module impl, sys_roll_camera, sys_camera_controller
- [ ] **5f.2** Migrate `pbr_grid` — Module impl, spawn camera + environment + models
- [ ] **5f.3** Migrate `skybox` — Module impl, environment cycling system
- [ ] **5f.4** Migrate `instancing` — Module impl, instanced model spawning
- [ ] **5f.5** Migrate `gltf_pbr_damaged_helmet` — Module impl, glTF import system

### Phase 6: Cleanup

- [ ] **6a** Remove `orbital_element` crate (or gut it)
- [ ] **6b** Remove `CameraStore`, `ModelStore`, `LightStore`, `EnvironmentStore`
- [ ] **6c** Remove `CameraEvent`, `ModelEvent`, `LightEvent`, `EnvironmentEvent`, `WorldEvent`
- [ ] **6d** Remove `CameraTransform`, `Mode<T>` from `orbital_resources`
- [ ] **6e** Remove `CameraController` Element impl (keep input utility functions)
- [ ] **6f** Remove old `Camera::from_descriptor` (keep `Camera::new` / `Camera::update_from_parts`)
- [ ] **6g** Clean up `orbital_world` crate — remove stores, simplify or remove World

---

## Migration Order (Recommended)

```
Phase 4 (Module runtime)
  ↓
Phase 5a (Model ECS) + 5b (Light ECS) + 5c (Environment ECS) — can be parallel
  ↓
Phase 5d (Import ECS) — depends on Model + Camera being in ECS
  ↓
Phase 5e (Bind group from ECS) — depends on all GPU state being in ECS
  ↓
Phase 5f (Example migration) — depends on Phase 4 + selective Phase 5
  ↓
Phase 6 (Cleanup) — last, after everything works
```

---

## Key Files to Create/Modify

| File | Action |
|------|--------|
| `orbital_ecs_bridge/src/events.rs` | **NEW** — EngineEvent, EngineEvents resource |
| `orbital_ecs_bridge/src/components.rs` | Add ModelDescriptorEcs, ModelRealization, ModelDirty, LightDescriptorEcs, LightDirty |
| `orbital_app/src/module.rs` | Update Module trait signature |
| `orbital_app/src/runtime.rs` | Rewrite — Module-based runtime, realize_schedule |
| `orbital_app/src/systems/realize.rs` | Extend — realize_models, realize_lights, realize_environment |
| `orbital_app/src/systems/import.rs` | **NEW** — sys_poll_importer |
| `orbital_world/src/world.rs` | Gut — remove all stores, keep bind group helper or remove |
| `orbital_world/src/store/*` | Remove all store files |
| `orbital_element/src/*` | Remove or gut entirely |
| `orbital_camera_controller/src/realization.rs` | Rewrite as ECS system |
| `Examples/*/src/lib.rs` | Rewrite as Module impls |

---

## Risks

1. **Model GPU realization complexity** — ModelStore has instancing deduplication, bounding box processing, mesh/material caching. The ECS version needs to replicate this correctly. The `instance_hash()` logic for GPU instancing is non-trivial.

2. **Light unified buffer** — All lights are packed into a single GPU buffer. The realization system needs to iterate all LightDirty entities, rebuild the buffer, and update the ECS resource. This is a full-rebuild-on-any-change pattern.

3. **Importer async** — The glTF importer uses rayon + mpsc channels. The ECS system needs to poll the channel and spawn entities for completed imports. Must handle the case where imports complete over multiple frames.

4. **Bind group dependencies** — The bind group needs camera buffer + light buffer + IBL textures all available. The realize_schedule must ensure all realization systems run before render.

5. **Backward compatibility** — All 5 examples need complete rewrites. The CameraController input logic is complex (mouse, gamepad, keyboard, axis/button mappings) and needs careful porting.
