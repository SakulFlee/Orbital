/// Frequency of updates.
/// There are two types:
///
/// **Fast**, means the update function of the [`TEntity`] is getting called
/// **every cycle**. I.e. Cycle-Count/UPS -per second.
/// This is most likely what you want for e.g. input checking.
///
/// **Slow**, means the update function gets called roughly every second.
/// This is most likely used for slower checks, e.g. quest progression.
///
/// > **Note**: If **Slow** is used and input is checked, you may run into
/// > issues where the player **did** press a key for a **short-time**, but
/// > by the time the update function gets called the given key was already
/// > released again. In which case the input would simply read as
/// > "never pressed"!
/// >
/// > Similar timings may occur elsewhere.
///
/// Additionally, there is **None**.
/// **None** is used in case a [`TEntity`] doesn't need update calls.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UpdateFrequency {
    /// -> Update function of [`TEntity`] gets called per-cycle
    Fast,
    /// -> Update function of [`TEntity`] gets called per-second (roughly!)
    Slow,
    /// -> Update function of [`TEntity`] should never get called
    None,
}
