use std::any::TypeId;
use std::collections::HashSet;

#[derive(Debug, Clone, Default)]
pub struct ComponentAccess {
    pub reads: HashSet<TypeId>,
    pub writes: HashSet<TypeId>,
}

impl ComponentAccess {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reads<T: 'static>(mut self) -> Self {
        self.reads.insert(TypeId::of::<T>());
        self
    }

    pub fn writes<T: 'static>(mut self) -> Self {
        self.writes.insert(TypeId::of::<T>());
        self
    }

    pub fn conflicts_with(&self, other: &ComponentAccess) -> bool {
        for ty in &self.writes {
            if other.writes.contains(ty) || other.reads.contains(ty) {
                return true;
            }
        }
        for ty in &self.reads {
            if other.writes.contains(ty) {
                return true;
            }
        }
        false
    }
}
