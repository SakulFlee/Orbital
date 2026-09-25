fn pcg_hash(seed: u32) -> u32 {
    var state = seed;
    state = state * 747796405u + 2891336453u;
    return ((state >> ((state >> 28u) + 4u)) ^ state);
}
