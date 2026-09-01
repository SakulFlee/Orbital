#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum SamplingType {
    ImportanceSampling,
    GaussianBlur,
    BoxBlur,
}

impl SamplingType {
    pub fn to_u32(&self) -> u32 {
        match self {
            SamplingType::ImportanceSampling => 0,
            SamplingType::GaussianBlur => 1,
            SamplingType::BoxBlur => 2,
        }
    }

    pub fn to_le_bytes(&self) -> [u8; 4] {
        let u = self.to_u32();
        u.to_le_bytes()
    }
}
