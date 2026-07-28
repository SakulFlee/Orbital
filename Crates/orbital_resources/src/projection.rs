//! Projection matrix builders using the wgpu/WebGPU clip-space convention.
//!
//! wgpu (like Vulkan, D3D12 and Metal) uses NDC z ∈ [0, 1], unlike OpenGL's
//! z ∈ [-1, 1] produced by `cgmath::perspective()` / `cgmath::ortho()`.
//! Using OpenGL-convention matrices with wgpu causes two problems:
//!
//! 1. Geometry in the near half of the frustum (z_ndc < 0) is clipped away.
//! 2. Depth stored in a depth attachment never matches a [0, 1] reference
//!    depth computed in a shader — which is what broke shadow mapping.
//!
//! All projection matrices used with this engine must therefore come from
//! the helpers below.
//!
//! ## The `flip_y` parameter
//!
//! wgpu NDC +Y maps to the **top** of the framebuffer, while texture
//! coordinate V = 0 is the **top** row of a texture. When a projection is
//! used to render into a texture that is later sampled with
//! `uv = ndc.xy * 0.5 + 0.5` (e.g. shadow maps), the Y axis must be flipped
//! in the projection so the sampling math matches the rasterized layout.
//! On-screen rendering (the main camera) does **not** need the flip.

use cgmath::{Matrix4, Rad, Vector4};

/// Right-handed perspective projection, wgpu clip convention (z_ndc ∈ [0, 1]).
///
/// * `fovy` – vertical field of view
/// * `aspect` – width / height
/// * `near` / `far` – positive view-space distances to the clip planes
/// * `flip_y` – negate the Y row (see module docs)
pub fn perspective_wgpu(
    fovy: Rad<f32>,
    aspect: f32,
    near: f32,
    far: f32,
    flip_y: bool,
) -> Matrix4<f32> {
    assert!(near > 0.0, "near plane must be positive");
    assert!(far > near, "far plane must be beyond near plane");

    let f = 1.0 / (fovy.0 * 0.5).tan();
    let y_scale = if flip_y { -f } else { f };

    // cgmath is column-major: .x/.y/.z/.w are columns 0..3.
    // clip.z = A * z_eye + B * w_eye, clip.w = -z_eye
    // with A = far/(near-far), B = near*far/(near-far):
    //   z_eye = -near -> z_ndc = 0,  z_eye = -far -> z_ndc = 1
    Matrix4 {
        x: Vector4::new(f / aspect, 0.0, 0.0, 0.0),
        y: Vector4::new(0.0, y_scale, 0.0, 0.0),
        z: Vector4::new(0.0, 0.0, far / (near - far), -1.0),
        w: Vector4::new(0.0, 0.0, (near * far) / (near - far), 0.0),
    }
}

/// Right-handed orthographic projection, wgpu clip convention (z_ndc ∈ [0, 1]).
///
/// `near` / `far` are positive view-space distances (camera looks down -Z).
/// * `flip_y` – negate the Y row (see module docs)
#[allow(clippy::too_many_arguments)]
pub fn ortho_wgpu(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
    flip_y: bool,
) -> Matrix4<f32> {
    assert!(right > left, "right must be > left");
    assert!(top > bottom, "top must be > bottom");
    assert!(far > near, "far must be > near");

    let y_sign = if flip_y { -1.0 } else { 1.0 };

    // z_ndc = -(z_eye + near) / (far - near):
    //   z_eye = -near -> 0,  z_eye = -far -> 1
    Matrix4 {
        x: Vector4::new(2.0 / (right - left), 0.0, 0.0, 0.0),
        y: Vector4::new(0.0, y_sign * 2.0 / (top - bottom), 0.0, 0.0),
        z: Vector4::new(0.0, 0.0, -1.0 / (far - near), 0.0),
        w: Vector4::new(
            -(right + left) / (right - left),
            y_sign * -(top + bottom) / (top - bottom),
            -near / (far - near),
            1.0,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cgmath::{Deg, Vector4};

    fn ndc(m: &Matrix4<f32>, view: Vector4<f32>) -> Vector4<f32> {
        let clip = *m * view;
        Vector4::new(clip.x / clip.w, clip.y / clip.w, clip.z / clip.w, 1.0)
    }

    #[test]
    fn perspective_depth_range_is_zero_to_one() {
        let (near, far) = (0.1f32, 100.0f32);
        let proj = perspective_wgpu(Rad::from(Deg(60.0)), 16.0 / 9.0, near, far, false);

        let z_near = ndc(&proj, Vector4::new(0.0, 0.0, -near, 1.0)).z;
        let z_far = ndc(&proj, Vector4::new(0.0, 0.0, -far, 1.0)).z;
        let z_mid = ndc(&proj, Vector4::new(0.0, 0.0, -(near + far) * 0.5, 1.0)).z;

        assert!(
            (z_near - 0.0).abs() < 1e-5,
            "near must map to 0, got {z_near}"
        );
        assert!((z_far - 1.0).abs() < 1e-5, "far must map to 1, got {z_far}");
        assert!(
            z_mid > 0.0 && z_mid < 1.0,
            "midpoint must stay inside [0, 1], got {z_mid}"
        );
    }

    #[test]
    fn perspective_y_flip() {
        let proj_no_flip = perspective_wgpu(Rad::from(Deg(60.0)), 1.0, 0.1, 100.0, false);
        let proj_flip = perspective_wgpu(Rad::from(Deg(60.0)), 1.0, 0.1, 100.0, true);

        let view = Vector4::new(0.0, 1.0, -1.0, 1.0);
        let y_no_flip = ndc(&proj_no_flip, view).y;
        let y_flip = ndc(&proj_flip, view).y;

        assert!(y_no_flip > 0.0, "unflipped +Y view must give +Y ndc");
        assert!(
            (y_flip + y_no_flip).abs() < 1e-6,
            "flipped must be the negation of unflipped"
        );
    }

    #[test]
    fn ortho_maps_corners_and_depth() {
        let (l, r, b, t, n, f) = (-10.0f32, 10.0f32, -5.0f32, 5.0f32, 0.5f32, 40.0f32);
        let ortho = ortho_wgpu(l, r, b, t, n, f, false);

        // Center of the box -> ndc (0, 0)
        let center = ndc(&ortho, Vector4::new(0.0, 0.0, -(n + f) * 0.5, 1.0));
        assert!(center.x.abs() < 1e-6 && center.y.abs() < 1e-6);

        // Near / far plane distances -> z 0 / 1
        let z_near = ndc(&ortho, Vector4::new(0.0, 0.0, -n, 1.0)).z;
        let z_far = ndc(&ortho, Vector4::new(0.0, 0.0, -f, 1.0)).z;
        assert!(
            (z_near - 0.0).abs() < 1e-6,
            "near must map to 0, got {z_near}"
        );
        assert!((z_far - 1.0).abs() < 1e-6, "far must map to 1, got {z_far}");

        // Box edges -> ndc ±1
        let left = ndc(&ortho, Vector4::new(l, 0.0, -1.0, 1.0)).x;
        let right = ndc(&ortho, Vector4::new(r, 0.0, -1.0, 1.0)).x;
        let top = ndc(&ortho, Vector4::new(0.0, t, -1.0, 1.0)).y;
        assert!((left + 1.0).abs() < 1e-6 && (right - 1.0).abs() < 1e-6);
        assert!((top - 1.0).abs() < 1e-6);
    }

    #[test]
    fn ortho_y_flip() {
        let ortho = ortho_wgpu(-1.0, 1.0, -1.0, 1.0, 0.1, 10.0, true);
        let y = ndc(&ortho, Vector4::new(0.0, 1.0, -1.0, 1.0)).y;
        assert!(
            (y + 1.0).abs() < 1e-6,
            "flipped top must map to -1, got {y}"
        );
    }
}
