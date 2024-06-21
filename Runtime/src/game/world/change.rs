use crate::resources::descriptors::{CompositionDescriptor, ModelDescriptor};

/// This is used to describe a [World](super::World) change.
/// When queueing changes, the order **is important**!
///
/// For example, say we do this:
/// 1. [Self::SwitchComposition]
/// 2. [Self::SpawnModels]
///
/// Then first, the [Composition] will be changed, then [Model]s are added.
///
/// However, if we instead do this in reverse:
/// 1. [Self::SpawnModels]
/// 2. [Self::SwitchComposition]
///
/// Then we would first spawn any [Model]s in, then **fully replace** the existing [Composition] with a new one.
/// Effectively undoing any changes done by [Self::SpawnModels]!
///
/// [Model]: crate::resources::realizations::Model
/// [Composition]: crate::resources::realizations::Composition
#[derive(Debug)]
pub enum WorldChangeDescriptor {
    /// This will **DROP** the current [Composition] and everything in it, by realizing the new [Composition] and **fully** replacing the previous [Composition].
    ///
    /// [Composition]: crate::resources::realizations::Composition
    SwitchComposition(CompositionDescriptor),

    /// This will **add** [Model]s to the existing [Composition] without dropping it.
    /// [Model]s will be realized, then added.d
    ///
    /// [Model]: crate::resources::realizations::Model
    /// [Composition]: crate::resources::realizations::Composition
    SpawnModels(Vec<ModelDescriptor>),
}
