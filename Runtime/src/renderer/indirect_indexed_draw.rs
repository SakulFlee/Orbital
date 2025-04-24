#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct IndirectIndexedDraw {
    index_count: u32,
    instance_count: u32,
    first_index: u32,
    base_vertex: u32,
    first_instance: u32,
}

impl IndirectIndexedDraw {
    pub fn new(index_count: u32, instance_count: u32) -> Self {
        Self::new_full(index_count, instance_count, 0, 0, 0)
    }

    pub fn new_full(
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        base_vertex: u32,
        first_instance: u32,
    ) -> Self {
        Self {
            index_count,
            instance_count,
            first_index,
            base_vertex,
            first_instance,
        }
    }

    pub fn to_binary_data(&self) -> Vec<u8> {
        [
            self.index_count.to_le_bytes(),
            self.instance_count.to_le_bytes(),
            self.first_index.to_le_bytes(),
            self.base_vertex.to_le_bytes(),
            self.first_instance.to_le_bytes(),
        ]
        .concat()
    }
    
    pub fn byte_space_requirement() -> usize {
        5 * std::mem::size_of::<u32>() 
    }
}
