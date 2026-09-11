fn radical_inverse_vdc(bits: u32) -> f32 {
    var reversed = bits;
    reversed = (reversed & 0x55555555u) << 1u | (reversed & 0xAAAAAAAAu) >> 1u;
    reversed = (reversed & 0x33333333u) << 2u | (reversed & 0xCCCCCCCCu) >> 2u;
    reversed = (reversed & 0x0F0F0F0Fu) << 4u | (reversed & 0xF0F0F0F0u) >> 4u;
    reversed = (reversed & 0x00FF00FFu) << 8u | (reversed & 0xFF00FF00u) >> 8u;
    reversed = (reversed << 16u) | (reversed >> 16u);
    return f32(reversed) * 2.3283064365386963e-10;
}
