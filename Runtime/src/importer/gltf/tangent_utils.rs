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

/// Generates tangent and bitangent for UV sphere vertices using spherical coordinates.
/// This provides more consistent tangent directions for UV spheres, especially at the poles.
///
/// # Arguments
/// * `normal` - The vertex normal (must be normalized).
/// * `uv` - The UV coordinates of the vertex (optional, for more precise calculation).
///
/// # Returns
/// A tuple (Tangent, Bitangent).
pub fn generate_sphere_tangent_frame(normal: Vector3<f32>, uv: Option<(f32, f32)>) -> (Vector3<f32>, Vector3<f32>) {
    // Special handling for poles to ensure consistent tangent directions
    let pole_threshold = 0.95; // More lenient threshold for pole detection
    
    if normal.y.abs() > pole_threshold {
        // We're at a pole (north or south)
        // At poles, we want a consistent tangent direction for all vertices
        // Use a fixed reference direction that's perpendicular to the pole normal
        
        log::trace!("Detected pole vertex with normal: {:?}", normal);
        
        // For poles, use a completely fixed tangent direction
        // This ensures ALL vertices at the pole have the EXACT same tangent
        let tangent = Vector3::new(1.0, 0.0, 0.0);
        let bitangent = normal.cross(tangent);
        
        return (tangent, bitangent);
    }
    
    // For non-pole vertices, use the standard spherical coordinate approach
    // Calculate tangent (U direction) - this is the derivative with respect to phi
    // For a unit sphere: x = sin(theta) * cos(phi), y = cos(theta), z = sin(theta) * sin(phi)
    // d/dphi = (-sin(theta) * sin(phi), 0, sin(theta) * cos(phi))
    let tangent = Vector3::new(
        -normal.z,  // -sin(theta) * sin(phi)
        0.0,        // 0
        normal.x,   // sin(theta) * cos(phi)
    ).normalize();
    
    // Calculate bitangent (V direction) - this is the derivative with respect to theta
    // d/dtheta = (cos(theta) * cos(phi), -sin(theta), cos(theta) * sin(phi))
    let bitangent = Vector3::new(
        normal.x * normal.y,  // cos(theta) * cos(phi)
        -(1.0 - normal.y * normal.y).sqrt(), // -sin(theta)
        normal.z * normal.y,  // cos(theta) * sin(phi)
    ).normalize();
    
    // Ensure right-handed coordinate system
    let cross_check = normal.cross(tangent);
    if cross_check.dot(bitangent) < 0.0 {
        // If the cross product points in the opposite direction, flip the bitangent
        return (tangent, -bitangent);
    }
    
    (tangent, bitangent)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cgmath::InnerSpace;

    #[test]
    fn test_sphere_tangent_frame_poles() {
        // Test north pole
        let north_pole = Vector3::new(0.0, 1.0, 0.0);
        let (tangent, bitangent) = generate_sphere_tangent_frame(north_pole, None);
        
        // Verify orthogonality
        assert!(tangent.dot(north_pole).abs() < 1e-6, "Tangent should be perpendicular to normal");
        assert!(bitangent.dot(north_pole).abs() < 1e-6, "Bitangent should be perpendicular to normal");
        assert!(tangent.dot(bitangent).abs() < 1e-6, "Tangent should be perpendicular to bitangent");
        
        // Verify that pole tangent is consistent (should be in X direction)
        assert!((tangent.x - 1.0).abs() < 1e-6, "North pole tangent should be in X direction");
        assert!(tangent.y.abs() < 1e-6, "North pole tangent should have no Y component");
        assert!(tangent.z.abs() < 1e-6, "North pole tangent should have no Z component");
        
        // Test south pole
        let south_pole = Vector3::new(0.0, -1.0, 0.0);
        let (tangent, bitangent) = generate_sphere_tangent_frame(south_pole, None);
        
        // Verify orthogonality
        assert!(tangent.dot(south_pole).abs() < 1e-6, "Tangent should be perpendicular to normal");
        assert!(bitangent.dot(south_pole).abs() < 1e-6, "Bitangent should be perpendicular to normal");
        assert!(tangent.dot(bitangent).abs() < 1e-6, "Tangent should be perpendicular to bitangent");
        
        // Verify that pole tangent is consistent (should be in X direction)
        assert!((tangent.x - 1.0).abs() < 1e-6, "South pole tangent should be in X direction");
        assert!(tangent.y.abs() < 1e-6, "South pole tangent should have no Y component");
        assert!(tangent.z.abs() < 1e-6, "South pole tangent should have no Z component");
    }

    #[test]
    fn test_sphere_tangent_frame_equator() {
        // Test equator point
        let equator_point = Vector3::new(1.0, 0.0, 0.0);
        let (tangent, bitangent) = generate_sphere_tangent_frame(equator_point, None);
        
        // Verify orthogonality
        assert!(tangent.dot(equator_point).abs() < 1e-6, "Tangent should be perpendicular to normal");
        assert!(bitangent.dot(equator_point).abs() < 1e-6, "Bitangent should be perpendicular to normal");
        assert!(tangent.dot(bitangent).abs() < 1e-6, "Tangent should be perpendicular to bitangent");
        
        // For equator point, tangent should be in the Y direction (around the sphere)
        assert!(tangent.y.abs() < 1e-6, "Tangent at equator should be perpendicular to Y axis");
    }
}