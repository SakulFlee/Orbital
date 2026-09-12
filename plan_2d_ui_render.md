# Orbital Engine: 2D Rendering & UI System — Master Plan

## Design Decisions

| Decision | Choice |
|----------|--------|
| Layout system | Absolute positioning + horizontal/vertical stacking |
| 2D Camera | Orthographic + perspective |
| Rendering | Batched (by shape type, one texture per draw call) |
| Font loading | fontdue crate, bundle default font |
| Text rendering | SDF (2D UI + 3D name tags) |
| UI communication | Generic `Events<T>` system |
| UI structure | ECS components per element |
| Layer system | Global configuration, configurable ordering |
| UI animations | Not in v1 |
| Sprite batching | One texture per draw call (optimize later) |

---

## Phase 1: ECS Extensions

**Goal**: Parent/child hierarchy + generic events. Prerequisites for UI.

### 1.1 Parent/Child Hierarchy
- `Parent(Entity)` component — marks child
- `Children(Vec<Entity>)` component — marks parent
- `sys_propagate_transforms()` — BFS from roots, computes world transforms
- `sys_clean_hierarchy()` — removes stale refs on despawn

### 1.2 Generic Events<T>
- `Events<T: 'static + Send + Sync>` resource with `send()`, `read()`, `clear()`
- Stored as `Events<T>` in ECS resources
- `ResMut<Events<T>>` works with `IntoSystem`

### Deliverables
- [ ] `Parent` / `Children` components
- [ ] `sys_propagate_transforms` + `sys_clean_hierarchy`
- [ ] `Events<T>` generic resource
- [ ] Unit tests

---

## Phase 2: 2D Rendering Core

**New crates**: `orbital_2d`, `orbital_shader_2d`

### 2.1 2D Camera
```rust
pub enum Camera2D {
    Orthographic { eye: Point2<f32>, zoom: f32, near: f32, far: f32 },
    Perspective { eye: Point3<f32>, fov: Rad<f32>, near: f32, far: f32 },
}
```

### 2.2 Shape Generation (Edge-Count Based)
- 3=triangle, 4=quad, 8=octagon, 16+=circle
- Vertex format: `[f32x2 pos, f32x4 color, f32x2 texcoord, f32x2 shape_params]`
- Solid color + textured fills

### 2.3 Batched Renderer
- Collects 2D draw requests per frame
- Sorts by (pipeline, texture, shape_type)
- Merges into shared vertex/index buffers
- Minimal draw calls

### 2.4 Shader Nodes
- `vertex_2d_input`, `vertex_2d_output`, `fragment_2d_output`, `shape_utils`
- Registered via `register_global_library()` at startup

### Deliverables
- [ ] `Camera2D` (orthographic + perspective)
- [ ] Edge-count shape generation
- [ ] `Batch2D` renderer
- [ ] `Renderer2D` GPU resources
- [ ] 2D shader nodes
- [ ] Unit tests

---

## Phase 3: Text Rendering

**New crate**: `orbital_text`

### 3.1 Font Loading
- fontdue crate, bundle default font (e.g., Inter)
- `FontLibrary` with named fonts + default fallback

### 3.2 SDF Atlas
- Generate SDF glyph atlas per font
- Shelf-pack into texture atlas
- Cache glyph metrics (UV, size, offset, advance)

### 3.3 Text Mesh
- Each char = textured quad
- Word wrapping via `max_width`
- Returns `Vec<Vertex2D>` for 2D, vertex data for 3D billboard

### 3.4 SDF Shader
- `smoothstep` for resolution-independent edges
- Works for both 2D screen-space and 3D world-space

### Deliverables
- [ ] `FontLibrary` with bundled default font
- [ ] SDF atlas generation
- [ ] Text mesh generation (2D + 3D)
- [ ] SDF shader node
- [ ] Billboard text support

---

## Phase 4: Render Layer System

**Modify**: `orbital_renderer`, `orbital_app`

### 4.1 Layer Definition
```rust
pub enum RenderLayer {
    Skybox,              // 0
    Scene3D,             // 1
    Scene3DTransparent,  // 2 [future]
    World2D,             // 3
    Overlay3D,           // 4
    UI,                  // 5
    Debug,               // 6
}
```

### 4.2 Layer-Aware Passes

| Layer | Projection | Depth | Blend | Clear |
|-------|-----------|-------|-------|-------|
| Skybox | Perspective | No | No | Clear |
| Scene3D | Perspective | Yes | No | Load |
| World2D | Orthographic | No | Alpha | Load |
| Overlay3D | Perspective | Yes | Alpha | Load |
| UI | Orthographic | No | Alpha | Load |
| Debug | Perspective | No | Alpha | Load |

### 4.3 LayerRenderer Trait
```rust
pub trait LayerRenderer: Send + Sync {
    fn layer(&self) -> RenderLayer;
    fn render(&mut self, ctx: RenderLayerContext);
}
```

### Deliverables
- [ ] `RenderLayer` enum + `LayerConfig`
- [ ] `LayerRenderer` trait
- [ ] Layer-aware render pass orchestration
- [ ] Integration into `ModuleRuntime::redraw()`

---

## Phase 5: UI System

**New crate**: `orbital_ui`

### 5.1 Components
- `UiEntity` — marker
- `UiElement` — id, visible, z_index
- `UiLayout` — Absolute | Horizontal | Vertical
- `UiButton`, `UiText`, `UiTextBox`, `UiCheckbox`, `UiImage`, `UiBackground`

### 5.2 Events
- `ButtonPressed`, `ButtonReleased`, `TextBoxChanged`, `CheckboxToggled`
- Stored as `Events<T>` resources

### 5.3 Layout
- `sys_layout_ui` resolves layouts using parent/child hierarchy
- Absolute: direct position
- Horizontal/Vertical: stack children with spacing + alignment

### 5.4 Hit Testing
- `sys_ui_hit_test` finds topmost UI element under cursor
- Checks z_index for overlapping elements

### 5.5 Rendering
- `sys_ui_render` queries visible UI elements
- Renders backgrounds (SDF corner radius), text, images
- Handles focus highlights

### 5.6 UiModule
- Implements `Module` trait
- Registers events, state, systems

### Deliverables
- [ ] UI components
- [ ] UI events
- [ ] Layout system
- [ ] Hit testing
- [ ] UI renderer
- [ ] `UiModule`

---

## Phase 6: Integration & Templates

### 6.1 Workspace Integration
- Add new crates to root `Cargo.toml`
- Add to `orbital` facade crate
- Register shader libraries in `ModuleRuntime::resumed()`

### 6.2 Update All-in-One Template
- Add UI + 2D example code
- Show: UiModule with Button, Text, event handling

### 6.3 Create New 2D Template
- Minimal 2D game setup
- Camera2D, shapes, sprites

### 6.4 Register New Template
- Add `"2d"` to template list in CLI

### 6.5 Examples
- `Examples/ui_demo/` — Button, Text, TextBox, Checkbox, event handling
- `Examples/2d_scene/` — Shapes, text, sprites, 2D camera

### Deliverables
- [ ] Workspace integration
- [ ] Updated all-in-one template with UI + 2D
- [ ] New 2d template
- [ ] Template registration in CLI
- [ ] UI example
- [ ] 2D scene example

---

## Implementation Sessions

| Session | Work | Tests |
|---------|------|-------|
| **1** | Phase 1: ECS hierarchy + Events<T> | Unit tests for hierarchy, events |
| **2** | Phase 2a: 2D camera + shape generation | Shape generation tests |
| **3** | Phase 2b: Batch renderer + shader nodes | Batch merging tests |
| **4** | Phase 3: Font loading + SDF atlas | Atlas generation tests |
| **5** | Phase 3b: Text mesh + SDF shader | Text layout tests |
| **6** | Phase 4: Render layer system | Layer ordering tests |
| **7** | Phase 5a: UI components + events | Component creation tests |
| **8** | Phase 5b: Layout + hit testing | Layout resolution tests |
| **9** | Phase 5c: UI rendering + module | Integration tests |
| **10** | Phase 6: Templates + examples | Template generation tests |

---

## Dependency Graph

```
Phase 1: ECS Extensions
    │
    ├──> Phase 2: 2D Rendering Core
    │        │
    │        ├──> Phase 3: Text Rendering
    │        │
    │        └──> Phase 4: Render Layer System
    │                 │
    │                 └──> Phase 5: UI System
    │                          │
    │                          └──> Phase 6: Integration
```

---

## Files to Create (Summary)

### New Crates
```
Crates/orbital_2d/
  Cargo.toml
  src/lib.rs
  src/camera.rs
  src/shape.rs
  src/batch.rs
  src/renderer.rs

Crates/orbital_shader_2d/
  Cargo.toml
  src/lib.rs

Crates/orbital_text/
  Cargo.toml
  src/lib.rs
  src/font.rs
  src/sdf_atlas.rs
  src/text_mesh.rs

Crates/orbital_ui/
  Cargo.toml
  src/lib.rs
  src/components.rs
  src/events.rs
  src/layout.rs
  src/input.rs
  src/render.rs
  src/widgets/mod.rs
  src/widgets/button.rs
  src/widgets/text.rs
  src/widgets/textbox.rs
  src/widgets/checkbox.rs
  src/widgets/image.rs
```

### New Template
```
Tools/orbital-cli/src/template/2d/
  Cargo.toml
  src/lib.rs
  src/main.rs
  Assets/
```

### Files to Modify
```
Cargo.toml (root) — Add workspace members
Crates/ecs/src/ — Add Parent, Children, Events<T>
Crates/ecs_bridge/src/ — Add UI component types
Crates/orbital_renderer/src/renderer.rs — Layer system
Crates/orbital_app/src/module_runtime.rs — Integrate layers
Crates/orbital_app/src/render_overlay.rs — LayerRenderer trait
Crates/orbital/src/lib.rs — Re-export new modules
Tools/orbital-cli/src/template/all-in-one/src/lib.rs — Add UI/2D
Tools/orbital-cli/src/init/template.rs — Register 2d template
Tools/orbital-cli/src/init/prompt.rs — Add 2d to list
```
