#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EntityTagDuplicationBehaviour {
    AllowDuplication,
    WarnOnDuplication,
    PanicOnDuplication,
    IgnoreEntityOnDuplication,
    OverwriteEntityOnDuplication,
}
