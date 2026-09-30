struct Light {
    position: vec4<f32>,     // xyz: position, w: padding
    color: vec4<f32>,        // xyz: color, w: intensity
    direction: vec4<f32>,    // xyz: direction, w: type
    params: vec4<f32>,       // x/y: spot angular attenuation scale/offset (0 for non-spot)
                            // z: range² for distance culling (0 = infinite)
                            // w: padding
}
