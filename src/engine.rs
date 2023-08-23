use std::sync::Arc;

use wgpu::{
    Adapter, Backend, Backends, CompositeAlphaMode, Device, DeviceDescriptor, Features, Instance,
    InstanceDescriptor, Limits, PresentMode, Queue, Surface, SurfaceConfiguration,     TextureUsages,
};

use crate::app::app_window::AppWindow;

pub struct Engine {
    window: Arc<AppWindow>,
    surface: Surface,
    adapter: Adapter,
    device: Device,
    queue: Queue,
}

impl Engine {
    /// Initializes the [Engine].
    /// Creates a bunch of critical internal components while doing so.
    pub async fn initialize(window: Arc<AppWindow>) -> Self {
        let instance = Engine::make_instance().await;
        log::debug!("{instance:?}");

        let surface = Engine::make_surface(&instance, window.clone()).await;
        log::debug!("{surface:?}");

        let adapter = Engine::make_adapter(&instance, &surface).await;
        log::debug!("{adapter:?}");

        let (device, queue) = Engine::make_device(&adapter).await;
        log::debug!("{device:?}");
        log::debug!("{queue:?}");

        Self {
            window,
            surface,
            adapter,
            device,
            queue,
        }
    }

    /// Configures the local [Surface].
    pub fn configure_surface(&self) {
        let surface_caps = self.surface.get_capabilities(&self.adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: self.window.get_window().inner_size().width,
            height: self.window.get_window().inner_size().height,
            present_mode: PresentMode::AutoVsync,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![],
        };

        self.surface.configure(&self.device, &surface_config)
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
    pub async fn rank_adapters(instance: &Instance, surface: &Surface) -> Vec<(Adapter, i32)> {
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
    async fn make_adapter(instance: &Instance, surface: &Surface) -> Adapter {
        let mut adapters = Self::rank_adapters(instance, surface).await;

        log::debug!("The following adapters are compatible:");
        let mut i = 0;
        for x in &adapters {
            log::debug!("#{}, Score: {} - {:?}", i, x.1, x.0.get_info());
            i += 1;
        }

        // Pick the last adapter.
        // After scoring and sorting, the higest score should be the
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
    async fn make_surface(instance: &Instance, window: Arc<AppWindow>) -> Surface {
        unsafe { instance.create_surface(&window.get_window()) }
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
    async fn make_instance() -> Instance {
        Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            dx12_shader_compiler: Default::default(),
        })
    }

    /// Returns a new [`Arc<AppWindow>`].
    pub fn get_window(&self) -> Arc<AppWindow> {
        self.window.clone()
    }

    /// Returns the local [`&Surface`].
    pub fn get_surface(&self) -> &Surface {
        &self.surface
    }

    /// Returns the local [`&Adapter`].
    pub fn get_adapter(&self) -> &Adapter {
        &self.adapter
    }

    /// Returns the local [`&Device`].
    pub fn get_device(&self) -> &Device {
        &self.device
    }

    /// Returns the local [`&Queue`].
    pub fn get_queue(&self) -> &Queue {
        &self.queue
    }
}
