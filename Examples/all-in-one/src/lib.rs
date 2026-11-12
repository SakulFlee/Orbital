use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, TryRecvError};

use orbital::app::{
    App, AppSettings, CursorToggle, LayerRenderer, Module, RenderLayer, RenderOverlayContext,
    sys_camera_controller, sys_ortho_dolly,
};
use orbital::billboard::{
    Billboard, BillboardMode, BillboardModule, SharedTextAtlas, Text, text_billboard_with_atlas,
};
use orbital::cgmath::{InnerSpace, Point3, Quaternion, Rad, Rotation3, Vector3};
use orbital::debug_render::DebugModule;
use orbital::ecs::{Commands, ComponentAccess, IntoSystem, Res, ResMut, System, World};
use orbital::ecs_bridge::{
    ActiveCamera, CameraDescriptor, CursorGrabConfig, CursorGrabState, DeltaTime,
    EnvironmentDescriptorResource, ImportQueueResource, LightDescriptor, LightDirty,
    ModelDescriptor, ModelDirty, ModelInstances, Position, Rotation, VolumetricQuality,
    VolumetricSettings,
};
use orbital::importer::{ImportTask, gltf::GltfImport};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use orbital::logging;
use orbital::logging::{error, info};
use orbital::procgeo::scene::SceneBuilder;
use orbital::procgeo::shapes::box_;
use orbital::renderer::{AntiAliasing, Camera2DUniform, Renderer2D};
use orbital::resources::WorldEnvironmentDescriptor;
use orbital::resources::{
    AlphaMode, GeneratedSkyParameters, MaterialShaderDescriptor, SamplingType, ShadowCaster,
    SunPosition, Transform, VertexStageLayout,
};
use orbital::text::{DefaultFont, TextConfig};
use orbital::twod::Vertex2D;
use orbital::winit::keyboard::KeyCode;

pub const NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn procgeo_scene_parses() {
        let scene = SceneBuilder::from_ron(include_str!("../../../Assets/Scenes/procgeo_demo.ron"))
            .expect("all-in-one scene must parse");

        for label in ["Red Sphere", "R6 Pane Red", "Stair 01", "Pedestal"] {
            assert!(
                scene
                    .entities
                    .iter()
                    .any(|entity| entity.label.as_deref() == Some(label)),
                "scene is missing {label}"
            );
        }
    }

    #[test]
    fn transparency_panes_form_a_fan() {
        let scene = SceneBuilder::from_ron(include_str!("../../../Assets/Scenes/procgeo_demo.ron"))
            .expect("all-in-one scene must parse");
        let pane = |name: &str| {
            scene
                .entities
                .iter()
                .find(|entity| entity.label.as_deref() == Some(name))
                .unwrap_or_else(|| panic!("scene is missing {name}"))
        };

        let red = pane("R6 Pane Red");
        let green = pane("R6 Pane Green");
        let blue = pane("R6 Pane Blue");

        // The centre pane faces the room entrance; the outer panes sit
        // closer to it and angle inward so their colours overlap.
        assert!(red.transform.position[0] < green.transform.position[0]);
        assert!(blue.transform.position[0] > green.transform.position[0]);
        assert!(red.transform.position[2] < green.transform.position[2]);
        assert!(blue.transform.position[2] < green.transform.position[2]);
        assert!(red.transform.rotation[2].abs() > 0.1);
        assert!(blue.transform.rotation[2].abs() > 0.1);
        assert!(green.transform.rotation[2].abs() < 1e-3);
    }

    #[test]
    fn bundled_scene_places_the_aa_stairs_in_room_seven() {
        let scene = SceneBuilder::from_ron(include_str!("../Assets/Scenes/procgeo_demo.ron"))
            .expect("bundled all-in-one scene must parse");
        let stairs: Vec<_> = scene
            .entities
            .iter()
            .filter(|entity| {
                entity
                    .label
                    .as_deref()
                    .is_some_and(|label| label.starts_with("Stair "))
            })
            .collect();

        assert_eq!(stairs.len(), 14);
        for stair in stairs {
            assert!(
                (stair.transform.position[0] + 10.0).abs() < 1e-3,
                "{:?} is not in R7",
                stair.label
            );
            assert!(
                stair.transform.position[2] >= 18.0,
                "{:?} is outside R7",
                stair.label
            );
        }
    }

    #[test]
    fn only_room_shells_are_mirrored() {
        assert!(is_room_shell("R1 Floor"));
        assert!(is_room_shell("R1 Right / R3 Left"));
        assert!(!is_room_shell("R6 Pane Red"));
        assert!(!is_room_shell("Stair 01"));
        assert!(!is_room_shell("Pedestal"));

        assert_eq!(second_row_room_name(-20.0), "R6");
        assert_eq!(second_row_room_name(-10.0), "R7");
        assert_eq!(second_row_room_name(0.0), "R8");
        assert_eq!(second_row_room_name(10.0), "R9");
        assert_eq!(second_row_room_name(20.0), "R10");
    }

    #[test]
    fn custom_shader_room_is_labelled() {
        assert!(room_labels().iter().any(|(text, position, _)| {
            *text == "Custom Shader" && position.x == 0.0 && position.z == 14.75
        }));
    }

    #[test]
    fn bundled_scene_includes_the_transparency_panes() {
        let scene = SceneBuilder::from_ron(include_str!("../Assets/Scenes/procgeo_demo.ron"))
            .expect("bundled all-in-one scene must parse");

        for (label, material) in [
            ("R6 Pane Red", "pane_red"),
            ("R6 Pane Green", "pane_green"),
            ("R6 Pane Blue", "pane_blue"),
        ] {
            let pane = scene
                .entities
                .iter()
                .find(|entity| entity.label.as_deref() == Some(label))
                .unwrap_or_else(|| panic!("bundled scene is missing {label}"));
            assert_eq!(pane.material, material);
            assert!(
                scene.materials.contains_key(material),
                "bundled scene is missing the {material} material"
            );
        }
    }
}

/// The two room rows face this central hallway. The second row's shells are
/// placed at `2 * HALLWAY_MIRROR_Z - z`, leaving the corridor between them.
const HALLWAY_MIRROR_Z: f32 = 10.0;

/// Shared style for the floating room labels. Every label uses the same font
/// size and colours, so one atlas and one material serve the whole group.
///
/// Room labels are read from the hallway, roughly 6-20 m away. Halving the
/// SDF font from 128 px to 64 px and doubling `world_scale` keeps the
/// on-screen size while cutting per-glyph SDF work about fourfold. The shared
/// atlas drops from 1024x1024 (4 MiB) to 512x512 (1 MiB).
///
/// Release-build shared-atlas construction for the label set: 1.14 s at
/// 128 px, 0.47 s at 96 px and 0.12 s at 64 px. 96 px was only marginally
/// crisper on the nearest sign, so 64 px is the better trade at room
/// distance.
const LABEL_FONT_SIZE: f32 = 64.0;
const LABEL_ATLAS_SIZE: u32 = 512;
const LABEL_WORLD_SCALE: f32 = 0.0045;
const LABEL_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const LABEL_BACKGROUND: [f32; 4] = [0.025, 0.035, 0.07, 0.55];

/// Room shells are the floor, back wall and side walls; their contents are
/// deliberately *not* mirrored. The first row has R1-R5, while the second row
/// owns R6-R10 and therefore gets only fresh, empty shells.
fn is_room_shell(label: &str) -> bool {
    label.starts_with('R')
        && [" Floor", " Back", " Left", " Right"]
            .iter()
            .any(|suffix| label.contains(suffix))
}

fn second_row_room_name(x: f32) -> &'static str {
    match x.round() as i32 {
        -25 | -20 => "R6",
        -15 => "R6/R7",
        -10 => "R7",
        -5 => "R7/R8",
        0 => "R8",
        5 => "R8/R9",
        10 => "R9",
        15 => "R9/R10",
        20 | 25 => "R10",
        _ => "R?",
    }
}

fn room_shell_kind(label: &str) -> &'static str {
    if label.contains("Floor") {
        "Floor"
    } else if label.contains("Back") {
        "Back"
    } else {
        "Wall"
    }
}

/// Floating room labels, as authored positions and orientations.
fn room_labels() -> [(&'static str, Point3<f32>, BillboardMode); 9] {
    [
        (
            "Metallic Materials",
            Point3::new(-20.0, 6.4, 5.25),
            BillboardMode::YAxis,
        ),
        (
            "Matte Materials",
            Point3::new(-10.0, 6.4, 5.25),
            BillboardMode::YAxis,
        ),
        (
            "Moving Spot",
            Point3::new(0.0, 6.4, 5.25),
            BillboardMode::YAxis,
        ),
        (
            "Helmet Gallery",
            Point3::new(10.0, 6.4, 5.25),
            BillboardMode::YAxis,
        ),
        (
            "Colored Helmet",
            Point3::new(20.0, 6.4, 5.25),
            BillboardMode::YAxis,
        ),
        (
            "Transparency",
            Point3::new(-20.0, 6.4, 14.75),
            BillboardMode::YAxis,
        ),
        (
            "Anti-Aliasing",
            Point3::new(-10.0, 6.4, 14.75),
            BillboardMode::YAxis,
        ),
        (
            "Custom Shader",
            Point3::new(0.0, 6.4, 14.75),
            BillboardMode::YAxis,
        ),
        (
            "All-in-One",
            Point3::new(0.0, 7.6, HALLWAY_MIRROR_Z),
            BillboardMode::Full,
        ),
    ]
}

fn build_text_label(
    font: &mut orbital::text::FontData,
    atlas: &SharedTextAtlas,
    text: &str,
    position: Point3<f32>,
    mode: BillboardMode,
) -> (Text, Billboard) {
    let label = text_billboard_with_atlas(
        text,
        font,
        &TextConfig {
            font_size: LABEL_FONT_SIZE,
            color: LABEL_COLOR,
            atlas_size: LABEL_ATLAS_SIZE,
            ..Default::default()
        },
        atlas,
        LABEL_WORLD_SCALE,
        None,
        Some(LABEL_BACKGROUND),
        AlphaMode::Blend,
    )
    .with_position(position)
    .with_mode(mode);
    let text = Text::new(text)
        .with_font_size(LABEL_FONT_SIZE)
        .with_color(LABEL_COLOR)
        .with_background(Some(LABEL_BACKGROUND))
        .with_world_scale(LABEL_WORLD_SCALE)
        .with_atlas_size(LABEL_ATLAS_SIZE)
        .with_alpha_mode(AlphaMode::Blend)
        .with_shared_atlas(atlas.clone());
    (text, label)
}

/// Builds the full label group, including the shared SDF atlas.
///
/// This is expensive enough to trip Android's ANR watchdog when run during
/// startup, so callers must invoke it on a worker thread.
fn build_room_labels() -> Vec<(Text, Billboard)> {
    let labels = room_labels();
    let mut font = DefaultFont::load();
    let shared_atlas = SharedTextAtlas::build_with_atlas_size(
        &mut font,
        LABEL_FONT_SIZE,
        LABEL_ATLAS_SIZE,
        labels.iter().map(|(text, _, _)| *text),
    );
    labels
        .into_iter()
        .map(|(text, position, mode)| {
            build_text_label(&mut font, &shared_atlas, text, position, mode)
        })
        .collect()
}

fn spawn_scene_model(
    ecs: &mut World,
    label: &str,
    mesh: orbital::resources::MeshDescriptor,
    material: Arc<orbital::resources::MaterialShaderDescriptor>,
    transform: Transform,
) {
    let entity = ecs.spawn_entity();
    ecs.attach_component(
        &entity,
        ModelDescriptor {
            label: label.to_string(),
            mesh: Arc::new(mesh),
            materials: vec![material],
        },
    )
    .unwrap();

    let mut instances = ModelInstances::new();
    instances.add_instance(transform);
    ecs.attach_component(&entity, instances).unwrap();
    ecs.attach_component(&entity, ModelDirty(true)).unwrap();
}

#[orbital::main]
pub fn entrypoint(
    event_loop_result: Result<
        orbital::winit::event_loop::EventLoop<()>,
        orbital::winit::error::EventLoopError,
    >,
) {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    logging::init();

    let event_loop = event_loop_result.expect("Event Loop failure");

    let present_mode = match std::env::var("ORBITAL_PRESENT_MODE").as_deref() {
        Ok("immediate") => Some(orbital::wgpu::PresentMode::Immediate),
        Ok("mailbox") => Some(orbital::wgpu::PresentMode::Mailbox),
        Ok("fifo") => Some(orbital::wgpu::PresentMode::Fifo),
        Ok("fiforelaxed") => Some(orbital::wgpu::PresentMode::FifoRelaxed),
        _ => None,
    };

    let app_settings = AppSettings {
        vsync_enabled: false,
        fullscreen: std::env::var_os("ORBITAL_FULLSCREEN").is_some(),
        present_mode,
        name: NAME.to_string(),
        // Showcase the higher-quality LUT-based SMAA path in the flagship example.
        anti_aliasing: AntiAliasing::Smaa,
        bloom_enabled: true,
        bloom_threshold: 8.0,
        bloom_intensity: 1.0,
        light_glare: true,
        exposure: 1.0,
        contrast: 1.0,
        back_presses_to_exit: 3,
        ..AppSettings::default()
    };

    match App::new()
        .add_module(ProcgeoSceneModule)
        .add_module(BillboardModule)
        .add_module(
            DebugModule::new()
                .with_toggle_key(KeyCode::F3)
                .with_freeze_key(KeyCode::F4),
        )
        .add_module(orbital::debug_ui::DebugUiModule::new())
        .add_module(orbital::touch_ui::TouchUiModule)
        .liftoff(event_loop, app_settings)
    {
        Ok(()) => info!("Cleanly exited!"),
        Err(e) => error!("Runtime failure: {e:?}"),
    }
}

struct HelmetAdjuster {
    adjusted: AtomicBool,
    access: ComponentAccess,
}

impl HelmetAdjuster {
    fn new() -> Self {
        Self {
            adjusted: AtomicBool::new(false),
            access: ComponentAccess::new()
                .reads::<ModelDescriptor>()
                .reads::<ModelInstances>()
                .writes::<ModelInstances>()
                .writes::<ModelDirty>(),
        }
    }
}

impl System for HelmetAdjuster {
    fn name(&self) -> &str {
        "helmet_adjuster"
    }

    fn access(&self) -> &ComponentAccess {
        &self.access
    }

    fn run(&mut self, world: &World, commands: &mut Commands) {
        if self.adjusted.load(Ordering::Relaxed) {
            return;
        }

        let descs = match world.get_component_store::<ModelDescriptor>() {
            Some(s) => s,
            None => return,
        };

        for &eid in descs.dense.as_slice() {
            let idx = match descs.sparse.get(eid).and_then(|x| *x) {
                Some(i) => i,
                None => continue,
            };
            if !descs.components[idx]
                .label
                .to_lowercase()
                .contains("helmet")
            {
                continue;
            }

            let generation = world.generation(eid);
            let entity = orbital::ecs::Entity::new(eid, generation);

            // Read original transform from imported glTF, only modify position
            let original_transform = world
                .get_component_store::<ModelInstances>()
                .and_then(|store| {
                    store.sparse.get(eid).and_then(|x| *x).map(|idx| {
                        let map = &store.components[idx].0;
                        map.values().next().copied().unwrap_or(Transform::new(
                            Vector3::new(0.0, 0.0, 0.0),
                            Quaternion::new(1.0, 0.0, 0.0, 0.0),
                            Vector3::new(1.0, 1.0, 1.0),
                        ))
                    })
                })
                .unwrap_or(Transform::new(
                    Vector3::new(10.0, 1.85, 0.0),
                    Quaternion::new(
                        std::f32::consts::FRAC_1_SQRT_2,
                        0.0,
                        -std::f32::consts::FRAC_1_SQRT_2,
                        0.0,
                    ),
                    Vector3::new(1.0, 1.0, 1.0),
                ));

            // Rotate 90° Y so the helmet faces +Z (toward camera)
            let correction = Quaternion::new(
                std::f32::consts::FRAC_1_SQRT_2,
                0.0,
                std::f32::consts::FRAC_1_SQRT_2,
                0.0,
            );
            let final_rot = correction * original_transform.rotation;

            let mut new_instances = ModelInstances::new();
            for x in [10.0, 20.0] {
                new_instances.add_instance(Transform::new(
                    Vector3::new(x, 2.2, 0.0),
                    final_rot,
                    original_transform.scale,
                ));
            }
            commands.detach_component::<ModelInstances>(&entity);
            commands.attach_component(&entity, new_instances);
            commands.detach_component::<ModelDirty>(&entity);
            commands.attach_component(&entity, ModelDirty(true));

            self.adjusted.store(true, Ordering::Relaxed);
            info!("Adjusted helmet instances in R4 and R5");
            break;
        }
    }
}

/// Frames to present before attaching the room labels, so their material
/// realization does not compete with startup and the first frame.
const LABEL_ATTACH_DELAY_FRAMES: u8 = 3;

/// Attaches room labels once their background atlas build completes.
struct AsyncRoomLabels {
    receiver: Option<Receiver<Vec<(Text, Billboard)>>>,
    delay_frames: u8,
    access: ComponentAccess,
}

impl AsyncRoomLabels {
    fn new(receiver: Receiver<Vec<(Text, Billboard)>>) -> Self {
        Self {
            receiver: Some(receiver),
            delay_frames: LABEL_ATTACH_DELAY_FRAMES,
            access: ComponentAccess::new()
                .writes::<Text>()
                .writes::<Billboard>(),
        }
    }
}

impl System for AsyncRoomLabels {
    fn name(&self) -> &str {
        "async_room_labels"
    }

    fn access(&self) -> &ComponentAccess {
        &self.access
    }

    fn run(&mut self, _world: &World, commands: &mut Commands) {
        if self.delay_frames > 0 {
            self.delay_frames -= 1;
            return;
        }

        let Some(receiver) = self.receiver.as_ref() else {
            return;
        };
        let labels = match receiver.try_recv() {
            Ok(labels) => labels,
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => {
                error!("Room-label worker stopped before producing labels");
                self.receiver = None;
                return;
            }
        };
        self.receiver = None;

        let count = labels.len();
        for (text, billboard) in labels {
            let entity = commands.spawn_entity();
            commands.attach_component(&entity, text);
            commands.attach_component(&entity, billboard);
        }
        info!("Attached {count} room labels from the background atlas build");
    }
}

struct ProcgeoSceneModule;

// ── Light animation for shadow diagnosis ─────────────────────────────

struct LightAnimator {
    t: f32,
    access: ComponentAccess,
    entity: orbital::ecs::Entity,
    x: f32,
    z: f32,
}

impl LightAnimator {
    fn new(entity: orbital::ecs::Entity, x: f32, z: f32) -> Self {
        Self {
            t: 0.0,
            entity,
            x,
            z,
            access: ComponentAccess::new()
                .reads::<Position>()
                .writes::<Position>()
                .writes::<LightDirty>(),
        }
    }
}

impl System for LightAnimator {
    fn name(&self) -> &str {
        "light_animator"
    }
    fn access(&self) -> &ComponentAccess {
        &self.access
    }

    fn run(&mut self, world: &World, commands: &mut Commands) {
        let dt = world
            .get_resource::<orbital::ecs_bridge::DeltaTime>()
            .map(|d| d.0)
            .unwrap_or(0.016);
        self.t += dt as f32;
        let y = (self.t * 0.8).sin() * 2.0 + 7.0;
        let new_pos = Position(Point3::new(self.x, y, self.z));
        commands.detach_component::<Position>(&self.entity);
        commands.attach_component(&self.entity, new_pos);

        commands.detach_component::<LightDirty>(&self.entity);
        commands.attach_component(&self.entity, LightDirty(true));
    }
}

// ── 2D shape overlay ──────────────────────────────────────────────

struct ShapeOverlay {
    renderer: Renderer2D,
    camera_buffer: orbital::wgpu::Buffer,
    camera_bind_group: orbital::wgpu::BindGroup,
    vertices: Vec<Vertex2D>,
    /// Screen size (physical pixels) the vertices were last laid out for.
    ///
    /// Vertex positions are baked in absolute pixels (`cx = screen_w / 2.0`),
    /// so they must be regenerated whenever the screen size changes — e.g. on
    /// Android screen rotation — otherwise the shapes stay anchored to the
    /// old width while the projection follows the new one.
    last_layout_size: Option<(f32, f32)>,
}

impl ShapeOverlay {
    fn new(device: &orbital::wgpu::Device, format: orbital::wgpu::TextureFormat) -> Self {
        let renderer = Renderer2D::new(device, format);

        let camera_buffer = device.create_buffer(&orbital::wgpu::BufferDescriptor {
            label: Some("2D Camera Buffer"),
            size: std::mem::size_of::<Camera2DUniform>() as u64,
            usage: orbital::wgpu::BufferUsages::UNIFORM | orbital::wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let camera_bind_group = renderer.create_bind_group(device, &camera_buffer);

        Self {
            renderer,
            camera_buffer,
            camera_bind_group,
            vertices: Vec::new(),
            last_layout_size: None,
        }
    }

    fn layout_shapes(&mut self, screen_w: f32, screen_h: f32) {
        self.vertices.clear();
        let cx = screen_w / 2.0;
        let top = 15.0;

        // Green circle — left
        self.vertices
            .extend_from_slice(&orbital::twod::shape::generate_circle(
                [cx - 60.0, top + 15.0],
                20.0,
                24,
                [0.2, 0.8, 0.3, 0.85],
            ));

        // Blue triangle — center
        let tri = orbital::twod::shape::ShapeDescriptor::solid_triangle([0.3, 0.4, 0.9, 0.85]);
        let mut tri_verts = orbital::twod::shape::generate_shape_vertices(&tri, 40.0, 40.0);
        for v in &mut tri_verts {
            v.position[0] += cx - 20.0;
            v.position[1] += top + 5.0;
        }
        self.vertices.extend_from_slice(&tri_verts);

        // Red square — right
        let quad = orbital::twod::shape::ShapeDescriptor::solid_quad([0.9, 0.2, 0.2, 0.85]);
        let mut quad_verts = orbital::twod::shape::generate_shape_vertices(&quad, 40.0, 40.0);
        for v in &mut quad_verts {
            v.position[0] += cx + 25.0;
            v.position[1] += top + 10.0;
        }
        self.vertices.extend_from_slice(&quad_verts);

        self.last_layout_size = Some((screen_w, screen_h));
    }
}

impl LayerRenderer for ShapeOverlay {
    fn layer(&self) -> RenderLayer {
        RenderLayer::World2D
    }

    fn render(&mut self, ctx: RenderOverlayContext) {
        let (screen_w, screen_h) = ctx.screen_size;

        // `WindowSize` starts at (0, 0) until the first `Resized` arrives, and
        // a zero size would produce a NaN projection below.
        if screen_w <= 0.0 || screen_h <= 0.0 {
            return;
        }

        // Vertex positions are baked in absolute pixels, so re-layout whenever
        // the screen size changes (screen rotation, window resize, …).
        if self.last_layout_size != Some((screen_w, screen_h)) {
            self.layout_shapes(screen_w, screen_h);
        }
        if self.vertices.is_empty() {
            return;
        }

        let projection = [
            [2.0 / screen_w, 0.0, 0.0, 0.0],
            [0.0, -2.0 / screen_h, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0, 1.0],
        ];

        let uniform = Camera2DUniform {
            view_proj: projection,
            screen_size: [screen_w, screen_h],
            _padding: [0.0; 2],
        };

        let uniform_bytes = unsafe {
            std::slice::from_raw_parts(
                &uniform as *const Camera2DUniform as *const u8,
                std::mem::size_of::<Camera2DUniform>(),
            )
        };

        ctx.queue
            .write_buffer(&self.camera_buffer, 0, uniform_bytes);

        let Some(command_encoder) = ctx.encoder else {
            return;
        };

        {
            let mut render_pass =
                command_encoder.begin_render_pass(&orbital::wgpu::RenderPassDescriptor {
                    label: Some("2D Overlay Pass"),
                    color_attachments: &[Some(orbital::wgpu::RenderPassColorAttachment {
                        view: ctx.target_view,
                        resolve_target: None,
                        ops: orbital::wgpu::Operations {
                            load: orbital::wgpu::LoadOp::Load,
                            store: orbital::wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });

            let byte_data = unsafe {
                std::slice::from_raw_parts(
                    self.vertices.as_ptr() as *const u8,
                    self.vertices.len() * std::mem::size_of::<Vertex2D>(),
                )
            };
            ctx.queue
                .write_buffer(self.renderer.vertex_buffer(), 0, byte_data);

            render_pass.set_pipeline(self.renderer.pipeline());
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.renderer.vertex_buffer().slice(..));
            render_pass.draw(0..self.vertices.len() as u32, 0..1);
        }
    }
}

// ────────────────────────────────────────────────────────────────────

// Dynamic time-of-day sky. Uses the cheap in-place update path
// (`WorldEnvironment::update_sky_parameters`), so the descriptor is rewritten
// and realized every frame.
const DYNAMIC_SKY_CUBE_SIZE: u32 = 256;
const DYNAMIC_SKY_MIP_LEVELS: u32 = 3;

fn sys_animate_dynamic_sky(
    initial_hours: f32,
) -> impl FnMut(Res<DeltaTime>, ResMut<EnvironmentDescriptorResource>) {
    // The sun moves smoothly enough at 30 Hz; regenerating the full sky at
    // render rate is unnecessary and dominates the dynamic-sky update count.
    const UPDATE_INTERVAL: f32 = 1.0 / 30.0;

    let mut clock = initial_hours;
    let mut accumulator = UPDATE_INTERVAL;

    move |dt: Res<DeltaTime>, mut descriptor: ResMut<EnvironmentDescriptorResource>| {
        // Full day/night cycle every ~2 minutes of real time.
        clock = (clock + dt.0 as f32 / 5.0).rem_euclid(24.0);
        accumulator += dt.0 as f32;
        if accumulator < UPDATE_INTERVAL {
            return;
        }
        accumulator %= UPDATE_INTERVAL;

        descriptor.0 = Some(WorldEnvironmentDescriptor::Generated {
            cube_face_size: DYNAMIC_SKY_CUBE_SIZE,
            sampling_type: SamplingType::GaussianBlur,
            custom_specular_mip_level_count: Some(DYNAMIC_SKY_MIP_LEVELS),
            parameters: Some(GeneratedSkyParameters {
                sun_position: SunPosition::TimeOfDay { hours: clock },
                ..GeneratedSkyParameters::default()
            }),
            dynamic: true,
        });
    }
}

impl Module for ProcgeoSceneModule {
    fn setup(
        &self,
        ecs: &mut World,
        _device: &orbital::wgpu::Device,
        _queue: &orbital::wgpu::Queue,
    ) -> Vec<Box<dyn System>> {
        // Spawn the camera in the central hallway, looking toward the
        // original room row. Turning around reveals the mirrored row.
        let camera = ecs.spawn_entity();
        ecs.attach_component(
            &camera,
            CameraDescriptor {
                label: "Default".into(),
                aspect: 16.0 / 9.0,
                fovy: Rad(std::f32::consts::FRAC_PI_4),
                projection_mode: Default::default(),
                ortho_height: 10.0,
                near: 0.1,
                far: 10000.0,
                global_gamma: 2.2,
            },
        )
        .unwrap();
        // Rotate +90° around Y so forward (+X) faces -Z (toward rooms)
        let rot = Quaternion::new(
            std::f32::consts::FRAC_1_SQRT_2,
            0.0,
            std::f32::consts::FRAC_1_SQRT_2,
            0.0,
        );
        // Start just behind the hallway centre so the centred sign below stays
        // in front of the camera instead of sharing its near plane.
        ecs.attach_component(
            &camera,
            Position(Point3::new(0.0, 7.0, HALLWAY_MIRROR_Z + 2.0)),
        )
        .unwrap();
        ecs.attach_component(&camera, Rotation(rot)).unwrap();
        ecs.insert_resource(ActiveCamera(camera));
        ecs.insert_resource(CursorGrabConfig(true));
        ecs.insert_resource(CursorGrabState(true));

        // Low-cost volumetrics showcase. The room spotlights and point lights
        // inject visible fog scattering while keeping the froxel grid at the
        // Low tier for the already busy all-in-one scene.
        ecs.insert_resource(VolumetricSettings {
            enabled: true,
            density: 0.0012,
            scattering: 1.0,
            extinction: 0.85,
            anisotropy: 0.35,
            height_base: 0.0,
            height_falloff: 0.25,
            height_density: 0.0035,
            ambient_color: [0.02, 0.03, 0.05],
            environment_intensity: 0.12,
            temporal_history: true,
            // Keep the indoor grid concentrated on the hallway and rooms;
            // a shorter range also reduces depth-slice flicker at Low quality.
            max_distance: 30.0,
            // The mobile froxel grid keeps the all-in-one scene usable on
            // Adreno and Mali while retaining the same volumetric look.
            quality: if cfg!(target_os = "android") {
                VolumetricQuality::Mobile
            } else {
                VolumetricQuality::Low
            },
        });

        // 2D shape overlay (draws on top of 3D scene; layout is regenerated
        // whenever the screen size changes, e.g. on screen rotation)
        // NOTE: ShapeOverlay is registered via register_overlays() below.

        // Dynamic procedural sky (in-place updates, cheap per frame).
        ecs.insert_resource(EnvironmentDescriptorResource(Some(
            WorldEnvironmentDescriptor::Generated {
                cube_face_size: DYNAMIC_SKY_CUBE_SIZE,
                sampling_type: SamplingType::GaussianBlur,
                custom_specular_mip_level_count: Some(DYNAMIC_SKY_MIP_LEVELS),
                parameters: Some(GeneratedSkyParameters::default()),
                dynamic: true,
            },
        )));

        // Load scene from RON file
        let scene = match SceneBuilder::load("Scenes/procgeo_demo.ron") {
            Ok(s) => s,
            Err(e) => {
                let asset_root = std::env::current_dir().unwrap_or_default().join("Assets");
                error!(
                    "Failed to load scene: {e} (assets are resolved from {})",
                    asset_root.display()
                );
                return vec![
                    sys_camera_controller(),
                    sys_ortho_dolly(),
                    Box::new(CursorToggle::new(true)),
                ];
            }
        };

        // Build and spawn the authored first row (R1-R5) and the two populated
        // second-row rooms (R6-R7). Only the empty shells are mirrored, so the
        // second row does not duplicate R1-R5's contents. R8-R10 end up as
        // empty shells, ready for future exhibits.
        let built = scene.build();
        for (descriptor, (mesh, material, transform)) in scene.entities.iter().zip(built) {
            let label = descriptor.label.as_deref().unwrap_or("procgeo");
            spawn_scene_model(ecs, label, mesh.clone(), material.clone(), transform);

            if is_room_shell(label) {
                let mut mirrored = transform;
                mirrored.position.z = 2.0 * HALLWAY_MIRROR_Z - mirrored.position.z;
                let room = second_row_room_name(transform.position.x);
                let kind = room_shell_kind(label);
                spawn_scene_model(ecs, &format!("{room} {kind}"), mesh, material, mirrored);
            }
        }

        // Custom material shader from the standalone example. R8 is one of the
        // empty mirrored shells, so it gets a dedicated exhibit in the middle
        // of the second row.
        let custom_material = Arc::new(MaterialShaderDescriptor {
            name: Some("AiO Custom RGB Cube".into()),
            nodes: &[
                "camera_uniform",
                "vertex_data_complex",
                "instance_data",
                "fragment_data",
            ],
            raw_source: Some(include_str!("custom_shader.wgsl").into()),
            vertex_stage_layouts: Some(vec![
                VertexStageLayout::ComplexVertexData,
                VertexStageLayout::InstanceData,
            ]),
            cull_mode: None,
            cast_shadows: false,
            receive_shadows: false,
            ..MaterialShaderDescriptor::default()
        });
        spawn_scene_model(
            ecs,
            "Custom Shader (R8)",
            box_(Vector3::new(2.4, 2.4, 2.4)),
            Arc::clone(&custom_material),
            Transform::new(
                Vector3::new(0.0, 1.5, HALLWAY_MIRROR_Z + 10.0),
                Quaternion::from_axis_angle(Vector3::unit_y(), Rad(0.55)),
                Vector3::new(1.0, 1.0, 1.0),
            ),
        );

        info!("Scene loaded with {} entities", scene.entities.len());

        // Floating SDF labels for both room rows.
        //
        // Building the shared atlas is CPU-heavy enough to trip Android's ANR
        // watchdog on the main thread, so prepare the atlas and label meshes on
        // a worker and attach them once the app is already presenting frames.
        let (label_sender, label_receiver) = mpsc::channel();
        let async_room_labels: Option<Box<dyn System>> = match std::thread::Builder::new()
            .name("room-label-atlas".into())
            .spawn(move || {
                let started = std::time::Instant::now();
                let labels = build_room_labels();
                info!(
                    "Built {} room labels in {:?} on the room-label worker",
                    labels.len(),
                    started.elapsed()
                );
                let _ = label_sender.send(labels);
            }) {
            Ok(_) => Some(Box::new(AsyncRoomLabels::new(label_receiver))),
            Err(e) => {
                error!("Failed to start the room-label worker: {e}");
                None
            }
        };

        // Import the DamagedHelmet into Room 2
        if let Some(mut queue) = ecs.get_resource_mut::<ImportQueueResource>() {
            queue.push(ImportTask::Gltf {
                file_path: "Models/DamagedHelmet.glb".into(),
                task: GltfImport::WholeFile,
            });
            info!("Queued DamagedHelmet import");
        }

        // ════════════════════════════════════════════════════════════
        // R3 — moving spot light over the small object cluster.
        // Keep the light inside the room so the inverse-square falloff
        // still reaches the floor and back wall. The old hallway position
        // put the objects at the edge of the default 10 unit range, which
        // made the light and its shadows effectively invisible.
        // ════════════════════════════════════════════════════════════
        let spot_light = ecs.spawn_entity();
        let spot_pos = Point3::new(0.0, 7.0, 2.0);
        let target = Point3::new(0.0, 0.0, -2.0);
        let dir = (target - spot_pos).normalize();
        ecs.attach_component(
            &spot_light,
            LightDescriptor::new_spot(
                Vector3::new(1.0, 1.0, 1.0),
                150.0,
                Vector3::new(dir.x, dir.y, dir.z),
                0.1,
                0.44, // inner/outer ~6°/~25°
            )
            .with_range(16.0),
        )
        .unwrap();
        ecs.attach_component(&spot_light, Position(spot_pos))
            .unwrap();
        ecs.attach_component(&spot_light, LightDirty(true)).unwrap();
        ecs.attach_component(
            &spot_light,
            ShadowCaster {
                cascade_count: 0,
                bias: 0.0002,
                ..Default::default()
            },
        )
        .unwrap();

        // ── R1/R2 — one central omni light per material gallery ─────
        for room_x in [-20.0, -10.0f32] {
            let pl = ecs.spawn_entity();
            ecs.attach_component(
                &pl,
                LightDescriptor::new_point(Vector3::new(1.0, 1.0, 1.0), 50.0),
            )
            .unwrap();
            ecs.attach_component(&pl, Position(Point3::new(room_x, 5.0, 0.0)))
                .unwrap();
            ecs.attach_component(&pl, LightDirty(true)).unwrap();
            ecs.attach_component(
                &pl,
                ShadowCaster {
                    bias: 0.0002,
                    ..Default::default()
                },
            )
            .unwrap();
        }

        // ── R4 — static spot ring around the helmet ─────────────────
        let helmet_10 = Point3::new(10.0, 2.2, 0.0);
        for angle in [0.0, 2.1, 4.2f32] {
            let (s, c) = angle.sin_cos();
            let lpos = Point3::new(helmet_10.x + c * 3.0, 4.0, helmet_10.z + s * 3.0);
            let d = (helmet_10 - lpos).normalize();
            let hl = ecs.spawn_entity();
            ecs.attach_component(
                &hl,
                LightDescriptor::new_spot(
                    Vector3::new(1.0, 0.9, 0.8),
                    30.0,
                    Vector3::new(d.x, d.y, d.z),
                    0.1,
                    0.44,
                ),
            )
            .unwrap();
            ecs.attach_component(&hl, Position(lpos)).unwrap();
            ecs.attach_component(&hl, LightDirty(true)).unwrap();
            ecs.attach_component(
                &hl,
                ShadowCaster {
                    cascade_count: 0,
                    bias: 0.0002,
                    ..Default::default()
                },
            )
            .unwrap();
        }

        // ── R5 — colourful omni ring around the second helmet ───────
        let helmet_20 = Point3::new(20.0, 2.2, 0.0);
        let colors: [Vector3<_>; 8] = [
            Vector3::new(1.0, 0.2, 0.2),
            Vector3::new(1.0, 0.6, 0.1),
            Vector3::new(1.0, 1.0, 0.2),
            Vector3::new(0.2, 1.0, 0.2),
            Vector3::new(0.2, 0.6, 1.0),
            Vector3::new(0.4, 0.2, 1.0),
            Vector3::new(1.0, 0.2, 1.0),
            Vector3::new(1.0, 0.9, 0.8),
        ];
        for (i, color) in colors.into_iter().enumerate() {
            let angle = std::f32::consts::TAU * i as f32 / 8.0;
            let (s, c) = angle.sin_cos();
            let lpos = Point3::new(helmet_20.x + c * 3.0, 4.0, helmet_20.z + s * 3.0);
            let pl = ecs.spawn_entity();
            ecs.attach_component(&pl, LightDescriptor::new_point(color, 30.0))
                .unwrap();
            ecs.attach_component(&pl, Position(lpos)).unwrap();
            ecs.attach_component(&pl, LightDirty(true)).unwrap();
        }

        // ── R6 — soft omni light behind the blended panes ───────────
        let r6_light = ecs.spawn_entity();
        ecs.attach_component(
            &r6_light,
            LightDescriptor::new_point(Vector3::new(1.0, 0.95, 0.9), 40.0),
        )
        .unwrap();
        ecs.attach_component(&r6_light, Position(Point3::new(-20.0, 4.0, 20.0)))
            .unwrap();
        ecs.attach_component(&r6_light, LightDirty(true)).unwrap();
        ecs.attach_component(
            &r6_light,
            ShadowCaster {
                bias: 0.0002,
                ..Default::default()
            },
        )
        .unwrap();

        // ── R6 — hallway-side fill so the blended panes stay legible
        // when viewed from the room entrance. The original light sits
        // behind the panes; without a front light their faces go dark.
        let r6_front = ecs.spawn_entity();
        ecs.attach_component(
            &r6_front,
            LightDescriptor::new_point(Vector3::new(1.0, 0.95, 0.9), 30.0),
        )
        .unwrap();
        ecs.attach_component(&r6_front, Position(Point3::new(-20.0, 3.5, 16.2)))
            .unwrap();
        ecs.attach_component(&r6_front, LightDirty(true)).unwrap();

        // ── R8 — soft omni light over the custom shader exhibit ─────
        let r8_light = ecs.spawn_entity();
        ecs.attach_component(
            &r8_light,
            LightDescriptor::new_point(Vector3::new(0.8, 0.9, 1.0), 40.0),
        )
        .unwrap();
        ecs.attach_component(&r8_light, Position(Point3::new(0.0, 5.5, 18.0)))
            .unwrap();
        ecs.attach_component(&r8_light, LightDirty(true)).unwrap();

        // ── R7 — shadow-casting spot across the AA staircase ────────
        let r7_spot = ecs.spawn_entity();
        let r7_pos = Point3::new(-10.0, 8.0, 16.0);
        let r7_target = Point3::new(-10.0, 0.5, 22.0);
        let r7_dir = (r7_target - r7_pos).normalize();
        ecs.attach_component(
            &r7_spot,
            LightDescriptor::new_spot(
                Vector3::new(1.0, 1.0, 1.0),
                60.0,
                Vector3::new(r7_dir.x, r7_dir.y, r7_dir.z),
                0.1,
                0.44,
            ),
        )
        .unwrap();
        ecs.attach_component(&r7_spot, Position(r7_pos)).unwrap();
        ecs.attach_component(&r7_spot, LightDirty(true)).unwrap();
        ecs.attach_component(
            &r7_spot,
            ShadowCaster {
                cascade_count: 0,
                bias: 0.0002,
                ..Default::default()
            },
        )
        .unwrap();

        // A slow vertical sweep keeps the spot inside the room and makes the
        // moving object shadows easy to read.
        let moving_spot = LightAnimator::new(spot_light, 0.0, 2.0);

        // Low-intensity directional fill light for scene-wide diffuse lighting
        let fill = ecs.spawn_entity();
        ecs.attach_component(
            &fill,
            LightDescriptor::new_directional(
                Vector3::new(0.0, -0.6, -0.8), // emission direction (shader negates -> light from above)
                Vector3::new(1.0, 0.88, 0.65), // subtle warm white
                0.25,
            ),
        )
        .unwrap();
        ecs.attach_component(&fill, Position(Point3::new(0.0, 0.0, 0.0)))
            .unwrap();
        ecs.attach_component(&fill, LightDirty(true)).unwrap();

        let mut systems = vec![
            sys_camera_controller(),
            sys_ortho_dolly(),
            Box::new(CursorToggle::new(true)),
            Box::new(HelmetAdjuster::new()),
            Box::new(moving_spot),
            sys_animate_dynamic_sky(14.0).into_system(),
        ];
        if let Some(async_room_labels) = async_room_labels {
            systems.push(async_room_labels);
        }
        systems
    }

    fn register_overlays(
        &self,
        ecs: &mut World,
        layer_renderers: &mut Vec<Box<dyn LayerRenderer>>,
    ) {
        let format = ecs
            .get_resource::<orbital::ecs_bridge::SurfaceFormatResource>()
            .map(|f| f.0)
            .unwrap_or(orbital::wgpu::TextureFormat::Bgra8UnormSrgb);

        let device = match ecs.get_resource::<orbital::ecs_bridge::DeviceResource>() {
            Some(d) => d,
            None => {
                orbital::logging::warn!(
                    "ProcgeoSceneModule: no DeviceResource, skipping overlay registration"
                );
                return;
            }
        };

        let overlay = ShapeOverlay::new(&device.0, format);
        layer_renderers.push(Box::new(overlay));
    }
}
