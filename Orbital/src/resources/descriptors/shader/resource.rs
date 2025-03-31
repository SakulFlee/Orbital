use wgpu::{BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType};

/// [`ShaderResource`] that is build-in into the Engine.
/// Each entry represents an engine feature and doesn't require
/// any further setup.
/// The bound resource will always be the most up to date version
/// available to the engine.
#[derive(Debug)]
pub enum ShaderBuildInResource {
    /// Binds the current set [`Material`] set inside the
    /// [`Model`](crate::resources::realizations::Model).  
    /// Can only be used during a render cycle!
    ///
    /// Due to [`Material`]s being complex structures, we can't pre-define a
    /// binding index reference here.
    /// Check the [`Material`] directly for more info.
    ///
    /// [`Material`]: crate::resources::realizations::Material
    PBRMaterial,
    /// Binds the current light state
    /// TODO: Will be removed with future lighting
    LightState,
    /// Binds the current active [`Camera`](crate::resources::realizations::Camera).
    ///
    /// This is a single uniform binding containing a struct with multiple variable.
    /// Check the PBR shader for an up-to-date version.
    ///
    /// Example binding in WGSL:
    /// ```
    /// struct CameraUniform {
    ///     position: vec3<f32>,
    ///     view_projection_matrix: mat4x4<f32>,
    ///     perspective_view_projection_matrix: mat4x4<f32>,
    ///     view_projection_transposed: mat4x4<f32>,
    ///     perspective_projection_invert: mat4x4<f32>,
    ///     global_gamma: f32,
    /// }
    ///
    /// @group(X) @binding(0) var<uniform> camera: CameraUniform;
    /// ```
    Camera,
}

impl ShaderBuildInResource {
    pub fn bind_group_layout_entries(&self) -> Vec<BindingType> {
        match self {
            ShaderBuildInResource::PBRMaterial => vec![BindingType::Buffer {
                ty: todo!(),
                has_dynamic_offset: todo!(),
                min_binding_size: todo!(),
            }],
            ShaderBuildInResource::LightState => todo!(),
            ShaderBuildInResource::Camera => todo!(),
        }
    }
}

#[derive(Debug)]
pub enum ShaderStaticResource {
    Buffer(Vec<u8>),
    // TODO: Texture support
}

/// Defines the resources to be bound in the shader.  
/// Each entry represents a whole [`BindGroup`](crate::wgpu::BindGroup)!  
///
/// The order here defines the group index!
/// The first entry here will be `@group(0)`, 2nd `@group(1)`, etc.
///
/// There is a limit of **four** bing groups per shader (as of writing this!).
/// While this limit is softly enforced and can be disabled, it's best to stick to this limitation to support more devices.
/// If you require more and are having an issue with this, please raise an issue!
#[derive(Debug)]
pub enum ShaderResource {
    /// A build-in resource, no further setup will be required.
    /// The engine automatically binds whatever the most up to date state
    /// of said resource is upon executing the shader.
    ///
    /// Check the specific [`ShaderBuildInResource`] for binding index info!
    BuildIn(ShaderBuildInResource),
    /// A static, custom, resource that will be created **once** and **never
    /// updated**. This is effectively a read-only shader resource.
    ///
    /// Multiple [`ShaderStaticResource`] can be listed here.
    /// Each will be put together into **one** [`BindGroup`](crate::wgpu::BindGroup)!
    Static(Vec<ShaderStaticResource>),
}

impl ShaderResource {
    pub fn bind_group_layout_descriptor(&self) -> BindGroupLayoutDescriptor {
        let entries: Vec<BindGroupLayoutEntry> = match self {
            ShaderResource::BuildIn(shader_build_in_resource) => todo!(),
            ShaderResource::Static(vec) => todo!(),
        };

        BindGroupLayoutDescriptor {
            label: None,
            entries: &[BindGroupLayoutEntry {
                binding: todo!(),
                visibility: todo!(),
                ty: todo!(),
                count: todo!(),
            }],
        }
    }
}

// TODO: Maybe there's a better way of solving this? Custom Material definition maybe with special properties ... ?
