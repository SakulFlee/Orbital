use cgmath::{Vector3, InnerSpace};

/// Generates an arbitrary but consistent tangent and bitangent for a vertex.
/// This is used when proper tangents are not available (e.g., missing from the mesh file
/// and UVs required for calculation are also missing).
///
/// This function aims to create a valid TBN (Tangent, Bitangent, Normal) matrix that is:
/// - Orthogonal: Tangent . Normal = 0, Bitangent . Normal = 0, Tangent . Bitangent = 0
/// - Right-handed: Bitangent = Normal.cross(Tangent)
///
/// # Arguments
/// * `normal` - The vertex normal (must be normalized).
///
/// # Returns
/// A tuple (Tangent, Bitangent).
pub fn generate_arbitrary_tangent_frame(normal: Vector3<f32>) -> (Vector3<f32>, Vector3<f32>) {
    // --- Strategy: Find an arbitrary orthogonal vector ---
    // We need a vector that is not parallel (or near-parallel) to the normal.
    // A common and effective way is to compare the normal's components.

    let tangent: Vector3<f32>;

    // Find the axis with the smallest absolute component.
    // This axis is the "furthest" from the normal's direction.
    // Using `abs()` handles normals pointing in negative directions correctly.
    let abs_normal_x = normal.x.abs();
    let abs_normal_y = normal.y.abs();
    let abs_normal_z = normal.z.abs();

    // Check which component is the smallest. In case of ties, the first match determines the path.
    if abs_normal_x <= abs_normal_y && abs_normal_x <= abs_normal_z {
        // X component is smallest (or tied for smallest).
        // Use World X-axis (1, 0, 0) as the reference vector.
        let reference = Vector3::new(1.0, 0.0, 0.0);
        // Calculate tangent: perpendicular to both normal and reference.
        tangent = normal.cross(reference).normalize();
    } else if abs_normal_y <= abs_normal_z {
        // Y component is smallest (or tied for smallest, and X was larger).
        // Use World Y-axis (0, 1, 0) as the reference vector.
        let reference = Vector3::new(0.0, 1.0, 0.0);
        // Calculate tangent: perpendicular to both normal and reference.
        tangent = normal.cross(reference).normalize();
    } else {
        // Z component must be the smallest.
        // Use World Z-axis (0, 0, 1) as the reference vector.
        let reference = Vector3::new(0.0, 0.0, 1.0);
        // Calculate tangent: perpendicular to both normal and reference.
        tangent = normal.cross(reference).normalize();
    }

    // --- Calculate Bitangent ---
    // Ensure a proper right-handed TBN matrix.
    // B = N.cross(T) is the standard form.
    let bitangent = normal.cross(tangent);

    (tangent, bitangent)
}