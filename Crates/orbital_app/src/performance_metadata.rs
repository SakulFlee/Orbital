//! Startup metadata for performance captures.
//!
//! The per-frame CSV is only useful when the capture records the hardware,
//! display, surface, renderer and scene state it was taken under. This module
//! builds that snapshot once per resume and exposes it both as a log line and
//! as a sidecar next to the CSV.

use orbital_ecs::World;
use orbital_ecs_bridge::{LightDescriptor, ModelDescriptor, VolumetricSettings};
use orbital_shadow::ShadowCaster;
use serde_json::{Value, json};
use wgpu::{Features, SurfaceConfiguration};
use winit::window::Fullscreen;

use crate::{AppContext, AppSettings, RenderSettings};

/// Builds the startup metadata snapshot.
///
/// `overlays` contains `(name, visible)` pairs for the registered overlay
/// layers. Visibility is only a snapshot; the CSV records it per row as well.
pub fn collect(
    settings: &AppSettings,
    ecs: &World,
    ctx: &AppContext,
    config: &SurfaceConfiguration,
    overlays: &[(String, bool)],
) -> Value {
    let adapter = ctx.adapter().get_info();
    let window = ctx.window();
    let monitor = window.current_monitor();
    let inner_size = window.inner_size();

    let mut system = sysinfo::System::new();
    system.refresh_cpu_all();
    system.refresh_memory();
    let cpu_brand = system
        .cpus()
        .first()
        .map(|cpu| cpu.brand().trim().to_string())
        .unwrap_or_default();
    let logical_cores = std::thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or_else(|_| system.cpus().len());
    let physical_cores = sysinfo::System::physical_core_count();
    let total_memory_mb = system.total_memory() / (1024 * 1024);
    let available_memory_mb = system.available_memory() / (1024 * 1024);

    let (monitor_name, monitor_refresh_mhz, monitor_width, monitor_height) = monitor
        .map(|monitor| {
            let size = monitor.size();
            (
                monitor.name().unwrap_or_default(),
                monitor.refresh_rate_millihertz().unwrap_or(0),
                size.width,
                size.height,
            )
        })
        .unwrap_or_default();

    let fullscreen = match window.fullscreen() {
        None => "windowed",
        Some(Fullscreen::Borderless(_)) => "borderless",
        Some(Fullscreen::Exclusive(_)) => "exclusive",
    };

    let render_settings = ecs
        .get_resource::<RenderSettings>()
        .map(|settings| *settings)
        .unwrap_or_default();
    let render_scale = render_settings.render_scale.clamp(
        render_settings.min_render_scale,
        render_settings.max_render_scale,
    );
    let render_width = ((config.width as f32 * render_scale).round() as u32).max(1);
    let render_height = ((config.height as f32 * render_scale).round() as u32).max(1);
    let volumetric = ecs
        .get_resource::<VolumetricSettings>()
        .map(|settings| *settings)
        .unwrap_or_default();
    let volumetric_grid = if volumetric.enabled {
        volumetric
            .quality
            .resolved_grid(render_width, render_height, volumetric.max_distance)
    } else {
        [0, 0, 0]
    };
    let volumetric_integration_steps = if volumetric.enabled {
        volumetric
            .quality
            .integration_steps(render_width, render_height, volumetric.max_distance)
    } else {
        0
    };

    let model_count = ecs
        .get_component_store::<ModelDescriptor>()
        .map_or(0, |store| store.dense.len());
    let light_count = ecs
        .get_component_store::<LightDescriptor>()
        .map_or(0, |store| store.dense.len());
    let shadow_caster_count = ecs
        .get_component_store::<ShadowCaster>()
        .map_or(0, |store| store.dense.len());

    let device = ctx.device();
    let features = device.features();
    let limits = device.limits();
    let supported_present_modes = ctx
        .surface()
        .get_capabilities(ctx.adapter())
        .present_modes
        .iter()
        .map(|mode| format!("{mode:?}"))
        .collect::<Vec<_>>();

    json!({
        "schema_version": 1,
        "app": {
            "name": settings.name,
            "version": env!("CARGO_PKG_VERSION"),
            "git_revision": env!("ORBITAL_GIT_REVISION"),
            "debug_assertions": cfg!(debug_assertions),
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
            "os_version": sysinfo::System::long_os_version(),
            "kernel_version": sysinfo::System::kernel_version(),
        },
        "cpu": {
            "brand": cpu_brand,
            "logical_cores": logical_cores,
            "physical_cores": physical_cores,
            "total_memory_mb": total_memory_mb,
            "available_memory_mb": available_memory_mb,
        },
        "gpu": {
            "name": adapter.name,
            "backend": format!("{:?}", adapter.backend),
            "device_type": format!("{:?}", adapter.device_type),
            "vendor": format!("0x{:04X}", adapter.vendor),
            "device": format!("0x{:04X}", adapter.device),
            "driver": adapter.driver,
            "driver_info": adapter.driver_info,
            "features": {
                "timestamp_query": features.contains(Features::TIMESTAMP_QUERY),
                "timestamp_query_inside_encoders": features.contains(Features::TIMESTAMP_QUERY_INSIDE_ENCODERS),
                "shader_f16": features.contains(Features::SHADER_F16),
                "polygon_mode_line": features.contains(Features::POLYGON_MODE_LINE),
            },
            "limits": {
                "max_texture_dimension_2d": limits.max_texture_dimension_2d,
                "max_storage_buffers_per_shader_stage": limits.max_storage_buffers_per_shader_stage,
                "max_bind_groups": limits.max_bind_groups,
            },
        },
        "window": {
            "width": inner_size.width,
            "height": inner_size.height,
            "scale_factor": window.scale_factor(),
            "dpi": effective_dpi(window.scale_factor()),
            "orientation": match config.width.cmp(&config.height) {
                std::cmp::Ordering::Greater => "landscape",
                std::cmp::Ordering::Less => "portrait",
                std::cmp::Ordering::Equal => "square",
            },
            "fullscreen": fullscreen,
            "monitor": {
                "name": monitor_name,
                "refresh_rate_mhz": if monitor_refresh_mhz > 0 {
                    monitor_refresh_mhz
                } else {
                    crate::android_performance::display_refresh_rate_millihertz().unwrap_or(0)
                },
                "width": monitor_width,
                "height": monitor_height,
            },
        },
        "surface": {
            "width": config.width,
            "height": config.height,
            "format": format!("{:?}", config.format),
            "color_space": format!("{:?}", config.color_space),
            "alpha_mode": format!("{:?}", config.alpha_mode),
            "present_mode": format!("{:?}", config.present_mode),
            "supported_present_modes": supported_present_modes,
            "desired_maximum_frame_latency": config.desired_maximum_frame_latency,
            "vsync": settings.vsync_enabled,
            "fps_limit": settings.fps_limit,
        },
        "render": {
            "width": render_width,
            "height": render_height,
            "scale": render_scale,
            "pixels_per_frame": u64::from(render_width) * u64::from(render_height),
            "adaptive": render_settings.adaptive_render_scale,
            "target_fps": render_settings.target_fps,
            "min_scale": render_settings.min_render_scale,
            "max_scale": render_settings.max_render_scale,
        },
        "renderer": {
            "path": if render_settings.deferred_enabled { "deferred" } else { "forward" },
            "anti_aliasing": format!("{:?}", render_settings.anti_aliasing),
            "smaa_quality": format!("{:?}", render_settings.smaa_quality),
            "bloom": render_settings.bloom_enabled,
            "light_glare": render_settings.light_glare,
            "volumetrics": volumetric.enabled,
            "volumetric_quality": format!("{:?}", volumetric.quality),
            "volumetric_grid": volumetric_grid,
            "volumetric_grid_texels": u64::from(volumetric_grid[0])
                * u64::from(volumetric_grid[1])
                * u64::from(volumetric_grid[2]),
            "volumetric_integration_steps": volumetric_integration_steps,
            "volumetric_composite_width": if volumetric.enabled { render_width } else { 0 },
            "volumetric_composite_height": if volumetric.enabled { render_height } else { 0 },
            "volumetric_max_distance": if volumetric.enabled {
                volumetric.max_distance
            } else {
                0.0
            },
            "volumetric_temporal_history": volumetric.enabled && volumetric.temporal_history,
        },
        "android": {
            "refresh_rate_mhz": crate::android_performance::display_refresh_rate_millihertz(),
            "adpf_target_duration_ms": crate::android_performance::adpf_target_duration_ns()
                .map(|ns| ns as f64 / 1_000_000.0),
            "sustained_performance": crate::android_performance::sustained_performance(),
            "power_save": crate::android_performance::power_save(),
            "game_mode": crate::android_performance::game_mode(),
        },
        "scene": {
            "entities": ecs.entity_count(),
            "models": model_count,
            "lights": light_count,
            "shadow_casters": shadow_caster_count,
        },
        "overlays": overlays.iter().map(|(name, visible)| json!({
            "name": name,
            "visible": visible,
        })).collect::<Vec<_>>(),
    })
}

/// Approximate DPI from a window scale factor.
///
/// Android scale factors are density buckets relative to 160 DPI; desktop
/// scale factors are relative to 96 DPI.
fn effective_dpi(scale_factor: f64) -> u32 {
    #[cfg(target_os = "android")]
    const BASE_DPI: f64 = 160.0;
    #[cfg(not(target_os = "android"))]
    const BASE_DPI: f64 = 96.0;

    (scale_factor * BASE_DPI).round().max(1.0) as u32
}

/// Logs the metadata as one machine-readable line.
pub fn log(metadata: &Value) {
    match serde_json::to_string(metadata) {
        Ok(text) => log::info!("PERF metadata: {text}"),
        Err(error) => log::warn!("PERF metadata serialization failed: {error}"),
    }
}
