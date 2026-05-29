mod entity_id;
pub use entity_id::*;

mod component;
pub use component::*;

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
};

#[derive(Debug)]
pub struct World {
    pub entity_counter: EntityIdType,
    pub entity_ids: Vec<Entity>,
    // TODO: Recycle entity IDs
    pub component_stores: HashMap<TypeId, Box<dyn WorldComponentStorage>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            entity_counter: 0,
            entity_ids: Vec::new(),
            component_stores: HashMap::new(),
        }
    }

    pub fn spawn_entity(&mut self) -> Entity {
        let entity_id = self.entity_counter;
        self.entity_counter = self.entity_counter.wrapping_add(1);

        let entity = Entity::new(entity_id);
        self.entity_ids.push(entity);

        entity
    }

    pub fn despawn_entity(&mut self, entity: &Entity) {
        self.entity_ids.retain(|e| e.index != entity.index);
        self.component_stores
            .iter_mut()
            .for_each(|(_, x)| x.remove_entity(entity.index));
    }

    pub fn get_component_store<T: Any + Debug>(&self) -> Option<&ComponentStore<T>> {
        let type_id = TypeId::of::<T>();

        self.component_stores
            .get(&type_id)
            .and_then(|store| (**store).as_any().downcast_ref::<ComponentStore<T>>())
    }
}

fn main() {
    println!("This should be a library ...");
}

// // --- Settings
//
// use std::{
//     any::{Any, TypeId},
//     collections::HashMap,
//     fmt::Debug,
// };
//
// use orbital_variant::Variant;
// use wasmer::{AsEngineRef, AsStoreMut, FunctionEnvMut, Instance, Module, Store, TypedFunction};
//
// type EntityId = usize;
// static PAGE_SIZE: usize = 1024;
//
// // --- Test ---
//
// #[derive(Debug)]
// struct TestMarker;
//
// #[derive(Debug)]
// struct TestComponent {
//     pub x: i32,
// }
//
// // --- Impl ---
//
// #[derive(Debug)]
// struct SparseSet<T> {
//     sparse: Vec<Option<usize>>,
//     dense_entities: Vec<EntityId>,
//     dense_components: Vec<T>,
// }
//
// impl<T> SparseSet<T> {
//     pub fn new() -> Self {
//         Self {
//             sparse: Vec::new(),
//             dense_entities: Vec::new(),
//             dense_components: Vec::new(),
//         }
//     }
//
//     pub fn add(&mut self, entity_id: EntityId, component: T) {
//         // Store the compoent at the end of the dense set.
//         // Len() is already +1, thus its the next index to be used!
//         let dense_index = self.dense_components.len();
//         self.dense_entities.push(entity_id);
//         self.dense_components.push(component);
//
//         // Resize sparse set if entity ID is bigger than entity id
//         if entity_id >= self.sparse.len() {
//             self.sparse.resize(entity_id + 1, None);
//         }
//
//         self.sparse[entity_id] = Some(dense_index);
//     }
//
//     pub fn get_index(&self, entity_id: EntityId) -> Option<usize> {
//         if entity_id >= self.sparse.len() {
//             return None;
//         }
//
//         self.sparse[entity_id]
//     }
//
//     pub fn contained(&self, entity_id: EntityId) -> bool {
//         self.get_index(entity_id).is_some()
//     }
//
//     pub fn get_entity_id(&self, dense_index: usize) -> Option<&EntityId> {
//         self.dense_entities.get(dense_index)
//     }
//
//     pub fn get_component(&self, entity_id: EntityId) -> Option<&T> {
//         let dense_index = self.get_index(entity_id)?;
//
//         let component = &self.dense_components[dense_index];
//         Some(component)
//     }
// }
//
// pub trait StorageBucket: Debug {
//     fn as_any(&self) -> &dyn Any;
//     fn as_any_mut(&mut self) -> &mut dyn Any;
// }
//
// impl<T: Debug + 'static> StorageBucket for SparseSet<T> {
//     fn as_any(&self) -> &dyn Any {
//         self
//     }
//
//     fn as_any_mut(&mut self) -> &mut dyn Any {
//         self
//     }
// }
//
// #[derive(Debug)]
// struct World {
//     next_entity_id: EntityId,
//     components: HashMap<TypeId, Box<dyn StorageBucket>>,
// }
//
// impl World {
//     pub fn new() -> Self {
//         Self {
//             next_entity_id: 0,
//             components: HashMap::new(),
//         }
//     }
//
//     pub fn spawn(&mut self) -> EntityId {
//         let next_id = self.next_entity_id;
//         self.next_entity_id += 1;
//         next_id
//     }
//
//     pub fn attach_component<T: Debug + 'static>(&mut self, entity_id: EntityId, component: T) {
//         let type_id = TypeId::of::<T>();
//         println!("TypeID: {:?}", type_id);
//
//         let entry = self
//             .components
//             .entry(type_id)
//             .or_insert_with(|| Box::new(SparseSet::<T>::new()));
//
//         let storage = entry.as_any_mut().downcast_mut::<SparseSet<T>>().unwrap();
//         storage.add(entity_id, component);
//     }
// }
//
// pub trait Component: Send + Sync + 'static {
//     fn to_variant(&self) -> Variant;
//     fn update_from_variant(&mut self, variant: Variant);
// }
//
// #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
// pub struct Position {
//     x: f32,
//     y: f32,
//     z: f32,
// }
//
// impl Component for Position {
//     fn to_variant(&self) -> Variant {
//         Variant::Array(vec![
//             Variant::F32(self.x),
//             Variant::F32(self.y),
//             Variant::F32(self.z),
//         ])
//     }
//
//     fn update_from_variant(&mut self, data: Variant) {
//         if let Variant::Array(values) = data
//             && values.len() == 3
//         {
//             if let Variant::F32(x) = values[0] {
//                 self.x = x;
//             }
//
//             if let Variant::F32(y) = values[1] {
//                 self.y = y;
//             }
//
//             if let Variant::F32(z) = values[2] {
//                 self.z = z;
//             }
//         }
//     }
// }
//
// #[repr(u8)]
// pub enum FieldType {
//     Bool = 0,
//     U8 = 1,
//     I8 = 2,
//     U16 = 3,
//     I16 = 4,
//     U32 = 5,
//     I32 = 6,
//     F32 = 7,
//     U64 = 8,
//     I64 = 9,
//     F64 = 10,
// }
//
// impl FieldType {
//     pub fn size(&self) -> usize {
//         use std::mem::size_of;
//
//         match self {
//             FieldType::Bool => size_of::<bool>(),
//             FieldType::U8 => size_of::<u8>(),
//             FieldType::I8 => size_of::<i8>(),
//             FieldType::U16 => size_of::<u16>(),
//             FieldType::I16 => size_of::<i16>(),
//             FieldType::U32 => size_of::<u32>(),
//             FieldType::I32 => size_of::<i32>(),
//             FieldType::F32 => size_of::<f32>(),
//             FieldType::U64 => size_of::<u64>(),
//             FieldType::I64 => size_of::<i64>(),
//             FieldType::F64 => size_of::<f64>(),
//         }
//     }
//
//     pub fn alignment(&self) -> usize {
//         match self {
//             FieldType::Bool | FieldType::U8 | FieldType::I8 => 1,
//             FieldType::U16 | FieldType::I16 => 2,
//             FieldType::U32 | FieldType::I32 | FieldType::F32 => 4,
//             FieldType::U64 | FieldType::I64 | FieldType::F64 => 8,
//         }
//     }
// }
//
// pub struct ComponentSchema {
//     pub fields: Vec<FieldType>,
//     pub field_names: Vec<String>,
// }
//
// fn main() {
//     // let mut world = World::new();
//     //
//     // let entity_id = world.spawn();
//     // world.attach_component(entity_id, TestComponent { x: 1 });
//     // world.attach_component(entity_id, TestMarker);
//     //
//     // let entity_id = world.spawn();
//     // world.attach_component(entity_id, TestComponent { x: 1 });
//     //
//     // println!("World: {:?}", world);
//     // println!();
//     //
//     // let mut position = Position {
//     //     x: 1.0,
//     //     y: 2.0,
//     //     z: 3.0,
//     // };
//     // println!("Position: {:?}", position);
//     // let variant = position.to_variant();
//     // println!("Variant: {:?}", variant);
//     //
//     // let update_variant = Variant::Array(vec![
//     //     Variant::F32(3.0),
//     //     Variant::F32(2.0),
//     //     Variant::F32(1.0),
//     // ]);
//     // println!(
//     //     "Updating position ({:?}) with variant: {:?}",
//     //     position, variant
//     // );
//     //
//     // position.update_from_variant(update_variant);
//     // println!("Position: {:?}", position);
//
//     // ---
//     // let input_variant = Variant::F32(0.123);
//     let input = Position {
//         x: 0.123,
//         y: 0.0,
//         z: 0.0,
//     };
//     let input_bytes = unsafe {
//         std::slice::from_raw_parts(
//             &input as *const Position as *const u8,
//             std::mem::size_of::<Position>(),
//         )
//     };
//     println!("Converted {:?} into {:?}", input, input_bytes);
//
//     // let wasm_module = include_bytes!("../../../target/wasm32-unknown-unknown/debug/test_wasm.wasm");
//     let wasm_module = include_bytes!("../../../CTestMod/test.wasm");
//
//     let mut store = Store::default();
//     let module = Module::new(&store, wasm_module).unwrap();
//     let instance = Instance::new(&mut store, &module, &wasmer::imports! {}).unwrap();
//     println!("{:?}", instance);
//
//     let memory = instance.exports.get_memory("memory").unwrap();
//     let heap_base_global = instance.exports.get_global("__heap_base").unwrap();
//     let heap_base = heap_base_global.get(&mut store).i32().unwrap() as u32;
//
//     {
//         let view = memory.view(&store);
//         let current_pages = view.size();
//         let required_bytes = heap_base + input_bytes.len() as u32;
//         let required_pages = (required_bytes / 65536) + 1;
//         if required_pages > current_pages.0 {
//             let delta = required_pages - current_pages.0;
//             memory.grow(&mut store, delta as u32).unwrap();
//         }
//     }
//
//     let view = memory.view(&store);
//     view.write(heap_base as u64, input_bytes).unwrap();
//
//     println!("Store: {:?}", store);
//     println!("Memory: {:?}", memory);
//
//     println!(" --- CALLING NOW ---");
//
//     let func: TypedFunction<(u32, u32), u64> =
//         instance.exports.get_typed_function(&store, "test").unwrap();
//     let packed_result = func
//         .call(&mut store, heap_base, input_bytes.len() as u32)
//         .unwrap();
//     let output_ptr = (packed_result >> 32) as u32;
//     let output_len = (packed_result & 0xFFFFFFFF) as u32;
//     println!("Raw result: {}", packed_result);
//     println!("Output pointer: {}", output_ptr);
//     println!("Output length: {}", output_len);
//
//     let mut result_bytes = vec![0u8; output_len as usize];
//     let view = memory.view(&store);
//     view.read(output_ptr as u64, &mut result_bytes).unwrap();
//     println!("Result binary extracted: {:?}", result_bytes);
//
//     println!("--- Results ---");
//     let result_position = unsafe { std::ptr::read(result_bytes.as_ptr() as *const Position) };
//     println!("WASM Result: {:?}", result_position);
//
//     println!("Input was: {:?}", input);
//     println!("Expecting result to be 0.123 + 128 = {}", 0.123 + 123.0);
//     if result_position.x == 123.123 {
//         println!("Test PASSED!");
//     } else {
//         println!("Correct type, but invalid result ...");
//     }
// }
//
// #[cfg(test)]
// mod tests {
//     use crate::{Component, Variant};
//
//     #[derive(Debug)]
//     struct Position {
//         x: f32,
//         y: f32,
//         z: f32,
//     }
//
//     impl Component for Position {
//         fn to_variant(&self) -> Variant {
//             Variant::Array(vec![
//                 Variant::F32(self.x),
//                 Variant::F32(self.y),
//                 Variant::F32(self.z),
//             ])
//         }
//
//         fn update_from_variant(&mut self, variant: Variant) {
//             if let Variant::Array(values) = variant
//                 && values.len() == 3
//             {
//                 if let Variant::F32(x) = values[0] {
//                     self.x = x;
//                 }
//
//                 if let Variant::F32(y) = values[1] {
//                     self.y = y;
//                 }
//
//                 if let Variant::F32(z) = values[2] {
//                     self.z = z;
//                 }
//             }
//         }
//     }
//
//     #[test]
//     fn component_to_variant() {
//         let position = Position {
//             x: 1.0,
//             y: 2.0,
//             z: 3.0,
//         };
//         println!("Position: {:?}", position);
//
//         let variant = position.to_variant();
//         println!("Variant: {:?}", variant);
//
//         let Variant::Array(values) = variant else {
//             panic!("Excepted Array!")
//         };
//
//         let Variant::F32(x) = values[0] else {
//             panic!("Expected F32")
//         };
//         assert_eq!(position.x, x);
//
//         let Variant::F32(y) = values[1] else {
//             panic!("Expected F32")
//         };
//         assert_eq!(position.y, y);
//
//         let Variant::F32(z) = values[2] else {
//             panic!("Expected F32")
//         };
//         assert_eq!(position.z, z);
//     }
//
//     #[test]
//     fn update_component_from_variant() {
//         let mut position = Position {
//             x: 1.0,
//             y: 2.0,
//             z: 3.0,
//         };
//         println!("Position: {:?}", position);
//
//         let update_variant = Variant::Array(vec![
//             Variant::F32(position.z),
//             Variant::F32(position.x),
//             Variant::F32(position.y),
//         ]);
//         position.update_from_variant(update_variant);
//         println!("Position: {:?}", position);
//
//         assert_eq!(position.x, 3.0);
//         assert_eq!(position.y, 1.0);
//         assert_eq!(position.z, 2.0);
//     }
// }
