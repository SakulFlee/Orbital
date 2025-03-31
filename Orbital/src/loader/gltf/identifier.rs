#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum GLTFIdentifier {
    /// The numerical id, starting at zero, of the location of the object
    /// inside the glTF file.  
    /// Indices cannot be skipped and the order will be preserved.
    /// Thus, the first object will always be the 0th index.
    Id(usize),
    /// Label/Name of the object inside the glTF file.
    ///
    /// ⚠️ Labels in glTF files are an optional feature and must be  
    ///     supported by the glTF file / exporter.
    ///
    /// Less performant than [GLTFIdentifier::Id] as it needs to
    /// search through all entries until the label is found or no
    /// more objects are to be inspected.  
    /// However, if used in a [Loader], performance can be ignored.
    Label(&'static str),
}

impl GLTFIdentifier {
    pub fn ranged_id(start: usize, end: usize) -> Vec<Self> {
        if start > end {
            panic!("Ranged start cannot be bigger than end!");
        }

        let mut v = Vec::new();

        for i in start..=end {
            v.push(GLTFIdentifier::Id(i));
        }

        v
    }
}
