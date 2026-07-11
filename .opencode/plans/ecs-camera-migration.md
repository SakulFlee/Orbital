# ECS Camera Migration Plan — Orbital Engine

## Architecture Overview

```
ECS World
  Entity (Camera)
    ├── Position(Point3<f32>)                    component
    ├── Rotation(Quaternion<f32>)                component
    ├── CameraDescriptorEcs { fovy, aspect... }  component
    └── CameraRealization(Arc<RwLock<Camera>>)   component → GPU link

No GpuCameraCache — Arc<RwLock<Camera>> lives on entity directly.
```

## Confirmed Decisions

1. **Split CameraDescriptor** — camera-only props (fovy, aspect, near, far, gamma) as `CameraDescriptorEcs`. Position/Rotation from separate components. `Camera::new` / `Camera::update_from_parts` take split params.
2. **Realization = direct World access** — `realize_cameras(ecs: &mut World)`, not an IntoSystem system. Extend macros later.
3. **Arc<RwLock<Camera>>** — shared mutable GPU state on the entity via `CameraRealization` component.
4. **Realization in on_render()** for now; separate schedule later.
5. **Module replaces App** — `fn setup(ecs, device, queue) -> Schedule`.
6. **Elements fully removed** — `ElementStore`, `Element` trait, `CameraEvent`, `CameraTransform`/`Mode<T>` all go away.
7. **Position/Rotation utility methods** — `offset_view_aligned()`, `rotate_pitch()`, `rotate_yaw()`, `rotate_roll()` replace the old Mode-based system. All rotation via quaternion math, no Euler decomposition.
8. **No gimbal lock** — Camera view matrix computed directly from `Quaternion` (no yaw/pitch/roll decomposition). `Camera::new` and `Camera::update_from_parts` take `position: Point3, rotation: Quaternion, camera props: ...`.

## TODO List

### Phase 1: Finish Stage 1 Gaps

- [ ] **1a** Add `sys_update_frame_counter` to `core_schedule.rs`; remove manual increment from `runtime.rs:147-149`
- [ ] **1b** Insert `CursorPosition(Vector2::new(0.0, 0.0))` and `WindowSize(Vector2::new(0, 0))` in `runtime.rs:liftoff()`
- [ ] **1c** Wire `CursorMoved` → `CursorPosition` and `Resized` → `WindowSize` in `runtime.rs:window_event()`
- [ ] **1d** Insert `DeviceResource(Arc::new(device.clone()))` and `QueueResource(Arc::new(queue.clone()))` in `runtime.rs:resumed()` after `on_resume()`

### Phase 2: ECS Infrastructure

- [ ] **2a** Create `Crates/orbital_ecs_bridge/src/components.rs` — `Position(Point3<f32>)`, `Rotation(Quaternion<f32>)` with utility methods: `forward_right_up()` → `(forward, right, up)` unit vectors from quaternion; `rotate_pitch(delta)`, `rotate_yaw(delta)`, `rotate_roll(delta)` → apply axis-angle rotations; `offset(delta)`, `offset_view_aligned(forward, right, up)` → move position relative to orientation
- [ ] **2b** Add `CameraDescriptorEcs` (camera-only: label, aspect, fovy, near, far, global_gamma), `CameraRealization(Arc<RwLock<Camera>>)`, `ActiveCamera(Entity)`, `CameraDirty(bool)` to `components.rs`
- [ ] **2c** Add `Camera::new(position: Point3, rotation: Quaternion, fovy, aspect, near, far, gamma, device, queue)` and `Camera::update_from_parts(position, rotation, fovy, aspect, near, far, gamma, queue)` to `orbital_resources/src/camera/mod.rs`. View matrix computed by rotating basis vectors: `forward = rotation * Vector3::new(0,0,1)`, `right = rotation * Vector3::new(1,0,0)`, `up = right.cross(forward)`. Keep old `from_descriptor` for backward compat. **Add unit test first:** create camera with known Euler angles (e.g. yaw=0.5, pitch=0.3, roll=0.1), compute view matrix via old `calculate_view_projection_matrix` AND new quaternion path, assert they match within epsilon. This validates coordinate convention before any other code changes.
- [ ] **2d** Create `Crates/orbital_app/src/module.rs` — `trait Module { fn setup(ecs: &mut World, device: &Device, queue: &Queue) -> Schedule; }`
- [ ] **2e** Wire `Commands` into `Schedule::run` — change `run(&World)` → `run(&mut World)`, update `System` trait, `Executor` trait, `SnapshotExecutor`. Add minimal `IntoSystem` impls for `Commands` parameter (at minimum: `fn(Commands, Res<A>, Res<B>)`).
- [ ] **2f** Export all new types from `orbital_ecs_bridge/src/lib.rs`

### Phase 3: Camera Entity & Realization

- [ ] **3a** Create `realize_cameras(ecs: &mut World)` function in `Crates/orbital_app/src/systems/realize.rs` — iterate entities with CameraDescriptorEcs+Position+Rotation, check CameraDirty, create/update CameraRealization via `Camera::new(pos, rot, ...)` or `Camera::update_from_parts(pos, rot, ...)`, clear dirty flag
- [ ] **3b** Add `recreate_bind_group_with_camera_buffer(camera_buffer_binding, device, queue)` to `orbital_world/src/world.rs` — same as `recreate_bind_group` but takes external camera buffer binding instead of reading from CameraStore
- [ ] **3c** Update `StandardApp::on_render` in `orbital_app/src/standard.rs` — call `realize_cameras`, get active camera from `ActiveCamera` resource, read `CameraRealization` component, call `realization.0.read().unwrap().camera_buffer().as_entire_buffer_binding()`, pass to `recreate_bind_group_with_camera_buffer`, render as before
- [ ] **3d** Update `StandardApp::on_update` signature to accept `ecs: &mut World` param (via App trait change in `lib.rs`)

### Phase 4: App → Module Transition

- [ ] **4a** Add `Module` param to `App::on_update` and `App::on_render` OR create parallel `Module` path in runtime
- [ ] **4b** Update `runtime.rs` to pass `&mut ecs_world` to app methods
- [ ] **4c** Keep `App` trait temporarily as wrapper that delegates to `Module`

### Phase 5: Example Migration

- [ ] **5a** Migrate `roll_camera` — `RollCameraModule` impl, sys_roll_camera using `rotation.rotate_roll(Rad(2.5 * dt))`
- [ ] **5b** Migrate `pbr_grid` — CameraController → ECS system
- [ ] **5c** Migrate `skybox` — DebugWorldEnvironment → ECS system
- [ ] **5d** Migrate `instancing`
- [ ] **5e** Migrate `gltf_pbr_damaged_helmet`

### Phase 6: Cleanup

- [ ] **6a** Remove `CameraStore` from `orbital_world::World`
- [ ] **6b** Remove `CameraEvent` from `WorldEvent` in `orbital_element`
- [ ] **6c** Remove `CameraTransform` + `Mode<T>` from `orbital_resources` (or deprecate)
- [ ] **6d** Remove `Element` impl from `CameraController`
- [ ] **6e** Remove `ElementStore` from `StandardApp`
- [ ] **6f** Eventually remove `App` trait and `StandardApp` entirely

## Key Files to Modify

| File | Change |
|------|--------|
| `Crates/ecs/src/system/schedule.rs` | `run(&mut World)`, Commands flush |
| `Crates/ecs/src/system/system.rs` | `System::run` gets `&mut Commands` |
| `Crates/ecs/src/system/executor.rs` | `Executor::execute` gets `&mut Commands` |
| `Crates/ecs/src/system/param_resource.rs` | Add `Commands` IntoSystem impls |
| `Crates/orbital_ecs_bridge/src/lib.rs` | Export components module |
| `Crates/orbital_ecs_bridge/src/components.rs` | **NEW** — Position, Rotation, CameraDescriptorEcs, CameraRealization, ActiveCamera, CameraDirty |
| `Crates/orbital_resources/src/camera/mod.rs` | `Camera::new()`, `Camera::update_from_parts()` |
| `Crates/orbital_app/src/module.rs` | **NEW** — Module trait |
| `Crates/orbital_app/src/lib.rs` | Export Module, update App trait |
| `Crates/orbital_app/src/runtime.rs` | Insert missing resources, pass ECS to app methods |
| `Crates/orbital_app/src/core_schedule.rs` | Add `sys_update_frame_counter` |
| `Crates/orbital_app/src/standard.rs` | `on_render` uses ECS camera, `on_update` gets ECS param |
| `Crates/orbital_world/src/world.rs` | `recreate_bind_group_with_camera_buffer()` |
| `Crates/orbital_camera_controller/src/realization.rs` | Remove `Element` impl, expose input logic as utility |
| `Examples/roll_camera/src/lib.rs` | Full rewrite as Module |
| `Examples/pbr_grid/src/lib.rs` | Full rewrite as Module |
| `Examples/skybox/src/lib.rs` | Full rewrite as Module |
| `Examples/instancing/src/lib.rs` | Full rewrite as Module |
| `Examples/gltf_pbr_damaged_helmet/src/lib.rs` | Full rewrite as Module |

## Risks

1. **Coordinate system convention** — Need to verify whether Orbital uses Y-up or Z-up, and the forward axis direction (+Z or -Z). The current code uses `pitch_cos * yaw_cos` for forward which implies a specific convention. The quaternion-based view matrix must match. Verify against existing tests/examples before implementing.
2. **RwLock contention** — If realization + user systems both hold locks on the same Camera, one blocks. Unlikely in practice since realization runs in on_render (sequential) and user systems run in game_schedule (before render).
3. **glTF importer** — Currently creates `CameraDescriptor` with embedded position/rotation (as Euler angles from glTF node transform). After split, importer needs to convert glTF quaternion to our `Rotation` component and spawn entities with separate Position/Rotation/CameraDescriptorEcs.
4. **Bind group layout** — Must remain identical (8 bindings: camera, lights, 6 IBL). Only the camera buffer source changes.
