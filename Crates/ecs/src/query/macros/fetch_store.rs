#[macro_export]
macro_rules! fetch_store {
    (Read, $world:expr, $T:ty) => {
        $world.get_component_store::<$T>()
    };
    (Write, $world:expr, $T:ty) => {
        $world.get_component_store_mut::<$T>()
    };
}
