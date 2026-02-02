// --- Settings

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
};

type EntityId = usize;
static PAGE_SIZE: usize = 1024;

// --- Test ---

#[derive(Debug)]
struct TestMarker;

#[derive(Debug)]
struct TestComponent {
    pub x: i32,
}

// --- Impl ---

#[derive(Debug)]
struct SparseSet<T> {
    sparse: Vec<Option<usize>>,
    dense_entities: Vec<EntityId>,
    dense_components: Vec<T>,
}

impl<T> SparseSet<T> {
    pub fn new() -> Self {
        Self {
            sparse: Vec::new(),
            dense_entities: Vec::new(),
            dense_components: Vec::new(),
        }
    }

    pub fn add(&mut self, entity_id: EntityId, component: T) {
        // Store the compoent at the end of the dense set.
        // Len() is already +1, thus its the next index to be used!
        let dense_index = self.dense_components.len();
        self.dense_entities.push(entity_id);
        self.dense_components.push(component);

        // Resize sparse set if entity ID is bigger than entity id
        if entity_id >= self.sparse.len() {
            self.sparse.resize(entity_id + 1, None);
        }

        self.sparse[entity_id] = Some(dense_index);
    }

    pub fn get_index(&self, entity_id: EntityId) -> Option<usize> {
        if entity_id >= self.sparse.len() {
            return None;
        }

        self.sparse[entity_id]
    }

    pub fn contained(&self, entity_id: EntityId) -> bool {
        self.get_index(entity_id).is_some()
    }

    pub fn get_entity_id(&self, dense_index: usize) -> Option<&EntityId> {
        self.dense_entities.get(dense_index)
    }

    pub fn get_component(&self, entity_id: EntityId) -> Option<&T> {
        let dense_index = self.get_index(entity_id)?;

        let component = &self.dense_components[dense_index];
        Some(component)
    }
}

pub trait StorageBucket: Debug {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Debug + 'static> StorageBucket for SparseSet<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug)]
struct World {
    next_entity_id: EntityId,
    components: HashMap<TypeId, Box<dyn StorageBucket>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            next_entity_id: 0,
            components: HashMap::new(),
        }
    }

    pub fn spawn(&mut self) -> EntityId {
        let next_id = self.next_entity_id;
        self.next_entity_id += 1;
        next_id
    }

    pub fn attach_component<T: Debug + 'static>(&mut self, entity_id: EntityId, component: T) {
        let type_id = TypeId::of::<T>();
        println!("TypeID: {:?}", type_id);

        let entry = self
            .components
            .entry(type_id)
            .or_insert_with(|| Box::new(SparseSet::<T>::new()));

        let storage = entry.as_any_mut().downcast_mut::<SparseSet<T>>().unwrap();
        storage.add(entity_id, component);
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Variant {
    Empty,
    // Normal types
    String(String),
    Boolean(bool),
    // Unsigned Integers
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    // Signed Integers
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    // Floating point numbers
    F32(f32),
    F64(f64),
    Array(Vec<Variant>),
    Blob(Vec<u8>),
}

pub trait Component: Send + Sync + 'static {
    fn to_variant(&self) -> Variant;
    fn update_from_variant(&mut self, variant: Variant);
}

#[derive(Debug)]
pub struct Position {
    x: f32,
    y: f32,
    z: f32,
}

impl Component for Position {
    fn to_variant(&self) -> Variant {
        Variant::Array(vec![
            Variant::F32(self.x),
            Variant::F32(self.y),
            Variant::F32(self.z),
        ])
    }

    fn update_from_variant(&mut self, data: Variant) {
        if let Variant::Array(values) = data
            && values.len() == 3
        {
            if let Variant::F32(x) = values[0] {
                self.x = x;
            }

            if let Variant::F32(y) = values[1] {
                self.y = y;
            }

            if let Variant::F32(z) = values[2] {
                self.z = z;
            }
        }
    }
}

fn main() {
    let mut world = World::new();

    let entity_id = world.spawn();
    world.attach_component(entity_id, TestComponent { x: 1 });
    world.attach_component(entity_id, TestMarker);

    let entity_id = world.spawn();
    world.attach_component(entity_id, TestComponent { x: 1 });

    println!("World: {:?}", world);
    println!();

    let mut position = Position {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    };
    println!("Position: {:?}", position);
    let variant = position.to_variant();
    println!("Variant: {:?}", variant);

    let update_variant = Variant::Array(vec![
        Variant::F32(3.0),
        Variant::F32(2.0),
        Variant::F32(1.0),
    ]);
    println!(
        "Updating position ({:?}) with variant: {:?}",
        position, variant
    );

    position.update_from_variant(update_variant);
    println!("Position: {:?}", position);

    println!();
}

#[cfg(test)]
mod tests {
    use crate::{Component, Variant};

    #[derive(Debug)]
    struct Position {
        x: f32,
        y: f32,
        z: f32,
    }

    impl Component for Position {
        fn to_variant(&self) -> Variant {
            Variant::Array(vec![
                Variant::F32(self.x),
                Variant::F32(self.y),
                Variant::F32(self.z),
            ])
        }

        fn update_from_variant(&mut self, variant: Variant) {
            if let Variant::Array(values) = variant
                && values.len() == 3
            {
                if let Variant::F32(x) = values[0] {
                    self.x = x;
                }

                if let Variant::F32(y) = values[1] {
                    self.y = y;
                }

                if let Variant::F32(z) = values[2] {
                    self.z = z;
                }
            }
        }
    }

    #[test]
    fn component_to_variant() {
        let position = Position {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };
        println!("Position: {:?}", position);

        let variant = position.to_variant();
        println!("Variant: {:?}", variant);

        let Variant::Array(values) = variant else {
            panic!("Excepted Array!")
        };

        let Variant::F32(x) = values[0] else {
            panic!("Expected F32")
        };
        assert_eq!(position.x, x);

        let Variant::F32(y) = values[1] else {
            panic!("Expected F32")
        };
        assert_eq!(position.y, y);

        let Variant::F32(z) = values[2] else {
            panic!("Expected F32")
        };
        assert_eq!(position.z, z);
    }

    #[test]
    fn update_component_from_variant() {
        let mut position = Position {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };
        println!("Position: {:?}", position);

        let update_variant = Variant::Array(vec![
            Variant::F32(position.z),
            Variant::F32(position.x),
            Variant::F32(position.y),
        ]);
        position.update_from_variant(update_variant);
        println!("Position: {:?}", position);

        assert_eq!(position.x, 3.0);
        assert_eq!(position.y, 1.0);
        assert_eq!(position.z, 2.0);
    }
}
