use std::sync::Arc;

use crate::resources::{MaterialShaderDescriptor, Mode, Transform};

use super::MeshDescriptor;

/// Descriptor for a model
#[derive(Debug, Clone)]
pub struct ModelDescriptor {
    pub label: String,
    pub mesh: Arc<MeshDescriptor>,
    /// TODO
    /// Multiple == Multiple materials being rendered ON-TOP
    /// Only define multiple if really required, e.g. "wireframes on-top of solid"
    pub materials: Vec<Arc<MaterialShaderDescriptor>>,
    /// TODO
    /// Multiple == Multiple instances of the same model
    pub transforms: Vec<Transform>,
}

impl ModelDescriptor {
    /// Sets one or multiple [Transform]s for this [Model].
    /// Will **replace** _any_ [Transform]s with the given [Transform]s.
    ///
    /// If this [Model] has multiple [Instance]s defined, all will be
    /// effectively removed with this.
    pub fn set_transforms(&mut self, transforms: Vec<Transform>) {
        self.transforms = transforms;
    }

    /// Sets a specific [Transform] on this [Model].
    /// Will **replace** the selected [Transform] with the given [Transform],
    /// if found.
    pub fn set_specific_transform(&mut self, transform: Transform, index: usize) {
        if let Some(model_transform) = self.transforms.get_mut(index) {
            *model_transform = transform;
        }
    }

    /// Adds one or many [Transform]_s_ to the [Model].
    /// Effectively, instancing the [Model].
    pub fn add_transform(&mut self, transform: Transform) {
        self.transforms.push(transform);
    }

    /// Removes a [Transform] from the [Model].
    ///
    /// ⚠️ Make sure at least one [Transform] is present!
    pub fn remove_transform(&mut self, index: usize) -> Option<Transform> {
        if index >= self.transforms.len() {
            return None; // Index out of bounds
        }

        Some(self.transforms.remove(index))
    }

    /// Applies the given [Transform] to the [Model].
    /// _All_ defined [Transform]s will be offset by the given
    /// [Transform].
    pub fn apply_transform(&mut self, mode: Mode<Transform>) {
        self.transforms.iter_mut().for_each(|x| match mode {
            Mode::Overwrite(transform) => *x = transform,
            Mode::Offset(transform)
            | Mode::OffsetViewAligned(transform)
            | Mode::OffsetViewAlignedWithY(transform) => x.apply_transform(transform),
        });
    }

    /// Applies the given [Transform] to the [Model] given a specific index for
    /// the [Transform] selection.
    /// _Only_ the defined [Transform] will be offset by the given
    /// [Transform].
    pub fn apply_transform_specific(&mut self, mode: Mode<Transform>, index: usize) {
        if let Some(model_transform) = self.transforms.get_mut(index) {
            match mode {
                Mode::Overwrite(transform) => *model_transform = transform,
                Mode::Offset(transform)
                | Mode::OffsetViewAligned(transform)
                | Mode::OffsetViewAlignedWithY(transform) => {
                    model_transform.apply_transform(transform)
                }
            }
        }
    }
}
