use super::ChangeType;

#[derive(Debug)]
pub enum Change {
    Clear(ChangeType),
    Added(ChangeType),
    Removed(ChangeType),
    Changed(ChangeType),
}
