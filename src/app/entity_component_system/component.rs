use std::any::Any;

pub trait Component {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

macro_rules! component {
    ($struct_name:ident) => {
        impl crate::app::Component for $struct_name {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }
    };
}
pub(crate) use component;

macro_rules! component_default {
    ($name: ident) => {
        (
            stringify!($name).to_string(),
            std::boxed::Box::new(crate::entities_components::components::$name::default()),
        )
    };
}
pub(crate) use component_default;

macro_rules! component_value {
    ($name: ident, $value: expr) => {
        (
            stringify!($name).to_string(),
            std::boxed::Box::new(crate::entities_components::components::$name::new($value)),
        )
    };
}
pub(crate) use component_value;
