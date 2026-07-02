#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity {
    pub index: usize,
    pub generation: usize,
}

impl Entity {
    pub fn new(index: usize, generation: usize) -> Self {
        Self { index, generation }
    }
}
