# Performance Reporting

Orbital can write a CSV report with one row per one-second timing window. The
same window feeds the FPS, timing and cache-counter lines in the console, but
the CSV path is independent and can be enabled on its own.

The runtime default is profile-aware:

- **Debug:** CSV capture is on by default. Set `ORBITAL_PERF_CSV=0` to opt out.
- **Release:** CSV capture is off by default. Set `ORBITAL_PERF_CSV=1` to opt
  in without rebuilding.
- `ORBITAL_PERF_CONSOLE=1` independently enables the console summaries in
  either profile.

## Render scale and performance target

The 3D scene can render at an internal resolution smaller than the window surface. The
tonemap/anti-aliasing pass stretches the result back to the surface, while 2D, UI, and text
overlays stay at native resolution.

`AppSettings` and the live `RenderSettings` resource expose:

- `render_scale` — fixed internal scale (`1.0`, `0.75`, `0.5`, `0.25`, and so on).
- `adaptive_render_scale` — automatically tune the scale once per FPS window.
- `target_fps` — frame rate the adaptive controller tries to hold.
- `fps_limit` — optional runtime frame cap. `AppSettings::fps_limit` sets the
  startup value; `RenderSettings::fps_limit` can change it while the app runs.
- `min_render_scale` / `max_render_scale` — bounds for adaptive scaling (default
  `0.25` / `1.0`).

```rust
let settings = AppSettings {
    target_fps: 30,
    adaptive_render_scale: true,
    ..AppSettings::default()
};
```

Adaptive scaling is off by default on desktop and on by default on Android and iOS. The
target is capped to the display refresh rate when the platform reports one and to a frame
cap configured with `fps_limit` when VSync is disabled. If the display refresh cannot be
queried, the configured target is used as-is.

The frame cap is a runtime control like the render scale. Set it from the Rendering panel
without rebuilding to hold a phone and tablet at the same rate for an A/B capture. The
limiter ignores the cap while VSync is enabled, where presentation is already paced by the
compositor; turn VSync off or use a non-VSync present mode first. Every CSV row records the
active `fps_limit`, so a capture can be split into capped and uncapped segments even when
the cap changes mid-run.

The runtime only lowers the scale immediately. It raises it after four consecutive timing
windows meet the target and only when the next scale is predicted to still meet it. This
avoids visible oscillation when the current scale is just barely fast enough. Every scale
change is logged and recorded in the next CSV row.

On Android, winit does not expose a refresh rate. Orbital falls back to the Java
`Display.getRefreshRate()` value captured at startup, then to ADPF's preferred update rate.
Refresh-rate selection is not exposed by the platform abstraction; match refresh rates
through the device display settings when comparing devices.

The Rendering debug panel exposes adaptive scaling, the target FPS, fixed
1x/0.75x/0.5x/0.25x presets, and the runtime frame cap. Changing the scale or cap there
takes effect without rebuilding the app.

Raw FPS is only directly comparable when the native surface resolution, internal render
resolution, and refresh rate match. Use the per-row capture fields to group samples before
comparing phone and tablet captures.

## Enable CSV capture from the debug UI

1. Start the app with the debug UI available.
   - Debug builds include `DebugUiModule` by default.
   - Release builds need `DebugUiModule::new().allow_in_release(true)`.
2. Press **F1** to open the debug panels.
3. Open the **Performance** panel.
4. Enable **Write performance CSV**.
5. Optionally enable **Log FPS, timings & cache counters** for the equivalent
   console output.

The file is created when the switch is enabled. Disabling and re-enabling
truncates and starts a new file. A metadata sidecar is written next to it:

```text
Logs/performance.csv
Logs/performance_meta.json
```

The runtime uses the process working directory as the storage root. The debug
panel shows both paths under the CSV switch. Copy both files together when
sharing a capture; the metadata sidecar records the hardware, display and
renderer settings that make the CSV comparable.

### Enable from code

The UI is convenient, but the same switch can be set from a module:

```rust
use orbital::ecs::World;
use orbital::ecs_bridge::PerformanceLogConfig;
use orbital::wgpu::{Device, Queue};
use orbital::{Module, ecs::System};

struct PerformanceLogging;

impl Module for PerformanceLogging {
    fn setup(
        &self,
        ecs: &mut World,
        _device: &Device,
        _queue: &Queue,
    ) -> Vec<Box<dyn System>> {
        ecs.insert_resource(PerformanceLogConfig {
            console: false,
            csv: true,
        });
        vec![]
    }
}
```

Add the module after the runtime's default setup so it overrides the startup
values from `PerformanceLogConfig::for_current_build()` (CSV on in debug, off
in release).

## Retrieve the CSV

### Desktop

The file is under the working directory of the process:

| How the app was started | Typical path |
|---|---|
| `cargo run -p template_all_in_one` from the repository root | `<repo>/Logs/performance.csv` |
| `cd Examples/all-in-one && cargo run` | `Examples/all-in-one/Logs/performance.csv` |
| `orbital run` from a generated project | `<project>/Logs/performance.csv` |

Check the current directory and copy both files:

```bash
pwd
ls -l Logs/performance.csv Logs/performance_meta.json
cp Logs/performance.csv ~/orbital-performance.csv
cp Logs/performance_meta.json ~/orbital-performance_meta.json
```

Open the containing directory:

```bash
# Linux
xdg-open Logs

# macOS
open Logs

# Windows (PowerShell)
explorer .\Logs
```

### Android

The app package is the `[android] package` value in `Orbital.toml`
(`de.sakulflee.orbital` in this repository). Android storage is the app's
internal data directory, so the file is normally:

```text
/data/data/<package>/files/Logs/performance.csv
```

For a **debuggable** build, use `run-as`:

```bash
adb devices
adb shell run-as <package> ls -l files/Logs
adb exec-out run-as <package> cat files/Logs/performance.csv > performance.csv
adb exec-out run-as <package> cat files/Logs/performance_meta.json > performance_meta.json
```

`adb exec-out` is preferred over `adb shell ... > file` because it does not
translate line endings. On Windows PowerShell, use UTF-8 explicitly (or run
the commands from Git Bash):

```powershell
adb exec-out run-as <package> cat files/Logs/performance.csv |
  Set-Content -Encoding utf8 performance.csv
```

If your device reports a different internal path, ask the app process:

```bash
adb shell run-as <package> pwd
adb shell run-as <package> find . -name performance.csv
```

A direct pull may work for debug/rooted devices:

```bash
adb pull /data/data/<package>/files/Logs/performance.csv .
```

For a debuggable build, Android Studio's **View → Tool Windows → Device
Explorer** can also browse `data/data/<package>/files/Logs` and save the file.

`run-as` and direct pulls are normally denied for non-debuggable release
builds. Collecting from a release build currently requires a native Android
share/save flow, which is not implemented yet.

### iOS

New projects generated by `orbital init` enable file sharing through
`UIFileSharingEnabled` and `LSSupportsOpeningDocumentsInPlace`. The CSV is in
the app's Documents directory:

```text
Documents/Logs/performance.csv
```

Retrieve it with **Files** on the device:

1. Open **Files**.
2. Go to **On My iPhone/iPad**.
3. Open the app folder.
4. Open **Logs** and copy `performance.csv`.

Or use Finder:

1. Connect the device.
2. Select the device in Finder.
3. Open **Files** and the app's folder.
4. Copy `Documents/Logs/performance.csv`.

Xcode's **Window → Devices and Simulators → Download Container** flow also
works; after downloading, open the container and navigate to
`AppData/Documents/Logs/performance.csv`.

For the iOS Simulator:

```bash
CONTAINER="$(xcrun simctl get_app_container booted <bundle-id> data)"
open "$CONTAINER/Documents/Logs/performance.csv"
```

`<bundle-id>` is the `[ios] bundle_id` value in `Orbital.toml`.

For an existing iOS project made before this change, add these keys to
`iOS/<App>/Info.plist`:

```xml
<key>UIFileSharingEnabled</key>
<true/>
<key>LSSupportsOpeningDocumentsInPlace</key>
<true/>
```

## Metadata sidecar

`Logs/performance_meta.json` is written when CSV capture starts. It records
the capture environment once per resume:

- app version, git revision, build profile, OS/arch
- CPU model, logical/physical cores, total/available RAM
- GPU adapter, backend, device type, vendor/device, driver, features and
  selected limits
- window size, scale factor, DPI estimate, orientation, monitor name/size/refresh,
  fullscreen mode
- surface format, colour space, alpha mode, supported/selected present mode,
  vsync, fps limit, desired frame latency
- internal render resolution, render scale, target FPS, min/max scale,
  pixels per frame
- renderer path (forward/deferred), AA/SMAA, bloom/glare, volumetrics
- scene entity/model/light/shadow-caster counts
- registered overlays and their visibility at startup
- Android refresh rate, ADPF target duration, sustained-performance state,
  power-save state, and game mode when available

The same metadata is emitted as a `PERF metadata:` log line.

## CSV schema

The header is:

```text
elapsed_s,fps,cdt_ms,<timing stage means>,<cache counters>,<surface counters>,visible_overlays,visible_overlay_names,<capture fields>,<timing window fields>,<per-stage distributions>
```

- `elapsed_s` — seconds since the app started.
- `fps` — frames in the last complete one-second window.
- `cdt_ms` — latest wall-clock frame delta in milliseconds.
- Timing stage means — one-second averages for every `TIMING_STAGES` entry, in
  order:
  `surface_acq`, `update`, `ui_events`, `input`, `core_schedule`,
  `importer_poll`, `game_schedule`, `engine_events`, `realize`,
  `realize_cameras`, `realize_lights`, `realize_models`, `realize_env`,
  `stagger`, `cull+extract`, `bind+models`, `volumetrics`, `scene_encode`,
  `scene_submit`, `timestamp_readback`, `overlay_main`, `overlay_layers`,
  `iced_draw`, `iced_submit`, `present`, `total`, `frame_limiter`,
  `frame_gap`, `surface_lock`, `surface_acquire`, `surface_view`,
  `gpu_complete`, `gpu_shadow`, `gpu_skybox`, `gpu_gbuffer`,
  `gpu_light_culling`, `gpu_deferred`, `gpu_models`, `gpu_transparent`,
  `gpu_volumetric_inject`, `gpu_volumetric_accumulate`,
  `gpu_volumetric_composite`, `gpu_scene_total`, `gpu_volumetrics`,
  `gpu_tonemap`, `gpu_aa`.
  The same stage list is repeated at the end of the row as `<stage>_min`,
  `<stage>_max`, and `<stage>_p95` columns.
  `gpu_models` is the forward opaque model section; `gpu_scene_total` is the
  aggregate of shadow, skybox, G-buffer, light culling, deferred lighting,
  forward models and transparent models. `gpu_volumetrics` is the aggregate of
  the volumetric inject, accumulate, and composite stages.
  `surface_lock`, `surface_acquire`, and `surface_view` break down the
  `surface_acq` stage; `surface_acquire` includes waiting for a swapchain
  image, so it can reflect GPU/present backpressure rather than pure CPU work.
  `frame_gap` is the part of the wall-clock frame not covered by
  `update + surface_acq + total + frame_limiter`.
  `gpu_complete` is the time from the end of submission until the GPU reports
  the submitted work complete; it is a latency, not an additive stage.
  `gpu_volumetric_inject`, `gpu_volumetric_accumulate`,
  `gpu_volumetric_composite`, `gpu_tonemap`, and `gpu_aa` cover the post/scene
  GPU work that the older scene timestamps did not include.
- Cache counters — `shader_modules_delta`, `pipelines_delta`, `unique_wgsl`,
  `material_hits_delta`, `material_misses_delta`, `sky_updates_delta`.
  `unique_wgsl` is cumulative; the others are deltas for the window.
- Surface counters — cumulative counts of success, suboptimal, timeout,
  occluded, outdated, lost, and validation results from swapchain acquisition.
- `visible_overlays` and `visible_overlay_names` — how many registered overlay
  layers were visible in that window and their names, separated by `|`. This
  matters because the debug UI itself can dominate a light scene.
- Capture fields — one value per row: `window_width`, `window_height`,
  `surface_width`, `surface_height`, `render_width`, `render_height`,
  `render_scale`, `scale_factor`, `dpi`, `orientation`, `refresh_rate_mhz`,
  `present_mode`, `frame_latency`, `fps_limit`, `pixels_per_frame`,
  `render_scale_changes`, `render_resolution_changes`,
  `gpu_timestamps_supported`, `gpu_timestamps_stale`.
  `refresh_rate_mhz` is empty when the platform does not report a value.
  `fps_limit` is empty when no cap is configured; a value is recorded even
  while VSync is enabled, where the limiter ignores it. The render fields
  change when adaptive scaling or a rotation changes the target size, so a
  capture can be split into comparable segments. The two change
  counters mark the rows where a scale or render-target change happened; a
  non-zero counter means that row covers a transition.
  `gpu_timestamps_stale` is set when GPU timestamp results did not advance
  between two capture rows, so the GPU stage columns should not be compared.
- Timing window fields — `frames` (samples in the window), `cdt_mean_ms`,
  `cdt_min_ms`, `cdt_max_ms`, and `cdt_p95_ms`. `cdt_ms` remains the latest
  frame delta for backward compatibility; use the `cdt_*` columns for
  distributions.

`ORBITAL_PERF_DETAIL=1` additionally logs per-system game-schedule timings and
per-overlay event/draw/submit timings once per second. It is intentionally
opt-in because it adds locking and instrumentation to every system/layer.

The first row appears only after the first complete timing window (about one
second). A header-only file means capture was enabled but no window completed
yet.

## Visualise the report

The analysis notebook is in `Tools/performance/`.

```bash
python3 -m venv .venv
source .venv/bin/activate        # Windows: .venv\Scripts\activate
pip install -r Tools/performance/requirements.txt
jupyter lab Tools/performance/performance_report.ipynb
```

Put the CSV beside the notebook as `performance.csv`, or set the
`ORBITAL_PERF_CSV` environment variable before launching Jupyter:

```bash
ORBITAL_PERF_CSV=~/orbital-performance.csv \
  jupyter lab Tools/performance/performance_report.ipynb
```

The notebook loads `performance_meta.json` when it is next to the CSV, then plots FPS,
frame time, internal render scale and resolution, CPU/GPU stages and cache counters. It also
prints automated observations such as frame-time percentiles, spike counts, the slowest
stages, and the stage share of frame time. A stage-distribution table shows p95 and worst-case
columns, while render-scale change rows and a per-resolution segment table make
phone/tablet captures comparable. Cache deltas, cumulative WGSL source count, and dynamic-sky
updates are plotted on separate axes because they use different units and scales.

## Troubleshooting

- **No file on desktop:** check `pwd`; `Logs/` is relative to the process
  working directory, not the executable.
- **No file on Android/iOS:** make sure the CSV switch is enabled and the app
  ran for at least one second.
- **Android permission denied:** the build is probably not debuggable. Use a
  debug build, or wait for the native share flow.
- **iOS Files app does not show the app:** add the two `Info.plist` keys and
  rebuild.
- **CSV is empty apart from the header:** wait for a full timing window or
  check that the app is rendering rather than suspended.
- **A previous capture disappeared:** enabling the switch truncates the
  existing `Logs/performance.csv`; copy the file before starting a new run.
