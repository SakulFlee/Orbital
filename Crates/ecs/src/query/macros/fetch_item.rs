#[macro_export]
macro_rules! fetch_item {
    (Read, $handle:expr, $idx:expr) => {
        &$handle.components[$idx]
    };
    (Write, $handle:expr, $idx:expr) => {
        &mut $handle.get_mut_store().components[$idx]
    };
}
