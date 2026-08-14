# Orbital Android Port — Issue Analysis & Cross-Platform FileManager (FM) Plan

_Status: FM implemented on `feat/file-manager`. A1–A6 migrated; separated glTF assets and the platform cache dir are handled. Category B/C still open._

## 0. Executive summary

The `android` branch produces a pure-native APK (`android:hasCode="false"`, `android.app.NativeActivity`, `lib*.so` via `cargo-ndk`), but **packages zero assets and remaps zero paths**. All runtime asset access assumes a desktop filesystem and resolves against the process working directory (`/` on Android), so nothing loads. The core fix is a **standalone cross-platform FileManager (FM) crate** that separates **read-only bundled assets** from **read-write storage**, then migrating the ~6 file-touching subsystems onto it.

Blender/model-export in `orbital/build.rs` is **out of scope** (ignored).

## 1. FileManager (FM) design

### 1.1 Crate
New workspace crate **`Crates/orbital_file_manager/`** (auto-included via `Cargo.toml`'s `members = ["Examples/*", "Crates/*"]`). Consumers: `orbital_resources`, `orbital_shader_preprocessor`, `orbital_importer_gltf`, `orbital_procgeo`, and the `orbital` facade (re-export as `orbital::file_manager`). No dependency cycle (nothing in FM needs the engine crates).

### 1.2 Path contract — **no `Assets/` prefix**
Asset paths are **relative to the asset root** (i.e. `"Models/DamagedHelmet.glb"`, `"Shaders/pbr.wgsl"`, `"Scenes/procgeo_demo.ron"`):

| Backend | Asset root | Storage root | Cache root |
|---|---|---|---|
| Desktop (`not(android)`) | `<cwd>/Assets` (current behavior preserved) | `<cwd>` | platform cache dir (`~/.cache`, `%LOCALAPPDATA%`, `~/Library/Caches`) via `dirs::cache_dir()` |
| Android | APK `assets/` via `AAssetManager` | app-internal files dir | app-internal `cache/` subdir |
| (future) WASM / iOS | — | — | — |

Writable user files (procgeo saves, logs) use the **storage** half; caches (IBL) use the **cache** namespace. Assets are read-only.

### 1.3 API sketch

```rust
// orbital_file_manager/src/lib.rs
pub struct FileManager { assets: Box<dyn AssetSource>, storage: Box<dyn Storage> }

pub trait AssetSource {                       // read-only, bundled
    fn read_bytes(&self, path: &str) -> Result<Vec<u8>, FsError>;
    fn read_to_string(&self, path: &str) -> Result<String, FsError>;
    fn open_read(&self, path: &str) -> Result<Box<dyn Read + Send>, FsError>;
    fn list_dir(&self, path: &str) -> Result<Vec<String>, FsError>;
    fn path_exists(&self, path: &str) -> bool;
}
pub trait Storage {                           // read-write
    fn read_bytes(&self, path: &str) -> Result<Vec<u8>, FsError>;
    fn read_to_string(&self, path: &str) -> Result<String, FsError>;
    fn write_bytes(&self, path: &str, data: &[u8]) -> Result<(), FsError>;
    fn open_write(&self, path: &str) -> Result<Box<dyn Write + Send>, FsError>; // + append
    fn create_dir_all(&self, path: &str) -> Result<(), FsError>;
}

impl FileManager { pub fn init_global(); pub fn global() -> &'static Self; /* + delegating helpers */ }
```

### 1.4 Android init path
`android_main` (extended in `make_android_main!`/`make_main!`) initializes the global FM from the `AndroidApp` handed to winit — it carries the `AAssetManager`, and `ndk_context` provides the Java context for internal dirs. Desktop uses the default backend; the global is a lazy singleton so uninitialized paths fail gracefully (`FsError`), never panic.

### 1.5 Integration targets

| Blocker | Site | Change |
|---|---|---|
| A2 | `orbital_importer_gltf/src/lib.rs:60-76`, `gltf/mod.rs:66` | **Resolved:** custom in-memory import — `FM.read_bytes` → `gltf::Gltf::from_slice`, then FM-backed reads for external buffer/image URIs (relative to the glTF's own directory), base64 `data:` URIs, `image`-crate decode. `.gltf` + separated `.bin`/`.png` assets work on desktop **and** Android; `.glb` unchanged |
| A3 | `orbital_resources/src/texture/mod.rs:86-88, 249, 281-294` | `FM.read_bytes` → `ImageReader::new(Cursor::new(bytes)).with_guessed_format()` |
| A4 | `world_environment/mod.rs:530` (`radiance_hdr_file`) | same as A3 |
| A1 + B6 | `world_environment/mod.rs:281-295, 322, 402`, `cache_file.rs:25-64, 66-103` | **Resolved:** `CacheFile::from_path/to_path` use the FM **cache** namespace; `find_cache_dir()`'s `dirs::cache_dir().expect(...)` panic is gone. Desktop cache = `dirs::cache_dir()`, Android cache = internal `cache/` |
| A5 | `shader/source.rs:20`, `shader_preprocessor/lib.rs:26, 29, 55-72, 93, 108-113`, `pbr_material_shader/mod.rs:142`, `procgeo/scene/material.rs:151` | `FM.read_to_string` for `ShaderSource::Path`; preprocessor folder import replaces `glob`+`read_to_string` with `FM.list_dir` + `FM.read_to_string` |
| Write support | `procgeo/scene/mod.rs:226, 231`, IBL cache | `SceneBuilder::save` → storage `FM.write_bytes`; `load` → asset `FM.read_to_string` |

**Good news:** shaders embedded via `include_str!` (sky, `shadow_depth.wgsl`, `default_shader.wgsl`) need no change — they're the model to follow for new assets.

## 2. Category A — Hard blockers (excluding Blender/build.rs)

1. **A1 — `dirs::cache_dir().expect(...)` hard-panics on Android** (`dirs` returns `None`). `world_environment/mod.rs:282`. Even a `Some` would be a non-writable sandbox path.
2. **A2 — glTF opened by path** (`gltf::import`). Call sites become `"Models/..."` (below).
3. **A3 — `Texture::from_path` / `ImageReader::open`** (`texture/mod.rs:249`), used by `TextureDescriptor::File` (`texture/descriptor.rs:13-16`).
4. **A4 — HDRI `FromFile` → `ImageReader::open`** (`world_environment/mod.rs:530`; descriptor `world_environment/descriptor.rs:257-283`).
5. **A5 — runtime shader reads**: `ShaderSource::Path` (`shader/source.rs:20`) + preprocessor auto-import scan of `Assets/Shaders` on every compile (`shader_preprocessor/lib.rs`).
6. **A6 — wgpu `request_device` unconditionally requires `Features::POLYGON_MODE_LINE`** (`app/context.rs:142-153`). Many mobile GPUs lack it → device request fails → app never starts. Gate behind `adapter.features().contains(...)` like the timestamp features.

## 3. Category B — Lifecycle / platform API (deferred)

1. **B1 — `std::process::exit`** on `EngineEvent::ForceClose` kills the process; should `event_loop.exit()` (`module_runtime.rs:1091-1094`).
2. **B2 — cursor grab/visibility always fire** (every example sets `CursorGrabConfig(true)`); no-op on Android (`module_runtime.rs:1072-1086, 1192-1203`).
3. **B3 — `Surface<'static>` via `transmute`** (`context.rs:124`) + suspend/resume does not recreate existing GPU resources (`module_runtime.rs:1109-1207`).
4. **B4 — touch input unhandled** (keyboard/mouse only; Android sends `Touch`) (`module_runtime.rs:1257-1339`, `orbital_input/src/event.rs`).
5. **B5 — `Gilrs::new().expect(...)`** hard-crash risk on Android (`module_runtime.rs:173`).
6. **B6 — IBL cache writes** (`cache_file.rs:66-103`) — resolved via FM storage (1.5).

## 4. Category C — Notes

- **Logcat tag mismatch:** Android logger default tag is `rust`; CLI hints `adb logcat -s rust_std_out` (`logging/mod.rs:5-10`, `Tools/orbital-cli/src/android/run.rs`).
- `ORBITAL_STAGGER_ALL` env probe (`module_runtime.rs:296`) — harmless (`None` on Android).
- `.cargo/config.toml` has no Android linkers — fine, `cargo-ndk` injects them.
- Repo `Examples/*` only wire `make_desktop_main!` — Android builds target the *generated* project which uses `make_main!`.
- `rayon` thread-pool `.expect` (`importer_gltf/lib.rs:25-34`) — works on Android but is an unconditional panic point.

## 5. File change list (FM phase)

**New crate:** `Crates/orbital_file_manager/{Cargo.toml, src/lib.rs, src/error.rs, src/dir.rs, src/android.rs}`

**FM additions (second pass):**
- `Storage` cache namespace: `read_cache_bytes` / `write_cache_bytes` / `cache_path_exists`; desktop root = `dirs::cache_dir()`, Android root = `internal_data_path()/cache`.
- `FileManager::new(assets, storage)` for custom backends; `DesktopAssetSource::with_base_dir`.
- `orbital_importer_gltf`: custom in-memory importer (base64 + percent-decode, FM-backed external URIs, `image`-crate decode); `import_with_file_manager` for tests.

**Migrate path strings — drop `Assets/` prefix:**
- `Examples/instancing/src/lib.rs:76` → `"Models/InstancingTest.glb"`
- `Examples/gltf_pbr_damaged_helmet/src/lib.rs:103` → `"Models/DamagedHelmet.glb"`
- `Examples/pbr_grid/src/lib.rs:76` → `"Models/PBR_Grid.glb"`
- `Examples/multi_module/src/modules/model_module.rs:18` → `"Models/Cubes.glb"`
- `Examples/procgeo_scene/src/lib.rs:273` → `"Scenes/procgeo_demo.ron"`, `:305` → `"Models/DamagedHelmet.glb"`
- `Crates/orbital_resources/src/pbr_material_shader/mod.rs:142` → `"Shaders/pbr.wgsl"`
- `Crates/orbital_procgeo/src/scene/material.rs:151` → `"Shaders/wireframe.wgsl"`
- `Crates/orbital_shader_preprocessor/src/lib.rs` → `"Shaders"`

**Code:**
- `Crates/orbital_resources/src/world_environment/mod.rs`, `cache_file.rs` (A1/A4/B6)
- `Crates/orbital_resources/src/texture/mod.rs` (A3)
- `Crates/orbital_resources/src/shader/source.rs` (A5)
- `Crates/orbital_shader_preprocessor/src/lib.rs` (A5)
- `Crates/orbital_importer_gltf/src/lib.rs`, `gltf/mod.rs` (A2)
- `Crates/orbital_procgeo/src/scene/{mod.rs,material.rs}` (A5/write)
- `Crates/orbital_core/src/macros/mod.rs` (FM global init in `android_main`)
- `Crates/orbital/src/lib.rs` (re-export FM)
- `Crates/orbital_app/src/context.rs` (A6, `POLYGON_MODE_LINE` gate)

## 6. Verification

- `cargo test --workspace` + `cargo clippy` on desktop (regression: FM desktop backend == current `std::fs` behavior).
- Android runtime: build via the branch tooling (`orbital build android` / `orbital run android`), confirm glTF + IBL + shaders load from the APK's `assets/`.
