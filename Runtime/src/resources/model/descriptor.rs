use std::{collections::hash_map::DefaultHasher, hash::{Hash, Hasher}, sync::Arc};

use hashbrown::HashMap;
use ulid::Ulid;

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
    pub transforms: HashMap<Ulid, Transform>,
}

impl ModelDescriptor {
    /// Sets one or multiple [Transform]s for this [Model].
    /// Will **replace** _any_ [Transform]s with the given [Transform]s.
    ///
    /// If this [Model] has multiple [Instance]s defined, all will be
    /// effectively removed with this.
    pub fn set_transforms(&mut self, transforms: HashMap<Ulid, Transform>) {
        self.transforms = transforms;
    }

    /// Sets a specific [Transform] on this [Model].
    /// Will **replace** the selected [Transform] with the given [Transform],
    /// if found.
    pub fn set_specific_transform(&mut self, transform: Transform, ulid: Ulid) {
        if let Some(model_transform) = self.transforms.get_mut(&ulid) {
            *model_transform = transform;
        }
    }

    /// Adds a [Transform] to the [Model] with a new ULID.
    /// Effectively, instancing the [Model].
    pub fn add_transform(&mut self, transform: Transform) -> Ulid {
        let ulid = Ulid::new();
        self.transforms.insert(ulid, transform);
        ulid
    }

    /// Removes a [Transform] from the [Model].
    ///
    /// ⚠️ Make sure at least one [Transform] is present!
    pub fn remove_transform(&mut self, ulid: &Ulid) -> Option<Transform> {
        self.transforms.remove(ulid)
    }

    /// Applies the given [Transform] to the [Model].
    /// _All_ defined [Transform]s will be offset by the given
    /// [Transform].
    pub fn apply_transform(&mut self, mode: Mode<Transform>) {
        self.transforms.values_mut().for_each(|x| match mode {
            Mode::Overwrite(transform) => *x = transform.clone(),
            Mode::Offset(transform)
            | Mode::OffsetViewAligned(transform)
            | Mode::OffsetViewAlignedWithY(transform) => x.apply_transform(transform.clone()),
        });
    }

    /// Applies the given [Transform] to the [Model] given a specific ULID for
    /// the [Transform] selection.
    /// _Only_ the defined [Transform] will be offset by the given
    /// [Transform].
    pub fn apply_transform_specific(&mut self, mode: Mode<Transform>, ulid: &Ulid) {
        if let Some(model_transform) = self.transforms.get_mut(ulid) {
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

    /// Computes a hash for instance detection based on mesh and materials.
    pub fn instance_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();

        // Hash mesh vertices and indices
        self.mesh.vertices.hash(&mut hasher);
        self.mesh.indices.hash(&mut hasher);

        // Hash materials
        for material in &self.materials {
            material.hash(&mut hasher);
        }

        hasher.finish()
    }
}
