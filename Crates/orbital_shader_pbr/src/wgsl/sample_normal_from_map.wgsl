fn sample_normal_from_map(fragment_data: FragmentData) -> vec3<f32> {
    let normal_sample = textureSample(
        normal_texture,
        normal_sampler,
        fragment_data.uv
    ).rgb;
    let tangent_normal = 2.0 * normal_sample - 1.0;

    let TBN = mat3x3(
        fragment_data.tangent,
        fragment_data.bitangent,
        fragment_data.normal,
    );
    let N = normalize(TBN * tangent_normal);
    return N;
}
