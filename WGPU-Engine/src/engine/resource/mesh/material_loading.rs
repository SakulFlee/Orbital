use crate::engine::StandardMaterial;

pub enum MaterialLoading {
    Ignore,
    Try,
    Replace(StandardMaterial),
}

impl PartialEq for MaterialLoading {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}
