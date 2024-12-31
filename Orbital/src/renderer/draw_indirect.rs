#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct DrawIndirect {
    vertex_count: u32,
    instance_count: u32,
    first_index: u32,
    first_instance: u32,
}

impl DrawIndirect {
    pub fn new(vertex_count: u32, instance_count: u32) -> Self {
        Self::new_full(vertex_count, instance_count, 0, 0)
    }

    pub fn new_full(
        vertex_count: u32,
        instance_count: u32,
        first_index: u32,
        first_instance: u32,
    ) -> Self {
        Self {
            vertex_count,
            instance_count,
            first_index,
            first_instance,
        }
    }

    pub fn to_binary_data(&self) -> Vec<u8> {
        [self.vertex_count.to_le_bytes(),
            self.instance_count.to_le_bytes(),
            self.first_index.to_le_bytes(),
            self.first_instance.to_le_bytes()]
        .concat()
    }
}
