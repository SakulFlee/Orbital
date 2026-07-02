/// Given a [`size`], will return the maximal possible
pub fn max_mip_level(size: u32) -> u32 {
    size.ilog2() + 1
}
