#[derive(Debug)]
pub enum ShaderPreprocessorError {
    /// A requested node name is not registered in the [`crate::NodeRegistry`].
    UnknownNode {
        name: String,
    },
    /// Two distinct nodes were registered under the same name.
    ConflictingNodeName {
        name: String,
    },
    /// Resolving a node followed a circular dependency chain.
    DependencyCycle {
        name: String,
    },
    Fs(orbital_file_manager::FsError),
}
