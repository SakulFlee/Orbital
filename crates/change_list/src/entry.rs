use crate::{action::ChangeListAction, ty::ChangeListType};

#[derive(Debug)]
pub struct ChangeListEntry {
    pub change_type: ChangeListType,
    pub action: ChangeListAction,
}
