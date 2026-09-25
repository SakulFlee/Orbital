fn star_hash(cx: u32, cy: u32, cz: u32) -> u32 {
    var h = cx * 374761393u + cy * 668265263u + cz * 1274126177u;
    h = (h ^ (h >> 13u)) * 1103515245u;
    h = h ^ (h >> 16u);
    return h;
}
