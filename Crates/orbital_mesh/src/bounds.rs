use cgmath::{EuclideanSpace, InnerSpace, Point3, Vector3};

#[derive(Debug, Clone, PartialEq)]
pub struct BoundingSphere {
    pub center: Point3<f32>,
    pub radius: f32,
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
