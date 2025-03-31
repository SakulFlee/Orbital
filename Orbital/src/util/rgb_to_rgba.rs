pub fn rgb_to_rgba(data: &[u8]) -> Vec<u8> {
    data.chunks(3)
        .map(|x| [x[0], x[1], x[2], 255])
        .collect::<Vec<_>>()
        .concat()
}
