use cgmath::{Vector3, InnerSpace};

/// Generates an arbitrary but consistent tangent and bitangent for a vertex.
/// This is used when proper tangents are not available (e.g., missing from the mesh file
/// and UVs required for calculation are also missing).
///
/// This function aims to create a valid TBN (Tangent, Bitangent, Normal) matrix that is:
/// - Orthogonal: Tangent . Normal = 0, Bitangent . Normal = 0, Tangent . Bitangent = 0
/// - Right-handed: Bitangent = Normal.cross(Tangent)
///
/// This version uses a more consistent approach to avoid discontinuities that can cause
/// visual artifacts like "X" shapes on spheres.
///
/// # Arguments
/// * `normal` - The vertex normal (must be normalized).
///
/// # Returns
/// A tuple (Tangent, Bitangent).
pub fn generate_arbitrary_tangent_frame(normal: Vector3<f32>) -> (Vector3<f32>, Vector3<f32>) {
    // --- Strategy: Use a fixed, consistent reference axis ---
    // Choose a reference vector that is not parallel to the normal.
    // We'll use World Z-axis (0, 0, 1) as the primary reference.
    // If the normal is very close to the Z-axis, we'll use the X-axis (1, 0, 0) instead.
    // This avoids the discontinuity of switching reference axes based on normal components.

    let reference: Vector3<f32>;

    // Check if the normal is pointing close to the World Z-axis.
    // The dot product N . Z = N.z. If |N.z| is close to 1.0, they are parallel.
    if normal.z.abs() < 0.999_999 {
        // Normal is not close to Z-axis, safe to use Z as reference.
        reference = Vector3::new(0.0, 0.0, 1.0);
    } else {
        // Normal is very close to the Z-axis (pointing up or down).
        // Using Z as a reference would result in a zero or near-zero tangent.
        // Use World X-axis instead.
        reference = Vector3::new(1.0, 0.0, 0.0);
    }

    // Calculate tangent: perpendicular to both normal and reference.
    // This is guaranteed to be non-zero due to the reference choice.
    let tangent: Vector3<f32> = normal.cross(reference).normalize();

    // Calculate bitangent: perpendicular to both normal and tangent.
    // This ensures a right-handed TBN matrix.
    let bitangent: Vector3<f32> = normal.cross(tangent);

    (tangent, bitangent)
}