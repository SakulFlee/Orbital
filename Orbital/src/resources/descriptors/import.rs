/// Union type used for imports.
/// Defines which things should be imported based on an index.
#[derive(Debug, Clone, PartialEq)]
pub enum ImportDescriptor {
    /// A simple numerical index starting at 0.
    /// Should always work, but the order of indices may change between exports!
    Index(u32),
    /// A name field index.
    /// ⚠️ Only works if the import has tags!
    Name(&'static str),
}
