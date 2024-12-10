use super::{EntryAction, EntryType};

#[derive(Debug)]
pub struct ChangeListEntry {
    pub action: EntryAction,
    pub ty: EntryType,
    pub label: String,
}
