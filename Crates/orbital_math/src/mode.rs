/// Defines how `<T>` will be used.
/// This is mainly intended to be used with _positions_ (e.g. `cgmath::Vector3<f32>`)
/// and cameras. The following documentation will be based around this assumption, but this may also be used elsewhere.
///
/// There are four modes:
///
/// - `Mode::Overwrite`: Directly overwrites the value in question.
///     Does not offset or figure out view angles.
/// - `Mode::Offset`: Offsets the current value by the given amount.
///     Does not figure out view angles and offset according to the view angle.
/// - `Mode::OffsetViewAligned`: Checks where "Forward" is based on
///     where the camera is currently looking at and offsets the current
///     value by the supplied amount, where "forward" will be equal to where
///     the camera is looking at.
/// - `Mode::OffsetViewAlignedWithY`: Same as `OffsetViewAligned`, but will also offset the Y-axis.
///
/// # When to use what?
///
/// Use `Mode::Overwrite` when:
/// - You need to set the position of something, like a camera,
///     directly to a specific location in the world.
/// - You need to "teleport" something and thus set the position to a new
///     location without needing to figure out an offset.
///
/// Use `Mode::Offset` when:
/// - You need to _offset_ a position by a certain amount.
/// - Ideal for most kinds of 3rd-person cameras like top-down!
///
/// Use `Mode::OffsetViewAligned` when:
/// - You need to _offset_ a position by a certain amount, following where the
///     camera is looking at.
/// - Ideal for any kind of 1st-person camera!
#[derive(Debug)]
pub enum Mode<T> {
    /// Will overwrite the inner value.
    /// E.g. if used for positions, will fully replace the existing position with the inner position.
    ///
    /// Current position: (0, 1, 2)
    /// Inner/New position: (5, 5, 5)
    /// After change: (5, 5, 5)
    ///
    /// When used with a position for changing the camera, this is ideal for teleporting the camera to a new location.
    Overwrite(T),
    /// Will offset the existing value by the inner value.
    /// E.g. if used for positions, will add the inner position to the existing position.
    ///
    /// Current position: (0, 1, 2)
    /// Inner/New position: (5, 5, 5)
    /// After change: (0 + 5, 1 + 5, 2 + 5)
    ///            == (5, 6, 7)
    ///
    /// When used with a position for changing the camera, this is ideal for "top down" cameras.
    Offset(T),
    /// Same as `Mode::Offset`, but will take the current view angle into account.
    /// Effectively making it so that the offset is always "forward" aligned.
    ///
    /// When used with a position for changing the camera, this is often referred to as "forward movement".
    OffsetViewAligned(T),
    /// Same as `Mode::OffsetViewAligned`, but will also offset the Y-axis.
    ///
    /// When used with a position for changing the camera, this is often referred to as "free cam".
    OffsetViewAlignedWithY(T),
}
