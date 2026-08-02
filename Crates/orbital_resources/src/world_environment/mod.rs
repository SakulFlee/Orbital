//! World environment handling IBL (Image-Based Lighting) textures.
//!
//! `WorldEnvironment` is a GPU-heavy singleton that can be stored as an ECS
//! resource (`world.insert_resource(...)`) rather than as a per-entity
//! component, since there is typically only one active environment.

use cgmath::Vector2;
use image::{GenericImageView, ImageReader};
use log::{debug, info, warn};
use std::error::Error;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    path::PathBuf,
};
use wgpu::MipmapFilterMode;
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType,
    BufferBindingType, BufferUsages, CompareFunction, ComputePassDescriptor, ComputePipeline,
    ComputePipelineDescriptor, Device, Extent3d, FilterMode as WFilterMode,
    PipelineLayoutDescriptor, Queue, SamplerBindingType, SamplerDescriptor, ShaderModuleDescriptor,
    ShaderStages, StorageTextureAccess, TextureDimension, TextureFormat, TextureSampleType,
    TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension, include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{FilterMode, MaterialShader, Texture, TextureSize};

mod error;
pub use error::*;

mod cache_file;
pub use cache_file::*;

mod skybox_type;
pub use skybox_type::*;

mod sampling_type;
pub use sampling_type::*;

mod descriptor;
pub use descriptor::*;

use super::{MaterialShaderDescriptor, ShaderSource, TextureDescriptor};

/// Maximum total time to spend polling for GPU work completion.
const GPU_POLL_TIMEOUT: Duration = Duration::from_secs(120);

/// Tile edge length (texels) used to split cubemap faces into small enough
/// dispatches to stay under the DX12 TDR timeout (~2 s).
/// 128 divides 1024 evenly, giving 8×8 = 64 tiles per face.
const TILE_SIZE: u32 = 128;
/// Wait for all submitted GPU work to complete, with a panic-safe wrapper.
///
/// WGPU 30.0.0 + DX12 panics internally (at `wgpu_core.rs:1924`) when the
/// device is lost during a `device.poll(Wait)`. We use
/// `catch_unwind(AssertUnwindSafe(...))` to prevent those panics from crashing
/// the application. If the device is genuinely lost we bail out early with
/// `Err` so the caller can abort the IBL pipeline gracefully.
///
/// Unlike a `Poll`-based busy-loop, this uses the real GPU fence wait so
/// submissions are properly serialised (no pile-up).
fn poll_wait(device: &Device, label: &str) -> Result<(), ()> {
    let start = std::time::Instant::now();
    loop {
        if start.elapsed() > GPU_POLL_TIMEOUT {
            warn!(
                "[IBL] Poll wait for '{}' timed out after {:?}",
                label, GPU_POLL_TIMEOUT
            );
            return Err(());
        }

        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            device.poll(wgpu::wgt::PollType::Wait {
                submission_index: None,
                timeout: None,
            })
        })) {
            Ok(Ok(_)) => return Ok(()),
            Ok(Err(e)) => {
                warn!("[IBL] Wait error for '{}': {:?}. Retrying ...", label, e);
                std::thread::sleep(Duration::from_millis(100));
                continue;
            }
            Err(panic_payload) => {
                warn!(
                    "[IBL] Wait for '{}' panicked (device lost?): {:?}",
                    label, panic_payload
                );
                return Err(());
            }
        }
    }
}

/// Fixed size of the dynamic sky's irradiance cube. The analytic diffuse is
/// smooth, so a small cube is visually indistinguishable from a full-size one
/// while costing ~4× less per frame.
const DYNAMIC_SKY_DIFFUSE_SIZE: u32 = 128;

/// Reusable GPU state for in-place updates of a generated dynamic sky.
///
/// Caches the compute pipelines, bind-group layouts, the `SkyParams` uniform
/// buffer and the per-mip bind groups so that
/// [`WorldEnvironment::update_sky_parameters`] only needs to rewrite a uniform
/// and dispatch a handful of cheap passes — no pipeline recompilation, no
/// texture allocation, no `poll_wait`.
#[derive(Debug)]
struct DynamicSkyState {
    params_buffer: wgpu::Buffer,
    sky_bind_group: BindGroup,
    diffuse_bind_group: BindGroup,
    pipeline_sky_cube: ComputePipeline,
    pipeline_diffuse: ComputePipeline,
    pipeline_mip: ComputePipeline,
    /// Per-mip convolve bind groups (group 0: src LoD 0 cube + sampler + dst).
    /// Index `i` → mip level `i + 1`; mip 0 is written directly and never
    /// convolved in-place.
    mip_src_bind_groups: Vec<BindGroup>,
    /// Per-mip `MipInfo` uniform bind groups (group 1). Same indexing as above.
    mip_buffer_bind_groups: Vec<BindGroup>,
    /// Round-robin counter selecting which specular mip to convolve next.
    next_mip: AtomicU32,
    cube_face_size: u32,
    specular_mip_level_count: u32,
    sampling_type: SamplingType,
}

#[derive(Debug)]
pub struct WorldEnvironment {
    /// IBL (= Image Based Lighting) diffuse Texture.
    /// To be used for illuminating objects in the current [`World`].
    ///
    /// _Should_ only contain a single LoD/MipMap.
    ibl_diffuse: Texture,
    /// IBL (= Image Based Lighting) specular Texture.
    /// To be used for sky box rendering and imitating reflections.
    ///
    /// _Should_ contain multiple LoD/MipMap's.
    /// Each LoD makes the sampled reflection blurrier and rougher (* if sampled correctly).
    ibl_specular: Texture,
    /// [`MaterialShader`] to be used with this [`WorldEnvironment`].
    material_shader: MaterialShader,
    /// Present only for `WorldEnvironmentDescriptor::Generated { dynamic: true }`
    /// skies — enables cheap in-place [`WorldEnvironment::update_sky_parameters`].
    dynamic_state: Option<DynamicSkyState>,
    /// `SkyParams` uniform for non-dynamic `Generated` skies (dynamic skies
    /// reuse [`DynamicSkyState::params_buffer`]). Bound by the world bind group
    /// so the analytic skybox shader can evaluate `sky_color` per-pixel.
    sky_params_buffer: Option<wgpu::Buffer>,
}

impl WorldEnvironment {
    pub fn bind_group_layout_descriptor() -> BindGroupLayoutDescriptor<'static> {
        BindGroupLayoutDescriptor {
            label: Some("Equirectangular to PBR IBL Environment Maps"),
            entries: &[
                // Input: Equirectangular Image as source
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Output
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba16Float,
                        view_dimension: TextureViewDimension::D2Array,
                    },
                    count: None,
                },
                // Tile offset (used by the diffuse shader to split a face across
                // multiple dispatches; specular shader declares but ignores it).
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        }
    }

    pub fn bind_group_layout_descriptor_mip_mapping() -> BindGroupLayoutDescriptor<'static> {
        BindGroupLayoutDescriptor {
            label: Some("PBR IBL Specular Environment Mip Mapping"),
            entries: &[
                // Input: PBR IBL Specular with LoD = 0 generated as source
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::Cube,
                        multisampled: false,
                    },
                    count: None,
                },
                // Src sampler
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                // Output
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba16Float,
                        view_dimension: TextureViewDimension::D2Array,
                    },
                    count: None,
                },
            ],
        }
    }

    pub fn bind_group_layout_descriptor_buffer() -> BindGroupLayoutDescriptor<'static> {
        BindGroupLayoutDescriptor {
            label: Some("Mip Buffer Bind Group Layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        }
    }

    pub fn bind_group_layout_descriptor_sky_cube() -> BindGroupLayoutDescriptor<'static> {
        BindGroupLayoutDescriptor {
            label: Some("Sky Generation (direct-to-cube)"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba16Float,
                        view_dimension: TextureViewDimension::D2Array,
                    },
                    count: None,
                },
            ],
        }
    }

    pub fn find_cache_dir() -> PathBuf {
        dirs::cache_dir().expect("Could not find a valid cache location for the current platform! This platform might be unsupported ...")
    .join("Orbital").join("IBLs")
    }

    pub fn find_cache_file(descriptor: &WorldEnvironmentDescriptor) -> PathBuf {
        let cache_dir = Self::find_cache_dir();

        // Hash the descriptor to use as filename
        let mut hasher = DefaultHasher::new();
        descriptor.hash(&mut hasher);
        let hash = hasher.finish().to_string();

        cache_dir.join(format!("{hash}.bin"))
    }

    pub fn from_descriptor(
        descriptor: &WorldEnvironmentDescriptor,
        surface_texture_format: Option<TextureFormat>,
        device: &Device,
        queue: &Queue,
    ) -> Result<Self, Box<dyn Error>> {
        // Dynamic generated skies are expected to change frequently, so the
        // disk cache is bypassed entirely — every change regenerates in memory.
        let dynamic = matches!(
            descriptor,
            WorldEnvironmentDescriptor::Generated { dynamic: true, .. }
        );

        // Generated skies use the analytic skybox (per-pixel `sky_color`);
        // file/data environments keep the texture-sampled skybox.
        let generated = matches!(descriptor, WorldEnvironmentDescriptor::Generated { .. });

        let cache_file = Self::find_cache_file(descriptor);
        debug!("Cache file: {:?}", cache_file);

        // Try loading cache file
        let (pbr_ibl_diffuse, pbr_ibl_specular, write_to_cache) = if dynamic {
            let (x, y) = Self::make_from_descriptor(descriptor, device, queue)?;
            (x, y, false)
        } else {
            match CacheFile::from_path(cache_file.clone()) {
                Ok(cache_file) => {
                    let (pbr_ibl_diffuse, pbr_ibl_specular) =
                        cache_file.make_textures(descriptor, device, queue);

                    info!("Using cached WorldEnvironment/IBL!");
                    debug!(
                        "Cached PBR IBL Diffuse Size: {:?} + Mip Levels: {:?}",
                        pbr_ibl_diffuse.texture().size(),
                        pbr_ibl_diffuse.texture().mip_level_count()
                    );
                    debug!(
                        "Cached PBR IBL Specular Size: {:?} + Mip Levels: {:?}",
                        pbr_ibl_specular.texture().size(),
                        pbr_ibl_specular.texture().mip_level_count()
                    );

                    (pbr_ibl_diffuse, pbr_ibl_specular, false)
                }
                Err(e) => {
                    warn!(
                        "WorldEnvironment::IBL cache failed to load, is corrupt or doesn't exist! Will continue generating IBL from HDRI. This may take a few seconds. Error: {e:?}"
                    );

                    let (x, y) = Self::make_from_descriptor(descriptor, device, queue)?;
                    (x, y, true)
                }
            }
        };

        let shader = Self::make_material_shader(generated, surface_texture_format, device, queue)?;

        let mut s = Self {
            ibl_diffuse: pbr_ibl_diffuse,
            ibl_specular: pbr_ibl_specular,
            material_shader: shader,
            dynamic_state: None,
            sky_params_buffer: None,
        };

        // Dynamic skies update their textures in place, so build the reusable
        // GPU state (pipelines, bind groups, parameter buffer) up-front.
        if dynamic {
            let state = match descriptor {
                WorldEnvironmentDescriptor::Generated {
                    cube_face_size,
                    sampling_type,
                    custom_specular_mip_level_count,
                    parameters,
                    ..
                } => Some(Self::build_dynamic_state(
                    &s,
                    parameters
                        .as_ref()
                        .unwrap_or(&GeneratedSkyParameters::default()),
                    *cube_face_size,
                    sampling_type,
                    Self::calculate_specular_mip_level_count(
                        *cube_face_size,
                        custom_specular_mip_level_count.as_ref(),
                    ),
                    device,
                )),
                _ => None,
            };
            s.dynamic_state = state;
        }

        // Static generated skies keep their `SkyParams` uniform for the
        // analytic skybox. Dynamic skies fall back to
        // `DynamicSkyState::params_buffer` (rewritten every frame).
        if !dynamic && let WorldEnvironmentDescriptor::Generated { parameters, .. } = descriptor {
            s.sky_params_buffer = Some(Self::make_sky_parameters_buffer(
                parameters
                    .as_ref()
                    .unwrap_or(&GeneratedSkyParameters::default()),
                device,
            ));
        }

        if write_to_cache && let Err(e) = s.write_to_cache(&cache_file, device, queue) {
            warn!("[IBL] Failed to write IBL cache (non-fatal): {e:?}");
        }

        Ok(s)
    }

    fn calculate_specular_mip_level_count(
        cube_face_size: u32,
        requested_mip_level_count: Option<&u32>,
    ) -> u32 {
        let max_possible_mip_levels = cube_face_size.ilog2() + 1;

        // Use a reasonable default of 7 levels (base level + 6 additional mipmap levels)
        // instead of generating the maximum possible number of mipmap levels
        // This provides good quality reflections while avoiding unnecessary computation
        // for very small mip levels (1x1, 2x2, 4x4) that don't contribute much to visual quality
        let reasonable_default_mip_levels = 7.min(max_possible_mip_levels);

        let requested_mip_levels = requested_mip_level_count
            .copied()
            .unwrap_or(reasonable_default_mip_levels);
        let clamped_mip_levels = requested_mip_levels.min(max_possible_mip_levels);

        if let Some(requested) = requested_mip_level_count
            && *requested > max_possible_mip_levels
        {
            warn!(
                "Requested specular mip level count {requested} exceeds maximum possible {max_possible_mip_levels} for cube face size {cube_face_size}. Clamping to {clamped_mip_levels}."
            );
        }

        clamped_mip_levels
    }

    pub fn make_from_descriptor(
        descriptor: &WorldEnvironmentDescriptor,
        device: &Device,
        queue: &Queue,
    ) -> Result<(Texture, Texture), Box<dyn Error>> {
        match descriptor {
            WorldEnvironmentDescriptor::FromFile {
                cube_face_size,
                path,
                sampling_type,
                custom_specular_mip_level_count: specular_mip_level_count,
            } => {
                let clamped_mip_levels = Self::calculate_specular_mip_level_count(
                    *cube_face_size,
                    specular_mip_level_count.as_ref(),
                );

                Self::radiance_hdr_file(
                    path,
                    *cube_face_size,
                    sampling_type,
                    clamped_mip_levels,
                    device,
                    queue,
                )
            }
            WorldEnvironmentDescriptor::FromData {
                cube_face_size,
                data,
                size,
                sampling_type,
                specular_mip_level_count,
            } => {
                let clamped_mip_levels = Self::calculate_specular_mip_level_count(
                    *cube_face_size,
                    specular_mip_level_count.as_ref(),
                );

                Self::radiance_hdr_vec(
                    data,
                    *size,
                    *cube_face_size,
                    sampling_type,
                    clamped_mip_levels,
                    device,
                    queue,
                )
            }
            WorldEnvironmentDescriptor::Generated {
                cube_face_size,
                sampling_type,
                custom_specular_mip_level_count: specular_mip_level_count,
                parameters,
                dynamic,
                ..
            } => {
                let clamped_mip_levels = Self::calculate_specular_mip_level_count(
                    *cube_face_size,
                    specular_mip_level_count.as_ref(),
                );

                Self::make_ibl_from_sky_parameters(
                    parameters
                        .as_ref()
                        .unwrap_or(&GeneratedSkyParameters::default()),
                    *cube_face_size,
                    // Dynamic skies regenerate their diffuse every frame, so a
                    // smaller cube keeps the per-frame cost flat.
                    if *dynamic {
                        DYNAMIC_SKY_DIFFUSE_SIZE
                    } else {
                        *cube_face_size
                    },
                    sampling_type,
                    clamped_mip_levels,
                    device,
                    queue,
                )
            }
            WorldEnvironmentDescriptor::None => Err(Box::new(WorldEnvironmentError::msg(
                "WorldEnvironmentDescriptor::None should not reach make_from_descriptor",
            ))),
        }
    }

    pub fn radiance_hdr_file(
        file_path: &str,
        dst_size: u32,
        sampling_type: &SamplingType,
        specular_mip_level_count: u32,
        device: &Device,
        queue: &Queue,
    ) -> Result<(Texture, Texture), Box<dyn Error>> {
        let img = ImageReader::open(file_path)
            .map_err(WorldEnvironmentError::IO)?
            .decode()
            .map_err(WorldEnvironmentError::Image)?;

        let width = img.dimensions().0;
        let height = img.dimensions().1;

        let data = img
            .into_rgba32f()
            .iter()
            .map(|x| x.to_le_bytes())
            .collect::<Vec<_>>()
            .concat();

        Self::radiance_hdr_vec(
            &data,
            Vector2 {
                x: width,
                y: height,
            },
            dst_size,
            sampling_type,
            specular_mip_level_count,
            device,
            queue,
        )
    }

    pub fn radiance_hdr_vec(
        data: &[u8],
        src_size: Vector2<u32>,
        dst_size: u32,
        sampling_type: &SamplingType,
        specular_mip_level_count: u32,
        device: &Device,
        queue: &Queue,
    ) -> Result<(Texture, Texture), Box<dyn Error>> {
        let src_texture = Texture::from_descriptors_and_data(
            &wgpu::TextureDescriptor {
                label: Some("Equirectangular SRC"),
                size: Extent3d {
                    width: src_size.x,
                    height: src_size.y,
                    ..Default::default()
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba32Float,
                usage: TextureUsages::STORAGE_BINDING
                    | TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST,
                view_formats: &[],
            },
            &TextureViewDescriptor::default(),
            &SamplerDescriptor {
                label: Some("Equirectangular SRC Sampler"),
                address_mode_u: AddressMode::ClampToEdge,
                address_mode_v: AddressMode::ClampToEdge,
                address_mode_w: AddressMode::ClampToEdge,
                mag_filter: WFilterMode::Linear,
                min_filter: WFilterMode::Linear,
                mipmap_filter: MipmapFilterMode::Linear,
                compare: Some(CompareFunction::Always),
                ..Default::default()
            },
            Some((
                data,
                Extent3d {
                    width: src_size.x,
                    height: src_size.y,
                    ..Default::default()
                },
            )),
            device,
            queue,
        );

        let bind_group_layout =
            device.create_bind_group_layout(&Self::bind_group_layout_descriptor());

        // Phase 1a: Generate diffuse irradiance map.
        // Each face is split into tiles to stay under the DX12 TDR timeout.
        let diffuse = Self::dispatch_ibl_cubemap_per_face(
            dst_size,
            "PBR IBL Diffuse",
            &bind_group_layout,
            src_texture.view(),
            include_wgsl!("make_ibl_diffuse.wgsl"),
            TextureFormat::Rgba16Float,
            1,
            TILE_SIZE,
            device,
            queue,
        )?;

        // Phase 1b: Generate raw specular (LoD 0).
        // This shader is very fast (1 sample/texel), so no tiling needed.
        let raw_specular = Self::dispatch_ibl_cubemap_per_face(
            dst_size,
            "PBR IBL Specular without LoDs",
            &bind_group_layout,
            src_texture.view(),
            include_wgsl!("make_ibl_specular.wgsl"),
            TextureFormat::Rgba16Float,
            specular_mip_level_count,
            dst_size,
            device,
            queue,
        )?;

        // Phase 2: Generate specular mip maps — each mip level is submitted
        //           separately to stay under the Windows/DX12 TDR timeout.
        let specular = Self::generate_specular_mip_maps_incremental(
            &raw_specular,
            sampling_type,
            specular_mip_level_count,
            device,
            queue,
        )?;

        Ok((diffuse, specular))
    }

    /// Generates the analytic time-of-day sky as a full IBL environment.
    ///
    /// This is the *fast* path: the sky is written **directly** into the cube
    /// faces (no equirectangular intermediate) and the diffuse irradiance is
    /// computed analytically (deterministic Fibonacci-sphere quadrature +
    /// closed-form sun/moon disks) instead of an 8192-sample Monte Carlo
    /// convolution.  Both shaders share the same `sky_color` function, so the
    /// skybox and the irradiance are always consistent.
    fn make_ibl_from_sky_parameters(
        params: &GeneratedSkyParameters,
        dst_size: u32,
        diffuse_size: u32,
        sampling_type: &SamplingType,
        specular_mip_level_count: u32,
        device: &Device,
        queue: &Queue,
    ) -> Result<(Texture, Texture), Box<dyn Error>> {
        let bind_group_layout =
            device.create_bind_group_layout(&Self::bind_group_layout_descriptor_sky_cube());

        let params_buffer = Self::make_sky_parameters_buffer(params, device);

        // Phase 1: Generate the sky directly into the specular cube (LoD 0).
        let raw_specular = Self::dispatch_sky_cube(
            dst_size,
            "Generated Sky Specular (direct-to-cube)",
            &bind_group_layout,
            &params_buffer,
            include_str!("generate_sky_cube.wgsl"),
            TextureFormat::Rgba16Float,
            specular_mip_level_count,
            device,
            queue,
        )?;

        // Phase 2: Generate diffuse irradiance analytically, also direct-to-cube.
        let diffuse = Self::dispatch_sky_cube(
            diffuse_size,
            "Generated Sky Diffuse (analytic)",
            &bind_group_layout,
            &params_buffer,
            include_str!("make_ibl_diffuse_analytic.wgsl"),
            TextureFormat::Rgba16Float,
            1,
            device,
            queue,
        )?;

        // Phase 3: Generate specular mip maps for reflections.
        let specular = Self::generate_specular_mip_maps_incremental(
            &raw_specular,
            sampling_type,
            specular_mip_level_count,
            device,
            queue,
        )?;

        Ok((diffuse, specular))
    }

    /// Builds the reusable GPU state needed by [`WorldEnvironment::update_sky_parameters`].
    ///
    /// This is done once when a `Generated { dynamic: true }` environment is
    /// created, so subsequent per-frame updates never recompile pipelines,
    /// allocate textures or block on `poll_wait`.
    #[allow(clippy::too_many_arguments)]
    fn build_dynamic_state(
        env: &WorldEnvironment,
        params: &GeneratedSkyParameters,
        cube_face_size: u32,
        sampling_type: &SamplingType,
        specular_mip_level_count: u32,
        device: &Device,
    ) -> DynamicSkyState {
        let sky_layout =
            device.create_bind_group_layout(&Self::bind_group_layout_descriptor_sky_cube());
        let mip_layout =
            device.create_bind_group_layout(&Self::bind_group_layout_descriptor_mip_mapping());
        let mip_buffer_layout =
            device.create_bind_group_layout(&Self::bind_group_layout_descriptor_buffer());

        let params_buffer = Self::make_sky_parameters_buffer(params, device);

        let sky_source = format!(
            "{}\n{}",
            include_str!("sky_common.wgsl"),
            include_str!("generate_sky_cube.wgsl")
        );
        let pipeline_sky_cube = Self::make_compute_pipeline(
            &[Some(&sky_layout)],
            ShaderModuleDescriptor {
                label: Some("Dynamic Sky Cube"),
                source: wgpu::ShaderSource::Wgsl(sky_source.into()),
            },
            "main",
            device,
        );

        let diffuse_source = format!(
            "{}\n{}",
            include_str!("sky_common.wgsl"),
            include_str!("make_ibl_diffuse_analytic.wgsl")
        );
        let pipeline_diffuse = Self::make_compute_pipeline(
            &[Some(&sky_layout)],
            ShaderModuleDescriptor {
                label: Some("Dynamic Sky Diffuse"),
                source: wgpu::ShaderSource::Wgsl(diffuse_source.into()),
            },
            "main",
            device,
        );

        let pipeline_mip = Self::make_compute_pipeline(
            &[Some(&mip_layout), Some(&mip_buffer_layout)],
            include_wgsl!("make_mip_maps.wgsl"),
            "main",
            device,
        );

        let specular = env.ibl_specular();
        let diffuse = env.ibl_diffuse();

        // LoD 0 of the specular cube — written every frame by the sky shader.
        let sky_dst_view = specular.texture().create_view(&TextureViewDescriptor {
            label: Some("Dynamic Sky LoD 0 dst"),
            dimension: Some(TextureViewDimension::D2Array),
            base_mip_level: 0,
            mip_level_count: Some(1),
            ..Default::default()
        });
        let sky_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Dynamic Sky Cube Bind Group"),
            layout: &sky_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(params_buffer.as_entire_buffer_binding()),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&sky_dst_view),
                },
            ],
        });

        let diffuse_dst_view = diffuse.texture().create_view(&TextureViewDescriptor {
            label: Some("Dynamic Sky Diffuse dst"),
            dimension: Some(TextureViewDimension::D2Array),
            base_mip_level: 0,
            mip_level_count: Some(1),
            ..Default::default()
        });
        let diffuse_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Dynamic Sky Diffuse Bind Group"),
            layout: &sky_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(params_buffer.as_entire_buffer_binding()),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&diffuse_dst_view),
                },
            ],
        });

        // LoD 0-only cube view used as the mip-convolution source. It must NOT
        // cover the mip being written as `dst`, otherwise the sampled and
        // storage bindings would alias the same subresource.
        let src_view = specular.texture().create_view(&TextureViewDescriptor {
            label: Some("Dynamic Sky LoD 0 src"),
            dimension: Some(TextureViewDimension::Cube),
            base_mip_level: 0,
            mip_level_count: Some(1),
            ..Default::default()
        });

        let base_size = specular.texture().size();

        // Pre-build per-mip bind groups for mips 1..N (LoD 0 is written
        // directly by the sky shader and never convolved in-place).
        // Index `i` → mip level `i + 1`.
        let mut mip_src_bind_groups = Vec::new();
        let mut mip_buffer_bind_groups = Vec::new();
        for mip_level in 1..specular_mip_level_count {
            let dst_view = specular.texture().create_view(&TextureViewDescriptor {
                label: Some("Dynamic Sky Mip dst"),
                dimension: Some(TextureViewDimension::D2Array),
                base_mip_level: mip_level,
                mip_level_count: Some(1),
                ..Default::default()
            });
            mip_src_bind_groups.push(device.create_bind_group(&BindGroupDescriptor {
                label: Some("Dynamic Sky Mip src"),
                layout: &mip_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&src_view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(specular.sampler()),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: BindingResource::TextureView(&dst_view),
                    },
                ],
            }));
            mip_buffer_bind_groups.push(Self::make_mip_buffer(
                mip_level,
                specular_mip_level_count,
                base_size.width,
                base_size.height,
                sampling_type,
                &mip_buffer_layout,
                device,
            ));
        }

        DynamicSkyState {
            params_buffer,
            sky_bind_group,
            diffuse_bind_group,
            pipeline_sky_cube,
            pipeline_diffuse,
            pipeline_mip,
            mip_src_bind_groups,
            mip_buffer_bind_groups,
            next_mip: AtomicU32::new(0),
            cube_face_size,
            specular_mip_level_count,
            sampling_type: sampling_type.clone(),
        }
    }

    /// In-place update of a generated dynamic sky.
    ///
    /// Reuses the existing GPU textures and pipelines — one encoder, one
    /// submission, no `poll_wait` (the following render pass synchronises
    /// naturally on the same queue).  The sky LoD 0 and analytic diffuse are
    /// updated every call; the specular mip levels (1..N) are convolved one
    /// per call, round-robin, so the per-frame cost stays flat.
    ///
    /// Only valid for environments created from
    /// `WorldEnvironmentDescriptor::Generated { dynamic: true }`.
    pub fn update_sky_parameters(
        &self,
        params: &GeneratedSkyParameters,
        device: &Device,
        queue: &Queue,
    ) {
        let Some(state) = &self.dynamic_state else {
            warn!("update_sky_parameters called on a non-dynamic WorldEnvironment");
            return;
        };

        queue.write_buffer(
            &state.params_buffer,
            0,
            &Self::make_sky_parameters_data(params),
        );

        let workgroup_size = 8u32;
        let wg_x = state.cube_face_size.div_ceil(workgroup_size);
        let wg_y = state.cube_face_size.div_ceil(workgroup_size);

        // The analytic diffuse is regenerated every frame at a fixed smaller
        // size (see `DYNAMIC_SKY_DIFFUSE_SIZE`) — dispatch at the actual cube
        // resolution, not the specular cube's.
        let diffuse_size = self.ibl_diffuse.texture().size().width;
        let diffuse_wg = diffuse_size.div_ceil(workgroup_size);

        let mut encoder = device.create_command_encoder(&Default::default());

        // Pass 1 — LoD 0: write the sky directly into the existing cube.
        {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Dynamic Sky: LoD 0"),
                ..Default::default()
            });
            pass.set_pipeline(&state.pipeline_sky_cube);
            pass.set_bind_group(0, &state.sky_bind_group, &[]);
            pass.dispatch_workgroups(wg_x, wg_y, 6);
        }

        // Pass 2 — analytic diffuse into the existing irradiance cube.
        {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Dynamic Sky: Diffuse"),
                ..Default::default()
            });
            pass.set_pipeline(&state.pipeline_diffuse);
            pass.set_bind_group(0, &state.diffuse_bind_group, &[]);
            pass.dispatch_workgroups(diffuse_wg, diffuse_wg, 6);
        }

        // Pass 3 — one specular mip per update, round-robin over 1..N-1.
        let mip_count = state.specular_mip_level_count;
        if mip_count > 1 {
            let mip_level = 1 + state.next_mip.fetch_add(1, Ordering::Relaxed) % (mip_count - 1);
            let mip_size = (state.cube_face_size >> mip_level).max(1);
            let idx = (mip_level - 1) as usize;

            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Dynamic Sky: Specular Mip"),
                ..Default::default()
            });
            pass.set_pipeline(&state.pipeline_mip);
            pass.set_bind_group(0, &state.mip_src_bind_groups[idx], &[]);
            pass.set_bind_group(1, &state.mip_buffer_bind_groups[idx], &[]);
            pass.dispatch_workgroups(
                mip_size.div_ceil(workgroup_size),
                mip_size.div_ceil(workgroup_size),
                6,
            );
        }

        queue.submit([encoder.finish()]);
    }

    /// Whether this environment can be updated in-place from `descriptor` —
    /// i.e. it was created as a `Generated { dynamic: true }` sky of matching
    /// cube size, mip count and sampling type.
    pub fn can_update_dynamic_sky(&self, descriptor: &WorldEnvironmentDescriptor) -> bool {
        let Some(state) = &self.dynamic_state else {
            return false;
        };
        match descriptor {
            WorldEnvironmentDescriptor::Generated {
                cube_face_size,
                sampling_type,
                custom_specular_mip_level_count,
                dynamic: true,
                ..
            } => {
                state.cube_face_size == *cube_face_size
                    && state.sampling_type == *sampling_type
                    && state.specular_mip_level_count
                        == Self::calculate_specular_mip_level_count(
                            *cube_face_size,
                            custom_specular_mip_level_count.as_ref(),
                        )
            }
            _ => false,
        }
    }

    /// Dispatches a sky-generation compute shader that writes directly into a
    /// cubemap (all 6 faces in a single dispatch, selected by `gid.z`).
    ///
    /// The shader source is the concatenation of `sky_common.wgsl` (shared
    /// `SkyParams` + `sky_color`) and the given entry shader.
    fn dispatch_sky_cube(
        dst_size: u32,
        label: &str,
        bind_group_layout: &BindGroupLayout,
        params_buffer: &wgpu::Buffer,
        entry_shader: &'static str,
        format: TextureFormat,
        mip_level_count: u32,
        device: &Device,
        queue: &Queue,
    ) -> Result<Texture, Box<dyn Error>> {
        let source = format!("{}\n{}", include_str!("sky_common.wgsl"), entry_shader);
        let pipeline = Self::make_compute_pipeline(
            &[Some(bind_group_layout)],
            ShaderModuleDescriptor {
                label: Some(label),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            },
            "main",
            device,
        );

        let dst_texture = Texture::create_empty_cube_texture(
            Some(label),
            Vector2 {
                x: dst_size,
                y: dst_size,
            },
            format,
            TextureUsages::STORAGE_BINDING
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC,
            mip_level_count,
            device,
        );

        // Full D2Array view covering all 6 faces (single mip level — storage
        // texture bindings cannot span multiple mips).
        let dst_view = dst_texture.texture().create_view(&TextureViewDescriptor {
            label: Some(&format!("{label} full D2Array view")),
            dimension: Some(TextureViewDimension::D2Array),
            base_mip_level: 0,
            mip_level_count: Some(1),
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some(label),
            layout: bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(params_buffer.as_entire_buffer_binding()),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&dst_view),
                },
            ],
        });

        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some(label),
                ..Default::default()
            });

            let workgroup_size = 8u32;
            let wg_x = dst_size.div_ceil(workgroup_size);
            let wg_y = dst_size.div_ceil(workgroup_size);

            debug!("{label} ({}², {mip_level_count} mips) ...", dst_size);

            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(wg_x, wg_y, 6);
        }
        queue.submit([encoder.finish()]);
        poll_wait(device, label).map_err(|_| format!("{label} failed"))?;

        Ok(dst_texture)
    }

    /// Helper: dispatches a cubemap-generating compute shader one face at a
    /// time.  Heavy shaders (diffuse) are further split into tiles so that no
    /// single GPU dispatch exceeds the DX12 TDR timeout.
    ///
    /// Pass `tile_size >= dst_size` to skip tiling (e.g. for the specular
    /// base shader which takes <1 ms per face).
    fn dispatch_ibl_cubemap_per_face(
        dst_size: u32,
        label: &str,
        bind_group_layout: &BindGroupLayout,
        src_view: &TextureView,
        shader: ShaderModuleDescriptor<'static>,
        format: TextureFormat,
        mip_level_count: u32,
        tile_size: u32,
        device: &Device,
        queue: &Queue,
    ) -> Result<Texture, Box<dyn Error>> {
        let pipeline =
            Self::make_compute_pipeline(&[Some(bind_group_layout)], shader, "main", device);

        let dst_texture = Texture::create_empty_cube_texture(
            Some(label),
            Vector2 {
                x: dst_size,
                y: dst_size,
            },
            format,
            TextureUsages::STORAGE_BINDING
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC,
            mip_level_count,
            device,
        );

        let workgroup_size = 8u32;
        let tiles = dst_size.div_ceil(tile_size);

        for face in 0..6 {
            let dst_view = dst_texture.texture().create_view(&TextureViewDescriptor {
                label: Some(&format!("{label} face {face}")),
                dimension: Some(TextureViewDimension::D2Array),
                base_array_layer: face,
                array_layer_count: Some(1),
                base_mip_level: 0,
                mip_level_count: Some(1),
                ..Default::default()
            });

            // One reusable uniform buffer per face — updated via
            // `queue.write_buffer` before each tile.
            let offset_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("{label} face {face} offset")),
                size: 16,
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            debug!("{label} face {face} ...");

            for ty in 0..tiles {
                for tx in 0..tiles {
                    let tile_x = tx * tile_size;
                    let tile_y = ty * tile_size;
                    let wg_x = tile_size.min(dst_size - tile_x).div_ceil(workgroup_size);
                    let wg_y = tile_size.min(dst_size - tile_y).div_ceil(workgroup_size);

                    let mut buf = [0u8; 16];
                    buf[..4].copy_from_slice(&tile_x.to_ne_bytes());
                    buf[4..8].copy_from_slice(&tile_y.to_ne_bytes());
                    buf[8..12].copy_from_slice(&face.to_ne_bytes());
                    queue.write_buffer(&offset_buffer, 0, &buf);

                    let bind_group = device.create_bind_group(&BindGroupDescriptor {
                        label: Some(&format!("{label} face {face} tile {tx}x{ty}")),
                        layout: bind_group_layout,
                        entries: &[
                            BindGroupEntry {
                                binding: 0,
                                resource: BindingResource::TextureView(src_view),
                            },
                            BindGroupEntry {
                                binding: 1,
                                resource: BindingResource::TextureView(&dst_view),
                            },
                            BindGroupEntry {
                                binding: 2,
                                resource: BindingResource::Buffer(wgpu::BufferBinding {
                                    buffer: &offset_buffer,
                                    offset: 0,
                                    size: None,
                                }),
                            },
                        ],
                    });

                    let mut encoder = device.create_command_encoder(&Default::default());
                    {
                        let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                            label: Some(&format!("{label} face {face} tile {tx}x{ty}")),
                            ..Default::default()
                        });
                        pass.set_pipeline(&pipeline);
                        pass.set_bind_group(0, &bind_group, &[]);
                        pass.dispatch_workgroups(wg_x, wg_y, 1);
                    }
                    queue.submit([encoder.finish()]);
                    poll_wait(device, &format!("{label} face {face} tile {tx}x{ty}"))
                        .map_err(|_| format!("{label} failed on face {face}"))?;
                }
            }
        }

        Ok(dst_texture)
    }

    fn generate_specular_mip_maps_incremental(
        src_specular_ibl: &Texture,
        sampling_type: &SamplingType,
        specular_mip_level_count: u32,
        device: &Device,
        queue: &Queue,
    ) -> Result<Texture, Box<dyn Error>> {
        let bind_group_layout =
            device.create_bind_group_layout(&Self::bind_group_layout_descriptor_mip_mapping());
        let mip_buffer_bind_group_layout =
            device.create_bind_group_layout(&Self::bind_group_layout_descriptor_buffer());

        let pipeline = Self::make_compute_pipeline(
            &[
                Some(&bind_group_layout),
                Some(&mip_buffer_bind_group_layout),
            ],
            include_wgsl!("make_mip_maps.wgsl"),
            "main",
            device,
        );

        let max_mip_levels = specular_mip_level_count;

        let dst_texture = Texture::create_empty_cube_texture(
            Some("PBR IBL Specular with LoDs"),
            Vector2 {
                x: src_specular_ibl.texture().width(),
                y: src_specular_ibl.texture().height(),
            },
            TextureFormat::Rgba16Float,
            TextureUsages::STORAGE_BINDING
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC,
            max_mip_levels,
            device,
        );

        let dst_size = dst_texture.texture().size();

        for mip_level in 0..max_mip_levels {
            let dst_view = dst_texture.texture().create_view(&TextureViewDescriptor {
                label: Some("PBR IBL Specular LoD processing view"),
                dimension: Some(TextureViewDimension::D2Array),
                base_mip_level: mip_level,
                mip_level_count: Some(1),
                ..Default::default()
            });

            let bind_group = device.create_bind_group(&BindGroupDescriptor {
                label: Some("World Environment Processing Bind Group for PBR IBL Specular Mip"),
                layout: &bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(src_specular_ibl.view()),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(src_specular_ibl.sampler()),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: BindingResource::TextureView(&dst_view),
                    },
                ],
            });

            let mip_bind_group = Self::make_mip_buffer(
                mip_level,
                max_mip_levels,
                dst_size.width,
                dst_size.height,
                sampling_type,
                &mip_buffer_bind_group_layout,
                device,
            );

            // Each mip level gets its own encoder and submission to avoid
            // Windows/DX12 TDR from overly large single submissions.
            let mut encoder = device.create_command_encoder(&Default::default());
            {
                let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                    label: Some("PBR IBL Specular Mip Mapping task"),
                    ..Default::default()
                });

                debug!(
                    "Generating PBR IBL Specular (LoD = {} / Roughness = {:.1}%) ...",
                    mip_level,
                    if max_mip_levels > 1 {
                        (mip_level as f32 / (max_mip_levels - 1) as f32) * 100.0
                    } else {
                        0.0
                    }
                );
                let current_mip_width = (dst_size.width >> mip_level).max(1);
                let current_mip_height = (dst_size.height >> mip_level).max(1);
                // Calculate workgroup count based on current mip level dimensions
                // Using 8x8 workgroups for better occupancy
                let workgroup_size = 8u32;
                let workgroups_x = current_mip_width.div_ceil(workgroup_size);
                let workgroups_y = current_mip_height.div_ceil(workgroup_size);
                pass.set_pipeline(&pipeline);
                pass.set_bind_group(0, &bind_group, &[]);
                pass.set_bind_group(1, &mip_bind_group, &[]);
                pass.dispatch_workgroups(workgroups_x, workgroups_y, 6);
            }
            queue.submit([encoder.finish()]);
            poll_wait(device, &format!("IBL Specular Mip Level {mip_level}"))
                .map_err(|_| format!("IBL specular mip level {mip_level} failed"))?;
        }

        Ok(dst_texture)
    }

    fn make_mip_buffer(
        mip_level: u32,
        max_mip_level: u32,
        base_width: u32,
        base_height: u32,
        sampling_type: &SamplingType,
        mip_buffer_bind_group_layout: &BindGroupLayout,
        device: &Device,
    ) -> BindGroup {
        let current_mip_width = (base_width >> mip_level).max(1);
        let current_mip_height = (base_height >> mip_level).max(1);

        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Mip Buffer"),
            contents: &[
                mip_level.to_le_bytes(),
                max_mip_level.to_le_bytes(),
                sampling_type.to_le_bytes(),
                current_mip_width.to_le_bytes(),
                current_mip_height.to_le_bytes(),
            ]
            .concat(),
            usage: BufferUsages::UNIFORM,
        });

        device.create_bind_group(&BindGroupDescriptor {
            label: Some("Mip Buffer Bind Group"),
            layout: mip_buffer_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(buffer.as_entire_buffer_binding()),
            }],
        })
    }

    /// Serializes [`GeneratedSkyParameters`] into the 176-byte layout of the
    /// WGSL `SkyParams` uniform struct (11 rows × 16 bytes).
    pub(crate) fn make_sky_parameters_data(params: &GeneratedSkyParameters) -> Vec<u8> {
        let mut data = Vec::with_capacity(176);

        let wf = |v: &mut Vec<u8>, f: f32| v.extend_from_slice(&f.to_le_bytes());

        let sun_dir = params.sun_direction();

        // Row 0: sun_direction (vec3) + _pad
        wf(&mut data, sun_dir[0]);
        wf(&mut data, sun_dir[1]);
        wf(&mut data, sun_dir[2]);
        wf(&mut data, 0.0);
        // Row 1: 4 × f32
        wf(&mut data, params.sun_angular_radius);
        wf(&mut data, params.sun_intensity);
        wf(&mut data, params.moon_angular_radius);
        wf(&mut data, params.moon_intensity);
        // Row 2: 4 × f32
        wf(&mut data, params.star_intensity);
        wf(&mut data, params.star_density);
        wf(&mut data, params.exposure);
        wf(&mut data, 0.0);
        // Row 3: ground_albedo (vec3) + _pad
        wf(&mut data, params.ground_albedo[0]);
        wf(&mut data, params.ground_albedo[1]);
        wf(&mut data, params.ground_albedo[2]);
        wf(&mut data, 0.0);
        // Rows 4-10: palette (7 × vec3 + pad each)
        for c in [
            &params.palette.day_zenith,
            &params.palette.day_horizon,
            &params.palette.night_zenith,
            &params.palette.night_horizon,
            &params.palette.twilight,
            &params.palette.sun_color,
            &params.palette.moon_color,
        ] {
            wf(&mut data, c[0]);
            wf(&mut data, c[1]);
            wf(&mut data, c[2]);
            wf(&mut data, 0.0);
        }

        data
    }

    pub fn make_sky_parameters_buffer(
        params: &GeneratedSkyParameters,
        device: &Device,
    ) -> wgpu::Buffer {
        let data = Self::make_sky_parameters_data(params);

        device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Sky Parameters"),
            contents: &data,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        })
    }

    fn make_compute_pipeline(
        bind_group_layouts: &[Option<&BindGroupLayout>],
        shader_module_descriptor: ShaderModuleDescriptor,
        shader_entrypoint: &str,
        device: &Device,
    ) -> ComputePipeline {
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts,
            immediate_size: 0,
        });

        let shader = device.create_shader_module(shader_module_descriptor);

        device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("WorldEnvironment Processing Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some(shader_entrypoint),
            compilation_options: Default::default(),
            cache: None,
        })
    }

    pub fn textures_to_texture_descriptors(
        pbr_ibl_diffuse: &Texture,
        pbr_ibl_specular: &Texture,
        device: &Device,
        queue: &Queue,
    ) -> (TextureDescriptor, TextureDescriptor) {
        let ibl_diffuse_data = pbr_ibl_diffuse.read_as_binary(device, queue);
        let ibl_diffuse_size = pbr_ibl_diffuse.texture().size();
        let ibl_diffuse_descriptor = TextureDescriptor::Data {
            pixels: ibl_diffuse_data,
            size: TextureSize {
                width: ibl_diffuse_size.width,
                height: ibl_diffuse_size.height,
                depth_or_array_layers: ibl_diffuse_size.depth_or_array_layers,
                base_mip: 0,
                mip_levels: pbr_ibl_diffuse.texture().mip_level_count(),
            },
            usages: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            format: pbr_ibl_diffuse.texture().format(),
            texture_dimension: TextureDimension::D2,
            texture_view_dimension: TextureViewDimension::Cube,
            filter_mode: FilterMode::nearest(),
        };

        let ibl_specular_data = pbr_ibl_specular.read_as_binary(device, queue);
        let ibl_specular_size = pbr_ibl_specular.texture().size();
        let ibl_specular_descriptor = TextureDescriptor::Data {
            pixels: ibl_specular_data,
            size: TextureSize {
                width: ibl_specular_size.width,
                height: ibl_specular_size.height,
                depth_or_array_layers: ibl_specular_size.depth_or_array_layers,
                base_mip: 0,
                mip_levels: pbr_ibl_specular.texture().mip_level_count(),
            },
            usages: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            format: pbr_ibl_specular.texture().format(),

            texture_dimension: TextureDimension::D2,
            texture_view_dimension: TextureViewDimension::Cube,
            filter_mode: FilterMode::nearest(),
        };

        (ibl_diffuse_descriptor, ibl_specular_descriptor)
    }

    pub fn make_material_shader_descriptor(generated: bool) -> MaterialShaderDescriptor {
        let shader_source = if generated {
            // Analytic skybox: `sky_common.wgsl` is prepended so the fragment
            // shader can evaluate `sky_color` per-pixel at full resolution.
            static ANALYTIC_SOURCE: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
                format!(
                    "{}\n{}",
                    include_str!("sky_common.wgsl"),
                    include_str!("material_shader_analytic.wgsl")
                )
            });
            ShaderSource::String(ANALYTIC_SOURCE.as_str())
        } else {
            ShaderSource::String(include_str!("material_shader.wgsl"))
        };

        MaterialShaderDescriptor {
            name: Some(String::from("WorldEnvironment MaterialShader")),
            shader_source,
            variables: vec![],
            depth_stencil: false,
            vertex_stage_layouts: None,
            cull_mode: None,
            ..Default::default()
        }
    }

    fn make_material_shader(
        generated: bool,
        surface_texture_format: Option<TextureFormat>,
        device: &Device,
        queue: &Queue,
    ) -> Result<MaterialShader, Box<dyn Error>> {
        let descriptor = Self::make_material_shader_descriptor(generated);

        MaterialShader::from_descriptor(&descriptor, surface_texture_format, device, queue)
    }

    pub fn write_to_cache(
        &self,
        cache_path: &PathBuf,
        device: &Device,
        queue: &Queue,
    ) -> Result<(), WorldEnvironmentError> {
        let ibl_diffuse_data = self.ibl_diffuse.read_as_binary(device, queue);
        let ibl_specular_data = self.ibl_specular.read_as_binary(device, queue);
        // Store the actual number of mip levels generated for the specular texture.
        // This ensures that when loading from cache, the texture is created with
        // the correct number of mip levels, regardless of the descriptor used for loading.
        let ibl_specular_mip_level_count = self.ibl_specular.texture().mip_level_count();
        debug!(
            "Writing IBL Specular cache with {} mip levels",
            ibl_specular_mip_level_count
        );

        let cache_file = CacheFile {
            ibl_diffuse_data,
            ibl_specular_data,
            ibl_specular_mip_level_count,
        };
        cache_file.to_path(cache_path)
    }

    pub fn ibl_diffuse(&self) -> &Texture {
        &self.ibl_diffuse
    }

    pub fn ibl_specular(&self) -> &Texture {
        &self.ibl_specular
    }

    pub fn material_shader(&self) -> &MaterialShader {
        &self.material_shader
    }

    /// The `SkyParams` uniform backing the analytic skybox — `Some` for all
    /// `Generated` skies (static generated own one, dynamic ones reuse the
    /// per-frame-updated [`DynamicSkyState::params_buffer`]).
    pub fn sky_parameters_buffer(&self) -> Option<&wgpu::Buffer> {
        self.sky_params_buffer
            .as_ref()
            .or_else(|| self.dynamic_state.as_ref().map(|s| &s.params_buffer))
    }
}
