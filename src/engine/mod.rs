use std::{ops::Range, sync::Arc};

use cgmath::{Deg, InnerSpace, Quaternion, Rotation3, Vector3, Zero};
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    Adapter, Backend, Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendState,
    Buffer, BufferUsages, ColorTargetState, ColorWrites, CompareFunction, CompositeAlphaMode,
    DepthBiasState, DepthStencilState, Device, DeviceDescriptor, Face, Features, FragmentState,
    FrontFace, Instance as WInstance, InstanceDescriptor, Limits, MultisampleState,
    PipelineLayoutDescriptor, PolygonMode, PresentMode, PrimitiveState, PrimitiveTopology, Queue,
    RenderPipeline, RenderPipelineDescriptor, SamplerBindingType, ShaderModule, ShaderStages,
    StencilState, Surface, SurfaceConfiguration, TextureFormat, TextureSampleType, TextureUsages,
    TextureViewDimension, VertexState,
};
use winit::window::Window;

use crate::Camera;

use self::texture::Texture;

pub mod texture;
pub use texture::*;

pub mod vertex;
pub use vertex::*;

pub mod instance;
pub use instance::*;

pub mod model;
pub use model::*;

const INSTANCES_ROWS: u32 = 10;
const INSTANCES_COLUMNS: u32 = 10;

pub struct Engine {
    window: Arc<Window>,
    surface: Arc<Surface>,
    surface_config: Option<SurfaceConfiguration>,
    surface_texture_format: Option<TextureFormat>,
    instance: Arc<WInstance>,
    adapter: Arc<Adapter>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    camera: Camera,
    render_pipeline: Option<RenderPipeline>,
    diffuse_bind_group: Option<BindGroup>,
    default_texture_bind_group_layout: Option<BindGroupLayout>,
    diffuse_texture: Option<Texture>,
    instances: Option<Vec<Instance>>,
    instance_buffer: Option<Buffer>,
    depth_texture: Option<Texture>,
}

impl Engine {
    /// Initializes the [Engine].
    /// Creates a bunch of critical internal components while doing so.
    pub async fn initialize(window: Arc<Window>) -> Self {
        let instance = Engine::make_instance().await;
        let instance_arc = Arc::new(instance);
        log::debug!("{instance_arc:?}");

        let surface = Engine::make_surface(&instance_arc, window.clone()).await;
        let surface_arc = Arc::new(surface);
        log::debug!("{surface_arc:?}");

        let adapter = Engine::make_adapter(&instance_arc, &surface_arc).await;
        let adapter_arc = Arc::new(adapter);
        log::debug!("{adapter_arc:?}");

        let (device, queue) = Engine::make_device(&adapter_arc).await;
        let device_arc = Arc::new(device);
        let queue_arc = Arc::new(queue);
        log::debug!("{device_arc:?}");
        log::debug!("{queue_arc:?}");

        let camera = Camera::new(device_arc.clone(), queue_arc.clone());

        Self {
            window,
            surface: surface_arc,
            surface_config: None,
            surface_texture_format: None,
            instance: instance_arc,
            adapter: adapter_arc,
            device: device_arc,
            queue: queue_arc,
            camera,
            render_pipeline: None,
            diffuse_bind_group: None,
            default_texture_bind_group_layout: None,
            diffuse_texture: None,
            instances: None,
            instance_buffer: None,
            depth_texture: None,
        }
    }

    pub fn configure(&mut self) {
        self.configure_surface();

        self.textures();

        self.instances();

        if self.render_pipeline.is_none() {
            let render_pipeline = self.make_render_pipeline();
            log::debug!("{render_pipeline:?}");
            self.render_pipeline = Some(render_pipeline);
        }
    }

    pub fn instances(&mut self) {
        if self.instances.is_none() {
            let instances = (0..INSTANCES_ROWS)
                .flat_map(|x| {
                    (0..INSTANCES_COLUMNS).map(move |z| {
                        let position = Vector3::new(x as f32 * 2.0, 0.0, z as f32 * 2.0);

                        let rotation = if position.is_zero() {
                            Quaternion::from_axis_angle(Vector3::unit_z(), Deg(0.0))
                        } else {
                            Quaternion::from_axis_angle(position.normalize(), Deg(45.0))
                        };

                        Instance::new(position, rotation)
                    })
                })
                .collect::<Vec<_>>();
            self.instances = Some(instances);
        }

        let instance_data = self
            .instances
            .as_ref()
            .unwrap()
            .iter()
            .map(Instance::to_uniform)
            .collect::<Vec<_>>();
        let instance_buffer = self.get_device().create_buffer_init(&BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: BufferUsages::VERTEX,
        });
        self.instance_buffer = Some(instance_buffer);
    }

    pub fn textures(&mut self) {
        self.diffuse_textures();
        self.depth_textures();
    }

    pub fn depth_textures(&mut self) {
        let depth_texture = Texture::make_depth_texture(
            &self.device,
            self.get_surface_config(),
            Some("Depth Texture"),
        );
        self.depth_texture = Some(depth_texture);
    }

    pub fn diffuse_textures(&mut self) {
        let diffuse_bytes = include_bytes!("../../res/test.png");
        let diffuse_texture =
            Texture::from_bytes(&self.device, &self.queue, diffuse_bytes, "Test Texture").unwrap();

        let texture_bind_group_layout =
            self.device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("Texture Bind Group Layout"),
                    entries: &[
                        BindGroupLayoutEntry {
                            binding: 0,
                            visibility: ShaderStages::FRAGMENT,
                            ty: BindingType::Texture {
                                sample_type: TextureSampleType::Float { filterable: true },
                                view_dimension: TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 1,
                            visibility: ShaderStages::FRAGMENT,
                            ty: BindingType::Sampler(SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });

        let diffuse_bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Diffuse Bind Group"),
            layout: &texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(diffuse_texture.get_view()),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(diffuse_texture.get_sampler()),
                },
            ],
        });

        self.diffuse_texture = Some(diffuse_texture);
        self.diffuse_bind_group = Some(diffuse_bind_group);
        self.default_texture_bind_group_layout = Some(texture_bind_group_layout);
    }

    /// Configures the local [Surface].
    pub fn configure_surface(&mut self) {
        let surface_caps = self.surface.get_capabilities(&self.adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        self.surface_texture_format = Some(surface_format.clone());

        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: self.window.inner_size().width,
            height: self.window.inner_size().height,
            present_mode: PresentMode::AutoVsync,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![],
        };

        self.surface.configure(&self.device, &surface_config);
        self.surface_config = Some(surface_config);
    }

    /// Creates a new [Device] and, as a by-product a [Queue] of that [Device].
    async fn make_device(adapter: &Adapter) -> (Device, Queue) {
        adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    features: Features::empty(),
                    limits: Limits::default(),
                },
                None,
            )
            .await
            .expect("failed requesting device")
    }

    /// Sorts through all available adapters and assigns a score
    /// to each adapter.
    ///
    /// # Score calculation
    ///
    /// Higher score = Better [Adapter]  
    /// Lower score = Worse [Adapter]  
    ///
    /// Adapters reaching a score of < 0 should be avoided as it's in
    /// most cases a combination of unsupported hardware (either by the
    /// OS, or, by drivers, or, by WGPU) or possibly even malfunctioning
    /// broken hardware.
    ///
    /// ## Surface
    ///
    /// If the adapter is compatible with the given surface a
    /// score is added.  
    /// If hte adapter is not compatible [i32::MIN] will be set as score.
    ///
    /// ## Adapter Types
    ///
    /// Adapter types are preferred in the following order:
    ///
    /// - Discrete GPU
    /// - Integrated GPU
    /// - Virtual GPU
    /// - CPU
    /// - Other
    ///
    /// A **Discrete GPU** (dGPU) is a separate GPU commonly connected via PCIe
    /// or USB-C.  
    /// These graphics cards are usually made for rendering or processing
    /// of huge data and thus should be our first choice if available.
    ///
    /// A **Integrated GPU** (iGPU) is commonly integrated into another chip.  
    /// These graphics cards are usually only equipped with the bare
    /// minimum to render e.g. your operating system with Office software.
    /// Commonly, those GPUs are integrated within a CPU.  
    /// **However**, some systems like consoles commonly use an APU design
    /// for their CPU & GPU.
    /// An APU **combines** the CPU and GPU on a chip, but doesn't
    /// sacrifice GPU performance _usually_.  
    /// Still, if available, a dGPU should be preferred.
    ///
    /// A **Virtual GPU** (vGPU) is commonly found in virtual machines.  
    /// It's an emulated or simulated GPU either for testing purposes
    /// or actual high-capacity use cases.  
    /// vGPU's don't have to be bad **if done correctly**, but a dedicated
    /// "real" GPU or even an integrated one usually are more performant.
    ///
    /// Alternatively, we could also use the **CPU** to render things on
    /// the screen.  
    /// This is terribly slow, there is a reason why GPUs exist ...
    /// It would work, for example if we want to do some unit testing for
    /// our application or similar use cases, but it would be terribly slow
    /// and may not even reach more than a handful of frames per second.
    ///
    /// Lastly, WGPU will categorize everything else into
    /// the **Other** category.  
    /// There is a small chance that a given architecture
    /// (like APUs on consoles) aren't recognized as **iGPUs** (see above)
    /// and instead go into this category.
    /// However, in most cases this will be some either broken device,
    /// combination of hardware or unsupported, possibly outdated, drivers.
    /// Which... should be avoided if possible.
    ///
    /// ## Backends
    ///
    /// Backends are preferred in the following order:
    ///
    /// - DX12 (Exclusively Windows & Xbox)
    /// - Metal (Exclusively macOS & iOS)
    /// - Vulkan (Universal)
    /// - DX11 (Windows Fallback)
    /// - Browser WGPU (Web-Browser)
    /// - OpenGL [WebGL] (Universal Fallback)
    /// - Empty/None (Failure ...)
    ///
    /// This ensures that on every operating system the right backends
    /// are preferred. E.g.:  
    /// On Windows we have DX12, Vulkan and DX11 available (as well as
    /// OpenGL). DX12 _should_ work better than Vulkan on Windows, furthermore both DX12 **and** Vulkan should perform better than DX11.  
    ///
    /// On macOS we only have Metal available (and OpenGL on
    /// legacy versions), all other backends shouldn't even show up
    /// as available.
    ///
    /// On Linux we only have Vulkan (and OpenGL) support.
    /// DX11 _can_ be done via [DXVK](https://github.com/doitsujin/dxvk)
    /// through WINE _in theory_, but should be avoided unless you want
    /// to loose performance.
    ///
    /// A console like the Xbox would support DX12 and possibly Vulkan,
    /// but not an outdated backend like DX11.  
    /// Meanwhile a Switch only supports Vulkan (and their own library).
    ///
    /// > Special Case: Empty/None  
    /// > In rare cases it can happen that WGPU is unable to find any
    /// > backend for the current platform _or an unknown backend is
    /// > presented like Nintendo's or Sony's graphics layers?_.  
    /// > In these cases we can't really use WGPU unfortunately (yet?) and
    /// > a negative score will be returned to put that [Adapter] at the
    /// > lowest possible score.
    pub async fn rank_adapters(instance: &WInstance, surface: &Surface) -> Vec<(Adapter, i32)> {
        let mut adapters: Vec<(Adapter, i32)> = instance
            .enumerate_adapters(Backends::all())
            .map(|x| {
                fn score_type(adapter: &Adapter) -> i32 {
                    match adapter.get_info().device_type {
                        wgpu::DeviceType::DiscreteGpu => 5000,
                        wgpu::DeviceType::IntegratedGpu => 2500,
                        wgpu::DeviceType::VirtualGpu => 1000,
                        wgpu::DeviceType::Cpu => 0,
                        wgpu::DeviceType::Other => i32::MIN,
                    }
                }

                fn score_backend(adapter: &Adapter) -> i32 {
                    match adapter.get_info().backend {
                        // Supported and preferred on Windows & Xbox
                        Backend::Dx12 => 100,
                        // Supported and preferred on macOS
                        Backend::Metal => 100,
                        // Universally supported, acting as a "modern fallback"
                        Backend::Vulkan => 50,
                        // Supported on Windows, acting as a "windows fallback"
                        Backend::Dx11 => 25,
                        // Supported only inside Browsers where no other
                        // option is present
                        Backend::BrowserWebGpu => 100,
                        // Old universal backend, acting as a last-resort
                        Backend::Gl => 0,
                        Backend::Empty => i32::MIN, // never hit, see above
                    }
                }

                fn score_surface_compatibility(adapter: &Adapter, surface: &Surface) -> i32 {
                    if adapter.is_surface_supported(surface) {
                        1000
                    } else {
                        i32::MIN
                    }
                }

                let score =
                    score_type(&x) + score_backend(&x) + score_surface_compatibility(&x, surface);

                (x, score)
            })
            .collect();
        adapters.sort_by_cached_key(|x| x.1);
        adapters
    }

    /// Returns a [`Adapter`].  
    /// If inside [`AppConfig`] a preferred adapter is set and it can be
    /// found, that adapter will be returned.  
    /// If the [`Adapter`] cannot be found, each available [`Adapter`]
    /// will be ranked based on a score in [`Self::rank_adapters`].  
    /// The [`Adapter`] with the highest scoring will be returned
    /// in this case.
    async fn make_adapter(instance: &WInstance, surface: &Surface) -> Adapter {
        let mut adapters = Self::rank_adapters(instance, surface).await;

        log::debug!("The following adapters are compatible:");
        let mut i = 0;
        for x in &adapters {
            log::debug!("#{}, Score: {} - {:?}", i, x.1, x.0.get_info());
            i += 1;
        }

        // TODO: Check app config for preferred adapter

        // Pick the last adapter.
        // After scoring and sorting, the highest score should be the
        // best option
        let (chosen_adapter, chosen_score) = adapters.pop().expect("no adapters found");
        log::info!(
            "Selected Adapter '{:?}' with a score of {}",
            chosen_adapter.get_info(),
            chosen_score
        );

        chosen_adapter
    }

    /// Creates a new [`Surface`].
    async fn make_surface(instance: &WInstance, window: Arc<Window>) -> Surface {
        unsafe { instance.create_surface(window.as_ref()) }
            .expect("failed creating surface from window")
    }

    /// Creates an [`Instance`] given the graphics library.
    /// The [`Instance`] will automatically pick which graphics
    /// backend will be used.
    /// Currently supported are:
    /// - Vulkan
    /// - Metal
    /// - DX12
    /// - DX11
    /// - OpenGL (and WebGL)
    /// - WebGPU (virtual GPU used in Browsers)
    async fn make_instance() -> WInstance {
        WInstance::new(InstanceDescriptor {
            backends: Backends::all(),
            dx12_shader_compiler: Default::default(),
        })
    }

    /// Returns the local [`Arc<WInstance>`]
    pub fn get_instance(&self) -> Arc<WInstance> {
        self.instance.clone()
    }

    /// Returns the local [`Arc<Surface>`]
    pub fn get_surface(&self) -> Arc<Surface> {
        self.surface.clone()
    }

    /// Returns the local [`Arc<Adapter>`].
    pub fn get_adapter(&self) -> Arc<Adapter> {
        self.adapter.clone()
    }

    /// Returns the local [`Arc<Device>`].
    pub fn get_device(&self) -> Arc<Device> {
        self.device.clone()
    }

    /// Returns the local [`Arc<Queue>`].
    pub fn get_queue(&self) -> Arc<Queue> {
        self.queue.clone()
    }

    pub fn get_render_pipeline(&self) -> &RenderPipeline {
        self.render_pipeline
            .as_ref()
            .expect("Engine::get_render_pipeline called before Engine::configure!")
    }

    fn make_shader(device: &Device) -> ShaderModule {
        device.create_shader_module(include_wgsl!("../shaders/main.wgsl"))
    }

    fn make_render_pipeline(&mut self) -> RenderPipeline {
        let main_shader = Self::make_shader(&self.device);

        let render_pipeline_layout =
            self.device
                .create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[
                        self.default_texture_bind_group_layout
                            .as_ref()
                            .expect("texture_bind_group_layout used before Engine::configure"),
                        &self.get_camera().get_bind_group_layout(),
                    ],
                    push_constant_ranges: &[],
                });

        let render_pipeline = self
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                // Vertex shader
                vertex: VertexState {
                    module: &main_shader,
                    entry_point: "vs_main",
                    // Vertex buffers
                    buffers: &[ModelVertex::descriptor(), InstanceUniform::descriptor()],
                },
                // Fragment shader
                fragment: Some(FragmentState {
                    module: &main_shader,
                    entry_point: "fs_main",
                    // Store the resulting colours in a format
                    // that is equal to the surface format
                    targets: &[Some(ColorTargetState {
                        // Match the surface format
                        format: self.surface_texture_format.unwrap(),
                        // Replace pixels
                        blend: Some(BlendState::REPLACE),
                        // Use all colour channels
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                // How to interpret the vertices
                primitive: PrimitiveState {
                    // Every three vertices form a triangle
                    topology: PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    // A given triangle is is facing "forward" if it's arranged counter-clockwise
                    front_face: FrontFace::Ccw,
                    // Cull the triangle if it's the backside
                    cull_mode: Some(Face::Back),
                    // Fill the triangle
                    // Note: requires Features::NON_FILL_POLYGON_MODE if not Fill
                    polygon_mode: PolygonMode::Fill,
                    // Note: requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Note: requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: Some(DepthStencilState {
                    format: Texture::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: CompareFunction::Less,
                    stencil: StencilState::default(),
                    bias: DepthBiasState::default(),
                }),
                multisample: MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });

        return render_pipeline;
    }

    pub fn get_diffuse_group(&self) -> &BindGroup {
        self.diffuse_bind_group
            .as_ref()
            .expect("Engine::get_diffuse_group called before Engine::configure!")
    }

    pub fn get_camera(&self) -> &Camera {
        &self.camera
    }

    pub fn get_camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    pub fn get_backend_name(&self) -> String {
        let mut raw_chars = self.get_adapter().get_info().backend.to_str().chars();
        raw_chars.next().unwrap().to_uppercase().collect::<String>() + raw_chars.as_str()
    }

    pub fn get_instance_buffer(&self) -> &Buffer {
        &self
            .instance_buffer
            .as_ref()
            .expect("Called Engine::get_instance_buffer before Engine::configure")
    }

    pub(crate) fn get_instance_count(&self) -> Range<u32> {
        let instance_count = self
            .instances
            .as_ref()
            .expect("Called Engine::get_instance_count before Engine::configure")
            .len() as u32;

        0..instance_count
    }

    fn get_surface_config(&self) -> &SurfaceConfiguration {
        &self
            .surface_config
            .as_ref()
            .expect("Called Engine::get_surface_config before Engine::configure")
    }

    pub fn get_depth_texture(&self) -> &Texture {
        self.depth_texture
            .as_ref()
            .expect("Called Engine::get_depth_texture before Engine::configure")
    }

    pub(crate) fn get_default_texture_layout(&self) -> &BindGroupLayout {
        &self
            .default_texture_bind_group_layout
            .as_ref()
            .expect("No default texture layout set")
    }
}
