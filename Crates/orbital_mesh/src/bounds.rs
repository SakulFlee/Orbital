use cgmath::{EuclideanSpace, InnerSpace, Point3, Vector3};

#[derive(Debug, Clone, PartialEq)]
pub struct BoundingSphere {
    pub center: Point3<f32>,
    pub radius: f32,
}

/// Axis-aligned local-space bounds of a mesh.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    pub min: Point3<f32>,
    pub max: Point3<f32>,
}

impl BoundingBox {
    pub fn compute_from_positions(positions: &[Vector3<f32>]) -> Option<Self> {
        let first = positions.first()?;
        let mut min = *first;
        let mut max = *first;

        for position in &positions[1..] {
            min = Vector3::new(
                min.x.min(position.x),
                min.y.min(position.y),
                min.z.min(position.z),
            );
            max = Vector3::new(
                max.x.max(position.x),
                max.y.max(position.y),
                max.z.max(position.z),
            );
        }

        Some(Self {
            min: Point3::from_vec(min),
            max: Point3::from_vec(max),
        })
    }

    /// The eight local-space corners, in the order used by
    /// [`Self::edge_indices`].
    pub fn corners(&self) -> [Point3<f32>; 8] {
        [
            Point3::new(self.min.x, self.min.y, self.min.z),
            Point3::new(self.max.x, self.min.y, self.min.z),
            Point3::new(self.max.x, self.max.y, self.min.z),
            Point3::new(self.min.x, self.max.y, self.min.z),
            Point3::new(self.min.x, self.min.y, self.max.z),
            Point3::new(self.max.x, self.min.y, self.max.z),
            Point3::new(self.max.x, self.max.y, self.max.z),
            Point3::new(self.min.x, self.max.y, self.max.z),
        ]
    }

    /// The twelve edges as pairs of corner indices.
    pub const fn edge_indices() -> [(usize, usize); 12] {
        [
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 0),
            (4, 5),
            (5, 6),
            (6, 7),
            (7, 4),
            (0, 4),
            (1, 5),
            (2, 6),
            (3, 7),
        ]
    }
}

impl BoundingSphere {
    pub fn compute_from_positions(positions: &[Vector3<f32>]) -> Option<Self> {
        if positions.is_empty() {
            return None;
        }

        let inv_n = 1.0 / positions.len() as f32;
        let center = positions
            .iter()
            .fold(Vector3::new(0.0, 0.0, 0.0), |acc, p| acc + *p)
            * inv_n;

        let radius = positions.iter().fold(0.0f32, |max_dist, p| {
            let dist = (*p - center).magnitude();
            max_dist.max(dist)
        });

        Some(BoundingSphere {
            center: Point3::from_vec(center),
            radius,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounding_box_tracks_min_and_max() {
        let bounds = BoundingBox::compute_from_positions(&[
            Vector3::new(-1.0, 2.0, 0.5),
            Vector3::new(4.0, -3.0, 2.0),
            Vector3::new(0.0, 0.0, 0.0),
        ])
        .expect("positions are non-empty");

        assert_eq!(bounds.min, Point3::new(-1.0, -3.0, 0.0));
        assert_eq!(bounds.max, Point3::new(4.0, 2.0, 2.0));
        assert_eq!(bounds.corners().len(), 8);
        assert_eq!(BoundingBox::edge_indices().len(), 12);
    }

    #[test]
    fn empty_positions_have_no_bounds() {
        assert!(BoundingBox::compute_from_positions(&[]).is_none());
    }
}
