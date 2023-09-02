use super::{renderable::Renderable, updateable::Updateable};

pub trait Object: Updateable + Renderable {}
