use cgmath::{Point3, Vector3};

use crate::{Camera, CameraChange, CameraDescriptor, Mode};

#[test]
fn realization_default() {
    let (_, device, queue) = wgpu_test_adapter::make_wgpu_connection();

    let descriptor = CameraDescriptor::default();

    let _realization = Camera::from_descriptor(descriptor, &device, &queue);
}

#[test]
fn defaults() {
    logging::test_init();

    let descriptor = CameraDescriptor::default();

    assert_eq!(descriptor.label, CameraDescriptor::DEFAULT_NAME);
    assert_eq!(descriptor.position, Point3::new(0.0, 0.0, 0.0));
    assert_eq!(descriptor.yaw, 0f32);
    assert_eq!(descriptor.pitch, 0f32);
    assert_eq!(descriptor.aspect, 16.0 / 9.0);
    assert_eq!(descriptor.fovy, 45.0);
    assert_eq!(descriptor.near, 0.1);
    assert_eq!(descriptor.far, 10000.0);
    assert_eq!(descriptor.global_gamma, 2.2);
}

#[test]
fn realization_change_is_not_changing() {
    let original_descriptor = CameraDescriptor::default();
    let mut to_be_changed_descriptor = original_descriptor.clone();

    let change = CameraChange {
        target: CameraDescriptor::DEFAULT_NAME.to_string(),
        position: None,
        pitch: None,
        yaw: None,
    };
    assert!(!change.is_introducing_change());

    to_be_changed_descriptor.apply_change(change);
    assert_eq!(original_descriptor, to_be_changed_descriptor);
}

#[test]
fn realization_change_is_changing() {
    const POSITION_X: f32 = 1.0;
    const POSITION_Y: f32 = 2.0;
    const POSITION_Z: f32 = 3.0;
    // Note: Pitch clamping is a thing! Keep it low.
    const PITCH: f32 = 0.12345;
    const YAW: f32 = 0.54321;

    let original_descriptor = CameraDescriptor::default();
    let mut to_be_changed_descriptor = original_descriptor.clone();

    let change = CameraChange {
        target: CameraDescriptor::DEFAULT_NAME.to_string(),
        position: Some(Mode::Overwrite(Vector3 {
            x: POSITION_X,
            y: POSITION_Y,
            z: POSITION_Z,
        })),
        pitch: Some(Mode::Overwrite(PITCH)),
        yaw: Some(Mode::Overwrite(YAW)),
    };
    assert!(change.is_introducing_change());

    to_be_changed_descriptor.apply_change(change);
    assert_ne!(original_descriptor, to_be_changed_descriptor);

    assert_eq!(
        to_be_changed_descriptor.position,
        Point3::new(POSITION_X, POSITION_Y, POSITION_Z)
    );
    assert_eq!(to_be_changed_descriptor.pitch, PITCH);
    assert_eq!(to_be_changed_descriptor.yaw, YAW);
}

#[test]
fn realization_change_position_only() {
    const POSITION_X: f32 = 1.0;
    const POSITION_Y: f32 = 2.0;
    const POSITION_Z: f32 = 3.0;

    let original_descriptor = CameraDescriptor::default();
    let mut to_be_changed_descriptor = original_descriptor.clone();

    let change = CameraChange {
        target: CameraDescriptor::DEFAULT_NAME.to_string(),
        position: Some(Mode::Overwrite(Vector3 {
            x: POSITION_X,
            y: POSITION_Y,
            z: POSITION_Z,
        })),
        pitch: None,
        yaw: None,
    };
    assert!(change.is_introducing_change());

    to_be_changed_descriptor.apply_change(change);
    assert_ne!(original_descriptor, to_be_changed_descriptor);

    assert_eq!(
        to_be_changed_descriptor.position,
        Point3::new(POSITION_X, POSITION_Y, POSITION_Z)
    );
    assert_eq!(to_be_changed_descriptor.pitch, original_descriptor.pitch);
    assert_eq!(to_be_changed_descriptor.yaw, original_descriptor.yaw);
}

#[test]
fn realization_change_pitch_only() {
    const PITCH: f32 = 0.12345;

    let original_descriptor = CameraDescriptor::default();
    let mut to_be_changed_descriptor = original_descriptor.clone();

    let change = CameraChange {
        target: CameraDescriptor::DEFAULT_NAME.to_string(),
        position: None,
        pitch: Some(Mode::Overwrite(PITCH)),
        yaw: None,
    };
    assert!(change.is_introducing_change());

    to_be_changed_descriptor.apply_change(change);
    assert_ne!(original_descriptor, to_be_changed_descriptor);

    assert_eq!(
        to_be_changed_descriptor.position,
        original_descriptor.position
    );
    assert_eq!(to_be_changed_descriptor.pitch, PITCH);
    assert_eq!(to_be_changed_descriptor.yaw, original_descriptor.yaw);
}

#[test]
fn realization_change_yaw_only() {
    const YAW: f32 = 0.54321;

    let original_descriptor = CameraDescriptor::default();
    let mut to_be_changed_descriptor = original_descriptor.clone();

    let change = CameraChange {
        target: CameraDescriptor::DEFAULT_NAME.to_string(),
        position: None,
        pitch: None,
        yaw: Some(Mode::Overwrite(YAW)),
    };
    assert!(change.is_introducing_change());

    to_be_changed_descriptor.apply_change(change);
    assert_ne!(original_descriptor, to_be_changed_descriptor);

    assert_eq!(
        to_be_changed_descriptor.position,
        original_descriptor.position
    );
    assert_eq!(to_be_changed_descriptor.pitch, original_descriptor.pitch);
    assert_eq!(to_be_changed_descriptor.yaw, YAW);
}

#[test]
fn pitch_clamping() {
    let mut to_be_changed_descriptor = CameraDescriptor::default();

    let change = CameraChange {
        target: CameraDescriptor::DEFAULT_NAME.to_string(),
        position: None,
        pitch: Some(Mode::Overwrite(CameraDescriptor::SAFE_FRAC_PI_2 + 0.1)),
        yaw: None,
    };
    assert!(change.is_introducing_change());
    to_be_changed_descriptor.apply_change(change);

    assert_eq!(
        to_be_changed_descriptor.pitch,
        CameraDescriptor::SAFE_FRAC_PI_2
    );
}

#[test]
fn pitch_negative_clamping() {
    let mut to_be_changed_descriptor = CameraDescriptor::default();

    let change = CameraChange {
        target: CameraDescriptor::DEFAULT_NAME.to_string(),
        position: None,
        pitch: Some(Mode::Overwrite(-CameraDescriptor::SAFE_FRAC_PI_2 - 0.1)),
        yaw: None,
    };
    assert!(change.is_introducing_change());
    to_be_changed_descriptor.apply_change(change);

    assert_eq!(
        to_be_changed_descriptor.pitch,
        -CameraDescriptor::SAFE_FRAC_PI_2
    );
}
