use std::cmp::Ordering;

use cgmath::{Point3, Vector3};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BoundingBox<T> {
    pub a: Point3<T>,
    pub b: Point3<T>,
}
