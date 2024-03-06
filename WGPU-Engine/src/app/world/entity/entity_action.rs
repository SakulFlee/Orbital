use std::mem::discriminant;

use wgpu::Color;

use crate::engine::CameraChange;

use super::BoxedEntity;

pub enum EntityAction {
    ClearColorAdjustment(Color),
    Spawn(Vec<BoxedEntity>),
    Remove(Vec<String>),
    CameraChange(CameraChange),
    Keep,
}

impl PartialEq for EntityAction {
    fn eq(&self, other: &Self) -> bool {
        discriminant(self) == discriminant(other)
    }
}
