use crate::resources::descriptors::ModelDescriptor;

#[derive(Default, Debug)]
pub struct ElementRegistration {
    /// Each [Element] can **optionally** have _Tags_.
    /// A _Tag_ can be used instead of the uniquely assigned [Ulid].
    ///
    /// Both, a _Tag_ and a [Ulid], can be used the same way.
    /// However, a [Ulid] is **unique** and only ever involves one [Element].
    /// On the other hand, a _Tag_ can be used for multiple objects.
    ///
    /// Say you want to send a message to all _NPCs_.
    /// Simply _Tag_ every NPC with `"NPC"` and send a message to that _Tag_.
    ///
    /// Similarly, this can also be used for e.g. removing [Element]s
    /// from the [World].
    /// Instead of removing all by their [Ulid] individually, a common _Tag_
    /// can be used to remove all at once.
    ///
    /// [Ulid]: ulid::Ulid
    /// [Element]: super::Element
    /// [World]: crate::game::world::World
    pub tags: Option<Vec<String>>,
    /// Each [Element] can **optionally** define one or more [Model]s to be
    /// associated with it.
    /// Upon registration, each [ModelDescriptor] will be
    /// realized into a [Model].
    ///
    /// Each [Model] will be linked with the registering [Element].
    /// If an [Element] is removed, any linked [Model] will also be removed.
    /// If an [Element] is moved, any linked [Model] will also be offset.
    ///
    /// If there is a need to remove or add [Model]s at some later point in time
    /// (i.e. **after registration**), it is possible to directly interact with
    /// [World]!
    ///
    /// [Model]: crate::resources::realizations::Model
    /// [ModelDescriptor]: crate::resources::descriptors::ModelDescriptor
    /// [Element]: super::Element
    /// [World]: crate::game::world::World
    pub models: Option<Vec<ModelDescriptor>>,
}
