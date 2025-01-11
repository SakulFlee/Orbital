use bitmask_enum::bitmask;

#[bitmask]
#[derive(Default)]
pub enum RenderMode {
    /// Will render a given [`Model`](crate::resources::realizations::Model) in solid mode.
    /// _Solid_ rendering will **always fill-in** space between vertices.
    /// This is the default mode and should be fitting for most games.
    Solid,
    /// Will render a given [`Model`](crate::resources::realizations::Model) in wireframe mode.
    /// _Wireframe_ rendering will **only** render edges **without** filling in spaces between vertices.
    /// This is mainly useful for debugging.
    Wireframe,
}
