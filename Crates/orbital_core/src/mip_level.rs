pub fn max_mip_level(size: u32) -> u32 {
    size.ilog2() + 1
}
