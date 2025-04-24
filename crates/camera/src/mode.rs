/// Defines how `<T>` will be used.  
/// This is mainly intended to be used with _positions_ (e.g. [Vector3<f32>])
/// and [Camera]s. The following documentation will be based around this assumption, but this may also be used elsewhere.
///
/// There are three modes:
///
/// - [PositionMode::Overwrite]: Directly overwrites the position in question.  
///     Does not offset or figure out view angles.
/// - [PositionMode::Offset]: Offsets the current location by the given amount.
///     Does not figure out view angles and offset according to the view angle.
/// - [PositionMode::OffsetViewAligned]: Checks where "Forward" is based on
///     where the [Camera] is currently looking at and offsets the current
///     location by the supplied amount, where "forward" will be equal to where
///     the [Camera] is looking at. Only valid if the current view angle is
///     known (e.g. by a [Camera]).
///     If not known, will default back to [PositionMode::Offset].
///
/// # When to use what?
///
/// Use [PositionMode::Overwrite] when:  
/// - You need to set the position of something, like the [Camera],
///     directly to a specific location in the world.
/// - You need to "teleport" something and thus set the position to a new
///     location without needing to figure out an offset.
///
/// Use [PositionMode::Offset] when:  
/// - You need to _offset_ a position by a certain amount.
/// - Ideal for most kinds of 3rd-person [Camera]s like top-down!
///
/// Use [PositionMode::OffsetViewAligned] when:  
/// - You need to _offset_ a position by a certain amount, following where the
///     [Camera] is looking at.
/// - Ideal for any kind of 1st-person [Camera]!
///
/// [Camera]: crate::resources::realizations::Camera
/// [Vector3<f32>]: crate::cgmath::Vector3
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
    ///
    /// Sometimes the same as [Mode::OffsetViewAligned] or [Mode::OffsetViewAlignedWithY]!
    Offset(T),
    /// Same as [Mode::Offset], but will take the current view angle into account.
    /// Effectively making it so that the offset is always "forward" aligned.
    ///
    /// When used with a position for changing the camera, this is often referred to as "forward movement".
    ///
    /// Sometimes the same as [Mode::OffsetViewAligned] or [Mode::OffsetViewAlignedWithY]!
    OffsetViewAligned(T),
    /// Same as [Mode::OffsetViewAligned], but will also offset the Y-axis.
    ///
    /// When used with a position for changing the camera, this is often referred to as "free cam".
    ///
    /// Sometimes the same as [Mode::OffsetViewAligned] or [Mode::OffsetViewAlignedWithY]!
    OffsetViewAlignedWithY(T),
}
