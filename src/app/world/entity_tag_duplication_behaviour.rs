pub enum EntityTagDuplicationBehaviour {
    AllowDuplication,
    WarnOnDuplication,
    PanicOnDuplication,
    IgnoreEntityOnDuplication,
    OverwriteEntityOnDuplication,
}
