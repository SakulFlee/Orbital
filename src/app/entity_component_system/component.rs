use std::any::{Any, TypeId};

pub trait Component {
    fn type_id(&self) -> TypeId {
        self.as_any().type_id()
    }

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

macro_rules! impl_component {
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
pub(crate) use impl_component;

macro_rules! component {
    ($name: expr, $value: expr) => {
        ($name, std::boxed::Box::new($value))
    };
}
pub(crate) use component;
