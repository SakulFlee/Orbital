pub trait TVertex {
    fn get_position_coordinates(&self) -> [f32; 3];
    fn get_texture_coordinates(&self) -> [f32; 2];
    fn get_normal_coordinates(&self) -> [f32; 3];
}
