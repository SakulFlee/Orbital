use crate::resources::WorldEnvironmentDescriptor;

#[derive(Debug)]
pub enum RenderEvent {
    WorldEnvironmentChange {
        /// TODO
        /// If `None` -> Disable skybox render fully
        /// If `Some` -> Enable skybox at least partially
        descriptor: Option<WorldEnvironmentDescriptor>,
        /// TODO
        /// If `false` -> Only do skybox ("traditional" / without IBL)
        /// If `true` -> Also do IBL
        enable_ibl: Option<bool>, // TODO: Use in Skybox AND normal renderer
    },
}
