#[macro_export]
macro_rules! fetch_state_ty {
    (Read, $l:lifetime, $T:ty) => {
        Option<crate::ReadStoreHandle<$l, $T>>
    };
    (Write, $l:lifetime, $T:ty) => {
        Option<crate::WriteStoreHandle<$l, $T>>
    };
}
