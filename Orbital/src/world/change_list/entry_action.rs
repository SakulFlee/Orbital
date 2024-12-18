#[derive(Debug)]
pub enum EntryAction {
    Added,
    Removed,
    Changed,
    /// Completely clears the entry type.
    /// Use an empty string for the label.    
    Clear,
}
