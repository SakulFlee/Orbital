#[macro_export]
macro_rules! fetch_item_ty {
    (Read, $l:lifetime, $T:ty) => {
        & $l $T
    };
    (Write, $l:lifetime, $T:ty) => {
        & $l mut $T
    };
}
