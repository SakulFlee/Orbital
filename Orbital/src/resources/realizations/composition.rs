use wgpu::{Device, Queue};

use crate::{
    error::Error,
    resources::{
        descriptors::{CompositionDescriptor, ImportDescriptor, ModelDescriptor},
        realizations::Instance,
    },
};

use super::Model;

#[derive(Default)]
pub struct Composition {
    models: Vec<Model>,
}

impl Composition {
    /// Creates a composition from a [CompositionDescriptor].
    ///
    /// # Returns
    /// Either, a [Composition] with all models loaded, or, the first [Error] found.
    pub fn from_descriptor(
        descriptor: &CompositionDescriptor,
        device: &Device,
        queue: &Queue,
    ) -> Result<Self, Error> {
        match descriptor {
            CompositionDescriptor::FromDescriptors(model_descriptors) => {
                Self::from_descriptors(model_descriptors, device, queue)
            }
            #[cfg(feature = "gltf")]
            CompositionDescriptor::FromGLTF(path, import_descriptor) => {
                Self::from_gltf(path, import_descriptor, device, queue)
            }
        }
    }

    /// Creates a composition from multiple [ModelDescriptor]s.
    ///
    /// # Returns
    /// Either, a [Composition] with all models loaded, or, the first [Error] found.
    pub fn from_descriptors(
        descriptors: &[ModelDescriptor],
        device: &Device,
        queue: &Queue,
    ) -> Result<Self, Error> {
        let mut iter = descriptors
            .iter()
            .map(|x| Model::from_descriptor(x, device, queue));

        // If any error is found, return it.
        if let Some(x) = iter.find(|x| x.is_err()) {
            x?;
        };

        // No errors should be in iter, unwrap all
        Ok(Self {
            models: iter.map(|x| x.unwrap()).collect(),
        })
    }

    /// Creates a composition from a _glTF file_.
    ///
    /// # Returns
    /// Either, a [Composition] with all models loaded, or, the first [Error] found.
    #[cfg(feature = "gltf")]
    fn from_gltf(
        path: &str,
        import_descriptor: &ImportDescriptor,
        device: &Device,
        queue: &Queue,
    ) -> Result<Self, Error> {
        // Load glTF file
        let gltf_file = easy_gltf::load(path).map_err(|e| Error::GltfError(e))?;

        // Query for scene. If found we continue.
        let scene = if let Some(scene) = match import_descriptor {
            ImportDescriptor::Index(i) => gltf_file.get(*i as usize),
            ImportDescriptor::Name(name) => gltf_file
                .iter()
                .find(|x| x.name.is_some() && x.name.as_ref().unwrap() == *name),
        } {
            scene
        } else {
            return Err(Error::SceneNotFound);
        };

        let mut models = Vec::<Model>::new();
        for gltf_model in &scene.models {
            match Model::from_gltf_model(gltf_model, vec![Instance::default()], device, queue) {
                Ok(model) => models.push(model),
                // Return the first error that occurs
                Err(e) => return Err(e),
            }
        }

        Ok(Self { models })
    }

    pub fn add_model(&mut self, model: Model) {
        self.models.push(model);
    }

    pub fn models(&self) -> &[Model] {
        &self.models
    }

    pub fn size(&self) -> usize {
        self.models.len()
    }

    pub fn is_empty(&self) -> bool {
        self.models.is_empty()
    }
}

pub type Scene = Composition;
