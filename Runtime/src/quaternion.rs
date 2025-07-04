use cgmath::num_traits::Pow;
use cgmath::Quaternion;

pub fn quaternion_to_pitch_yaw(q: &Quaternion<f32>) -> (f32, f32) {
    let yaw = f32::atan2(
        2.0 * (q.v.y * q.v.z + q.s * q.v.x),
        q.s * q.s - q.v.x * q.v.x - q.v.y * q.v.y + q.v.z * q.v.z,
    );
    let pitch = f32::asin(-2.0 * (q.v.x * q.v.z - q.s * q.v.y));

    // For later once we need it! :)
    // let roll = f32::atan2(
    //     2.0 * (q.v.x * q.v.y + q.s * q.v.z),
    //     q.s * q.s + q.v.x * q.v.x - q.v.y * q.v.y - q.v.z * q.v.z
    // );

    (pitch, yaw)
}
