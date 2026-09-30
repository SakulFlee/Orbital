//! Making component values editable from the debug UI.
//!
//! Display is generic: any `Component` is `Debug`, so the inspector can render
//! any value. Editing cannot be — turning `"1.5"` back into a `T` needs to know
//! `T`. This module closes that gap for the types a game is most likely to
//! want to tweak live, and leaves everything else read-only.

use std::any::TypeId;

/// A component value that can be shown in, and written back from, a text field.
pub trait EditableValue {
    /// The value as it should appear in the editor.
    fn to_edit_string(&self) -> String;

    /// Parses `input` and replaces the value.
    ///
    /// Returns a human-readable message on failure, which the panel shows
    /// instead of applying anything. On failure the value must be left
    /// untouched.
    fn set_from_str(&mut self, input: &str) -> Result<(), String>;
}

// ---------------------------------------------------------------------------
// Scalars
// ---------------------------------------------------------------------------

impl EditableValue for f64 {
    fn to_edit_string(&self) -> String {
        // `{:?}` round-trips a float exactly, so what is shown is what will be
        // parsed back.
        format!("{self:?}")
    }

    fn set_from_str(&mut self, input: &str) -> Result<(), String> {
        *self = input
            .trim()
            .parse()
            .map_err(|_| format!("`{input}` is not a number"))?;
        Ok(())
    }
}

/// Implements [`EditableValue`] for an integer type, range-checked.
///
/// Parsing through `f64` and then `as`-casting would be simpler but is wrong:
/// a Rust float-to-int cast *saturates*, so typing `-1` into a `u32` field
/// would silently write 0 instead of reporting an error.
macro_rules! editable_integer {
    ($($ty:ty),* $(,)?) => {$(
        impl EditableValue for $ty {
            fn to_edit_string(&self) -> String {
                self.to_string()
            }

            fn set_from_str(&mut self, input: &str) -> Result<(), String> {
                let trimmed = input.trim();
                let parsed: f64 = trimmed
                    .parse()
                    .map_err(|_| format!("`{trimmed}` is not a number"))?;

                if !parsed.is_finite() || parsed.fract() != 0.0 {
                    return Err(format!("`{trimmed}` is not a whole number"));
                }

                // Compared against the widened bounds of the target type, so
                // this works for every width without a per-type literal.
                if parsed < <$ty>::MIN as f64 || parsed > <$ty>::MAX as f64 {
                    return Err(format!("`{trimmed}` is out of range for {}", stringify!($ty)));
                }

                *self = parsed as $ty;
                Ok(())
            }
        }
    )*};
}

editable_integer!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);

impl EditableValue for f32 {
    fn to_edit_string(&self) -> String {
        // `{:?}` round-trips a float exactly.
        format!("{self:?}")
    }

    fn set_from_str(&mut self, input: &str) -> Result<(), String> {
        *self = input
            .trim()
            .parse()
            .map_err(|_| format!("`{input}` is not a float"))?;
        Ok(())
    }
}

impl EditableValue for bool {
    fn to_edit_string(&self) -> String {
        self.to_string()
    }

    fn set_from_str(&mut self, input: &str) -> Result<(), String> {
        match input.trim().to_ascii_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => *self = true,
            "false" | "0" | "no" | "off" => *self = false,
            _ => return Err("expected true or false".to_string()),
        }
        Ok(())
    }
}

impl EditableValue for String {
    fn to_edit_string(&self) -> String {
        self.clone()
    }

    fn set_from_str(&mut self, input: &str) -> Result<(), String> {
        *self = input.to_string();
        Ok(())
    }
}

impl<T: EditableValue> EditableValue for Option<T> {
    fn to_edit_string(&self) -> String {
        self.as_ref()
            .map_or_else(|| "-".to_string(), EditableValue::to_edit_string)
    }

    fn set_from_str(&mut self, input: &str) -> Result<(), String> {
        if input.trim() == "-" {
            *self = None;
            return Ok(());
        }

        let Some(inner) = self.as_mut() else {
            return Err("cannot set a value on `None`".to_string());
        };

        inner.set_from_str(input)
    }
}

// ---------------------------------------------------------------------------
// cgmath vectors and points
// ---------------------------------------------------------------------------

/// Implements [`EditableValue`] for a `cgmath` vector of `N` components,
/// formatted as space-separated numbers.
macro_rules! editable_cgmath_vector {
    ($($ty:ty),* $(,)?) => {$(
        impl EditableValue for $ty {
            fn to_edit_string(&self) -> String {
                Self::components(self)[..Self::COUNT]
                    .iter()
                    .map(|component| format!("{component:?}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            }

            fn set_from_str(&mut self, input: &str) -> Result<(), String> {
                let values = parse_f32_list(input, Self::COUNT)?;

                // The array is padded to the widest editable type, so one
                // generated impl serves 2-, 3- and 4-component vectors alike.
                let mut padded = [0.0f32; 4];
                padded[..Self::COUNT].copy_from_slice(&values);

                Self::set_components(self, padded);

                Ok(())
            }
        }
    )*};
}

/// Uniform component access for the editable cgmath types.
///
/// A trait because the types come from cgmath and cannot carry inherent
/// helpers. The array is always 4 wide and padded, so a single generated impl
/// can serve 2-, 3- and 4-component types.
trait ComponentCount {
    /// How many components are meaningful; the rest of the array is padding.
    const COUNT: usize;

    fn components(&self) -> [f32; 4];

    fn set_components(&mut self, values: [f32; 4]);
}

/// Builds a two-component `ComponentCount` impl from `x` and `y` fields.
macro_rules! component_count_2 {
    ($ty:ty, $x:ident, $y:ident) => {
        impl ComponentCount for $ty {
            const COUNT: usize = 2;

            fn components(&self) -> [f32; 4] {
                [self.$x, self.$y, 0.0, 0.0]
            }

            fn set_components(&mut self, values: [f32; 4]) {
                self.$x = values[0];
                self.$y = values[1];
            }
        }
    };
}

component_count_2!(cgmath::Vector2<f32>, x, y);
component_count_2!(cgmath::Point2<f32>, x, y);

impl ComponentCount for cgmath::Vector3<f32> {
    const COUNT: usize = 3;

    fn components(&self) -> [f32; 4] {
        [self.x, self.y, self.z, 0.0]
    }

    fn set_components(&mut self, values: [f32; 4]) {
        self.x = values[0];
        self.y = values[1];
        self.z = values[2];
    }
}

impl ComponentCount for cgmath::Point3<f32> {
    const COUNT: usize = 3;

    fn components(&self) -> [f32; 4] {
        [self.x, self.y, self.z, 0.0]
    }

    fn set_components(&mut self, values: [f32; 4]) {
        self.x = values[0];
        self.y = values[1];
        self.z = values[2];
    }
}

impl ComponentCount for cgmath::Vector4<f32> {
    const COUNT: usize = 4;

    fn components(&self) -> [f32; 4] {
        [self.x, self.y, self.z, self.w]
    }

    fn set_components(&mut self, values: [f32; 4]) {
        self.x = values[0];
        self.y = values[1];
        self.z = values[2];
        self.w = values[3];
    }
}

editable_cgmath_vector!(
    cgmath::Vector2<f32>,
    cgmath::Vector3<f32>,
    cgmath::Vector4<f32>,
    cgmath::Point2<f32>,
    cgmath::Point3<f32>,
);

/// Splits `input` into exactly `expected` floats.
fn parse_f32_list(input: &str, expected: usize) -> Result<Vec<f32>, String> {
    let values = input
        .split([' ', ','])
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| {
            part.parse::<f32>()
                .map_err(|_| format!("`{part}` is not a number"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    if values.len() != expected {
        return Err(format!("expected {expected} numbers, got {}", values.len()));
    }

    Ok(values)
}

// ---------------------------------------------------------------------------
// cgmath quaternion, degree and radian
// ---------------------------------------------------------------------------

impl EditableValue for cgmath::Quaternion<f32> {
    fn to_edit_string(&self) -> String {
        format!("{:?} {:?} {:?} {:?}", self.s, self.v.x, self.v.y, self.v.z)
    }

    fn set_from_str(&mut self, input: &str) -> Result<(), String> {
        let values = parse_f32_list(input, 4)?;

        self.s = values[0];
        self.v.x = values[1];
        self.v.y = values[2];
        self.v.z = values[3];

        Ok(())
    }
}

impl EditableValue for cgmath::Deg<f32> {
    fn to_edit_string(&self) -> String {
        format!("{:?}", self.0)
    }

    fn set_from_str(&mut self, input: &str) -> Result<(), String> {
        self.0 = input
            .trim()
            .parse()
            .map_err(|_| format!("`{input}` is not a number"))?;
        Ok(())
    }
}

impl EditableValue for cgmath::Rad<f32> {
    fn to_edit_string(&self) -> String {
        format!("{:?}", self.0)
    }

    fn set_from_str(&mut self, input: &str) -> Result<(), String> {
        self.0 = input
            .trim()
            .parse()
            .map_err(|_| format!("`{input}` is not a number"))?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Engine types
// ---------------------------------------------------------------------------

impl EditableValue for orbital_ecs_bridge::Position {
    fn to_edit_string(&self) -> String {
        self.0.to_edit_string()
    }

    fn set_from_str(&mut self, input: &str) -> Result<(), String> {
        self.0.set_from_str(input)
    }
}

impl EditableValue for orbital_ecs_bridge::Rotation {
    fn to_edit_string(&self) -> String {
        self.0.to_edit_string()
    }

    fn set_from_str(&mut self, input: &str) -> Result<(), String> {
        self.0.set_from_str(input)
    }
}

// ---------------------------------------------------------------------------
// The type registry
// ---------------------------------------------------------------------------

/// How to read and write values of one component type.
pub struct EditableOps {
    /// Formats a value for the text field.
    pub display: fn(&dyn std::any::Any) -> String,
    /// Parses a field's text and writes it into a value.
    pub apply: fn(&mut dyn std::any::Any, &str) -> Result<(), String>,
}

/// The operations for the type identified by `type_id`, or `None` if it has no
/// [`EditableValue`] impl and so is read-only in the inspector.
///
/// Both halves come from the same `TypeId` arm, so a type can never get a
/// display form without a matching parser. The match *is* the registry: there is
/// no second list to keep in step with the impls above.
pub fn editable_ops(type_id: TypeId) -> Option<EditableOps> {
    macro_rules! ops {
        ($($ty:ty),* $(,)?) => {$(
            if type_id == TypeId::of::<$ty>() {
                return Some(EditableOps {
                    display: display::<$ty>,
                    apply: apply::<$ty>,
                });
            }
        )*};
    }

    ops!(
        f32,
        f64,
        i8,
        i16,
        i32,
        i64,
        isize,
        u8,
        u16,
        u32,
        u64,
        usize,
        bool,
        String,
        cgmath::Vector2<f32>,
        cgmath::Vector3<f32>,
        cgmath::Vector4<f32>,
        cgmath::Point2<f32>,
        cgmath::Point3<f32>,
        cgmath::Quaternion<f32>,
        cgmath::Deg<f32>,
        cgmath::Rad<f32>,
        orbital_ecs_bridge::Position,
        orbital_ecs_bridge::Rotation,
    );

    None
}

/// The text-field form of a value, if its type is editable.
pub fn display_for(type_id: TypeId, value: &dyn std::any::Any) -> Option<String> {
    Some((editable_ops(type_id)?.display)(value))
}

/// Writes `input` into a value of the type identified by `type_id`.
pub fn apply_to(type_id: TypeId, value: &mut dyn std::any::Any, input: &str) -> Result<(), String> {
    let ops =
        editable_ops(type_id).ok_or_else(|| "this component type is not editable".to_string())?;
    (ops.apply)(value, input)
}

fn display<T: EditableValue + 'static>(value: &dyn std::any::Any) -> String {
    value
        .downcast_ref::<T>()
        .map(EditableValue::to_edit_string)
        .unwrap_or_else(|| "<type mismatch>".to_string())
}

fn apply<T: EditableValue + 'static>(
    value: &mut dyn std::any::Any,
    input: &str,
) -> Result<(), String> {
    value
        .downcast_mut::<T>()
        .ok_or_else(|| "component type changed under the inspector".to_string())?
        .set_from_str(input)
}

/// A short, human-readable name for each editable type.
///
/// Purely cosmetic, so it is a plain list; a unit test checks every entry
/// resolves through [`editable_ops`] too.
pub fn editable_label(type_id: TypeId) -> Option<&'static str> {
    let labels: [(&str, TypeId); 12] = [
        ("f32", TypeId::of::<f32>()),
        ("f64", TypeId::of::<f64>()),
        ("i32", TypeId::of::<i32>()),
        ("i64", TypeId::of::<i64>()),
        ("u32", TypeId::of::<u32>()),
        ("u64", TypeId::of::<u64>()),
        ("bool", TypeId::of::<bool>()),
        ("String", TypeId::of::<String>()),
        ("Vector3<f32>", TypeId::of::<cgmath::Vector3<f32>>()),
        ("Quaternion<f32>", TypeId::of::<cgmath::Quaternion<f32>>()),
        ("Position", TypeId::of::<orbital_ecs_bridge::Position>()),
        ("Rotation", TypeId::of::<orbital_ecs_bridge::Rotation>()),
    ];

    labels
        .into_iter()
        .find(|(_, id)| *id == type_id)
        .map(|(name, _)| name)
}

/// Whether values of the type identified by `type_id` can be edited.
pub fn is_editable(type_id: TypeId) -> bool {
    editable_ops(type_id).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalars_round_trip_exactly() {
        let mut value = 0.0f32;
        value.set_from_str("0.1").expect("parse failed");
        assert_eq!(value, 0.1f32);
        // `{:?}` is the shortest representation that round-trips.
        assert_eq!(value.to_edit_string(), "0.1");
    }

    #[test]
    fn integers_accept_whole_floats_but_not_fractions() {
        let mut value = 0i32;
        assert!(value.set_from_str("2.0").is_ok());
        assert_eq!(value, 2);

        let mut value = 0i32;
        assert!(value.set_from_str("2.5").is_err(), "fractional");
        assert_eq!(value, 0);
    }

    #[test]
    fn integers_reject_out_of_range_without_saturating() {
        // A float-to-int `as` cast saturates, so `-1 as u32` is 0. The parser
        // has to range-check or a typo would silently write the wrong value.
        let mut value = 7u32;
        assert!(value.set_from_str("-1").is_err(), "negative into unsigned");
        assert_eq!(value, 7, "value untouched after failure");

        let mut value = 7i8;
        assert!(value.set_from_str("200").is_err(), "above i8::MAX");
        assert_eq!(value, 7);

        let mut value = 0u64;
        assert!(value.set_from_str("1e30").is_err(), "far past u64::MAX");
        assert_eq!(value, 0);

        let mut value = 0i64;
        assert!(value.set_from_str("inf").is_err());
        assert!(value.set_from_str("nan").is_err());
    }

    #[test]
    fn bool_accepts_the_usual_spellings() {
        for input in ["true", "TRUE", "1", "yes", "on"] {
            let mut value = false;
            value.set_from_str(input).expect("parse failed");
            assert!(value, "input {input}");
        }

        for input in ["false", "0", "no", "off"] {
            let mut value = true;
            value.set_from_str(input).expect("parse failed");
            assert!(!value, "input {input}");
        }

        let mut value = true;
        assert!(value.set_from_str("maybe").is_err());
        assert!(value, "value untouched after failure");
    }

    #[test]
    fn vectors_use_space_separated_components() {
        let mut value = cgmath::Vector3::<f32>::new(1.0, 2.0, 3.0);
        assert_eq!(value.to_edit_string(), "1.0 2.0 3.0");

        value.set_from_str("4 5 6").expect("parse failed");
        assert_eq!(value, cgmath::Vector3::new(4.0, 5.0, 6.0));

        // Commas work too.
        value.set_from_str("7, 8, 9").expect("parse failed");
        assert_eq!(value.x, 7.0);
    }

    #[test]
    fn wrong_component_count_is_rejected_without_mutating() {
        let mut value = cgmath::Vector3::<f32>::new(1.0, 2.0, 3.0);
        let before = value;

        assert!(value.set_from_str("1 2").is_err());
        assert_eq!(value, before, "value untouched after failure");

        assert!(value.set_from_str("1 2 x").is_err());
        assert_eq!(value, before);
    }

    #[test]
    fn quaternions_are_wxyz() {
        let mut value = cgmath::Quaternion::<f32>::new(1.0, 0.0, 0.0, 0.0);
        assert_eq!(
            value.to_edit_string(),
            "1.0 0.0 0.0 0.0",
            "same format as vectors"
        );

        value.set_from_str("0 0.5 0.5 0.5").expect("parse failed");
        assert_eq!(value.s, 0.0);
        assert_eq!(value.v.y, 0.5);
    }

    #[test]
    fn option_round_trips_through_a_dash() {
        let mut value: Option<f32> = Some(2.0);
        assert_eq!(value.to_edit_string(), "2.0");

        value.set_from_str("-").expect("parse failed");
        assert_eq!(value, None);
        assert_eq!(value.to_edit_string(), "-");

        assert!(
            value.set_from_str("3").is_err(),
            "cannot fill in a None that is currently None"
        );
    }

    #[test]
    fn engine_components_delegate_to_their_inner_types() {
        let mut position = orbital_ecs_bridge::Position(cgmath::Point3::new(1.0, 2.0, 3.0));
        assert_eq!(position.to_edit_string(), "1.0 2.0 3.0");

        position.set_from_str("0 0 0").expect("parse failed");
        assert_eq!(position.0, cgmath::Point3::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn the_registry_knows_which_types_are_editable() {
        assert!(is_editable(TypeId::of::<f32>()));
        assert!(is_editable(TypeId::of::<cgmath::Vector3<f32>>()));
        assert!(is_editable(TypeId::of::<orbital_ecs_bridge::Position>()));

        // A type with no impl must not claim to be editable.
        assert!(!is_editable(TypeId::of::<orbital_ecs_bridge::CameraDirty>()));
    }

    #[test]
    fn every_labelled_type_really_has_an_editor() {
        // A label without an editor would show a field that cannot be saved.
        for (name, type_id) in [
            ("f32", TypeId::of::<f32>()),
            ("f64", TypeId::of::<f64>()),
            ("i32", TypeId::of::<i32>()),
            ("i64", TypeId::of::<i64>()),
            ("u32", TypeId::of::<u32>()),
            ("u64", TypeId::of::<u64>()),
            ("bool", TypeId::of::<bool>()),
            ("String", TypeId::of::<String>()),
            ("Vector3<f32>", TypeId::of::<cgmath::Vector3<f32>>()),
            ("Quaternion<f32>", TypeId::of::<cgmath::Quaternion<f32>>()),
            ("Position", TypeId::of::<orbital_ecs_bridge::Position>()),
            ("Rotation", TypeId::of::<orbital_ecs_bridge::Rotation>()),
        ] {
            assert_eq!(editable_label(type_id), Some(name));
            assert!(is_editable(type_id), "{name} is labelled but not editable");
        }
    }
}
