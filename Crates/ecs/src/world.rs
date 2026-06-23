use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::{Component, ComponentStore, ECSError, Entity, WorldComponentStorage};

pub struct World {
    generations: Vec<usize>,
    free_indices: Vec<usize>,
    component_ids: HashMap<TypeId, usize>,
    component_stores: Vec<RwLock<Box<dyn WorldComponentStorage>>>,
    resources: HashMap<TypeId, RwLock<Box<dyn Any + Send + Sync>>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            component_ids: HashMap::new(),
            component_stores: Vec::new(),
            generations: Vec::new(),
            free_indices: Vec::new(),
            resources: HashMap::new(),
        }
    }

    pub fn insert_resource<T: 'static + Send + Sync>(&mut self, resource: T) {
        let type_id = TypeId::of::<T>();
        self.resources
            .insert(type_id, RwLock::new(Box::new(resource)));
    }

    pub fn get_resource<T: 'static + Send + Sync>(&self) -> Option<ResourceHandle<'_, T>> {
        let lock = self.resources.get(&TypeId::of::<T>())?;
        let guard = lock.read().ok()?;
        let ptr: *const T = (*guard).downcast_ref::<T>()?;
        Some(ResourceHandle { _guard: guard, ptr })
    }

    pub fn get_resource_mut<T: 'static + Send + Sync>(
        &self,
    ) -> Option<ResourceMutHandle<'_, T>> {
        let lock = self.resources.get(&TypeId::of::<T>())?;
        let mut guard = lock.write().ok()?;
        let ptr: *mut T = (*guard).downcast_mut::<T>()?;
        Some(ResourceMutHandle { _guard: guard, ptr })
    }

    pub fn is_valid(&self, entity: &Entity) -> bool {
        let idx = entity.index;
        idx < self.generations.len() && self.generations[idx] == entity.generation
    }

    pub fn spawn_entity(&mut self) -> Entity {
        let index = if let Some(idx) = self.free_indices.pop() {
            idx
        } else {
            let new_idx = self.generations.len();
            self.generations.push(0);
            new_idx
        };
        Entity::new(index, self.generations[index])
    }

    pub fn despawn_entity(&mut self, entity: &Entity) {
        if !self.is_valid(entity) {
            return;
        }
        self.generations[entity.index] = self.generations[entity.index].wrapping_add(1);
        for store in &mut self.component_stores {
            if let Ok(store) = store.get_mut() {
                store.remove_entity(entity.index);
            }
        }
        self.free_indices.push(entity.index);
    }

    pub fn attach_component<C: Component>(
        &mut self,
        entity: &Entity,
        component: C,
    ) -> Result<(), ECSError> {
        if !self.is_valid(entity) {
            return Err(ECSError::InvalidEntity(*entity));
        }

        let type_id = TypeId::of::<C>();
        let store_idx = if let Some(&idx) = self.component_ids.get(&type_id) {
            idx
        } else {
            let idx = self.component_stores.len();
            self.component_stores
                .push(RwLock::new(Box::new(ComponentStore::<C>::new())));
            self.component_ids.insert(type_id, idx);
            idx
        };

        let store = self.component_stores[store_idx]
            .get_mut()
            .expect("RwLock poisoned");
        let typed_store = store
            .as_any_mut()
            .downcast_mut::<ComponentStore<C>>()
            .expect("Unexpected downcasting failure at ComponentStore");
        typed_store.attach(entity.index, component);

        Ok(())
    }

    pub fn detach_component<C: Component>(
        &mut self,
        entity: &Entity,
    ) -> Result<(), ECSError> {
        if !self.is_valid(entity) {
            return Err(ECSError::InvalidEntity(*entity));
        }

        let type_id = TypeId::of::<C>();
        let store_idx = *self
            .component_ids
            .get(&type_id)
            .ok_or(ECSError::ComponentStoreNotExisting)?;

        let store = self.component_stores[store_idx]
            .get_mut()
            .expect("RwLock poisoned");
        store.remove_entity(entity.index);

        Ok(())
    }

    pub fn component_id<C: Component>(&self) -> Option<usize> {
        self.component_ids.get(&TypeId::of::<C>()).copied()
    }

    pub fn get_component_store<C: Component>(&self) -> Option<ReadStoreHandle<'_, C>> {
        let idx = *self.component_ids.get(&TypeId::of::<C>())?;
        let guard = self.component_stores[idx]
            .read()
            .expect("RwLock poisoned");
        let ptr: *const ComponentStore<C> = (*guard)
            .as_any()
            .downcast_ref::<ComponentStore<C>>()?;
        Some(ReadStoreHandle {
            _guard: guard,
            ptr,
            _marker: PhantomData,
        })
    }

    pub fn get_component_store_mut<C: Component>(
        &self,
    ) -> Option<WriteStoreHandle<'_, C>> {
        let idx = *self.component_ids.get(&TypeId::of::<C>())?;
        let mut guard = self.component_stores[idx]
            .write()
            .expect("RwLock poisoned");
        let ptr: *mut ComponentStore<C> = (*guard)
            .as_any_mut()
            .downcast_mut::<ComponentStore<C>>()?;
        Some(WriteStoreHandle {
            _guard: guard,
            ptr,
            _marker: PhantomData,
        })
    }
}

impl Debug for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("World")
            .field("generations", &self.generations)
            .field("free_indices", &self.free_indices)
            .field("component_ids", &self.component_ids)
            .field("component_stores", &self.component_stores)
            .field("resource_count", &self.resources.len())
            .finish()
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ReadStoreHandle<'a, C: Component> {
    _guard: RwLockReadGuard<'a, Box<dyn WorldComponentStorage>>,
    ptr: *const ComponentStore<C>,
    _marker: PhantomData<&'a C>,
}

impl<C: Component> Deref for ReadStoreHandle<'_, C> {
    type Target = ComponentStore<C>;
    fn deref(&self) -> &ComponentStore<C> {
        unsafe { &*self.ptr }
    }
}

impl<C: Component> std::fmt::Debug for ReadStoreHandle<'_, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReadStoreHandle")
            .field("store", unsafe { &*self.ptr })
            .finish()
    }
}

pub struct WriteStoreHandle<'a, C: Component> {
    _guard: RwLockWriteGuard<'a, Box<dyn WorldComponentStorage>>,
    ptr: *mut ComponentStore<C>,
    _marker: PhantomData<&'a C>,
}

impl<C: Component> Deref for WriteStoreHandle<'_, C> {
    type Target = ComponentStore<C>;
    fn deref(&self) -> &ComponentStore<C> {
        unsafe { &*self.ptr }
    }
}

impl<C: Component> WriteStoreHandle<'_, C> {
    /// Gets a mutable reference to the inner store.
    /// Safe because the RwLockWriteGuard ensures exclusive access.
    pub fn get_mut_store(&self) -> &mut ComponentStore<C> {
        unsafe { &mut *self.ptr }
    }
}

impl<C: Component> DerefMut for WriteStoreHandle<'_, C> {
    fn deref_mut(&mut self) -> &mut ComponentStore<C> {
        unsafe { &mut *self.ptr }
    }
}

impl<C: Component> std::fmt::Debug for WriteStoreHandle<'_, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WriteStoreHandle")
            .field("store", unsafe { &*self.ptr })
            .finish()
    }
}

pub struct ResourceHandle<'a, T: 'static> {
    _guard: RwLockReadGuard<'a, Box<dyn Any + Send + Sync>>,
    ptr: *const T,
}

impl<T: 'static> Deref for ResourceHandle<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.ptr }
    }
}

impl<T: 'static + std::fmt::Debug> std::fmt::Debug for ResourceHandle<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceHandle")
            .field("value", unsafe { &*self.ptr })
            .finish()
    }
}

pub struct ResourceMutHandle<'a, T: 'static> {
    _guard: RwLockWriteGuard<'a, Box<dyn Any + Send + Sync>>,
    ptr: *mut T,
}

impl<T: 'static> Deref for ResourceMutHandle<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.ptr }
    }
}

impl<T: 'static> DerefMut for ResourceMutHandle<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.ptr }
    }
}

impl<T: 'static + std::fmt::Debug> std::fmt::Debug for ResourceMutHandle<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceMutHandle")
            .field("value", unsafe { &*self.ptr })
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_entity() {
        let mut world = World::new();
        let entity = world.spawn_entity();
        assert_eq!(0, entity.index);
        assert_eq!(0, entity.generation);
    }

    #[test]
    fn despawn_entity() {
        let mut world = World::new();

        let entity_0 = world.spawn_entity();
        assert_eq!(0, entity_0.index);
        assert_eq!(0, entity_0.generation);
        world
            .attach_component(&entity_0, String::from("First"))
            .expect("Attachment failure");
        let entity_1 = world.spawn_entity();
        assert_eq!(1, entity_1.index);
        assert_eq!(0, entity_1.generation);
        world
            .attach_component(&entity_1, String::from("Second"))
            .expect("Attachment failure");
        let entity_2 = world.spawn_entity();
        assert_eq!(2, entity_2.index);
        assert_eq!(0, entity_2.generation);
        world
            .attach_component(&entity_2, String::from("Third"))
            .expect("Attachment failure");

        world.despawn_entity(&entity_1);

        let store = world
            .get_component_store::<String>()
            .expect("Store failure");
        assert_eq!(
            store.get_component(entity_0.index),
            Some(&String::from("First"))
        );
        assert_ne!(
            store.get_component(entity_1.index),
            Some(&String::from("Second"))
        );
        assert_eq!(
            store.get_component(entity_2.index),
            Some(&String::from("Third"))
        );
    }

    #[test]
    fn test_index_reuse_and_generation_increment() {
        let mut world = World::new();

        let e1 = world.spawn_entity();
        let idx1 = e1.index;
        let gen1 = e1.generation;
        assert_eq!(idx1, 0);
        assert_eq!(gen1, 0);

        world.despawn_entity(&e1);

        let e2 = world.spawn_entity();
        assert_eq!(e2.index, idx1, "Should reuse the freed index");
        assert_ne!(e2.generation, gen1, "Generation should have incremented");
        assert_eq!(e2.generation, 1);
    }

    #[test]
    fn test_stale_handle_invalidation() {
        let mut world = World::new();

        let e1 = world.spawn_entity();
        world.despawn_entity(&e1);

        assert!(
            !world.is_valid(&e1),
            "Handle should be invalid after despawn"
        );

        let result = world.attach_component(&e1, String::from("Ghost"));
        assert!(
            result.is_err(),
            "Should not allow attaching components to stale handles"
        );
    }

    #[test]
    fn test_complex_reuse_pattern() {
        let mut world = World::new();

        let e0 = world.spawn_entity();
        let e1 = world.spawn_entity();
        let e2 = world.spawn_entity();

        world.despawn_entity(&e1);

        let e1_new = world.spawn_entity();
        assert_eq!(e1_new.index, e1.index);
        assert_eq!(e1_new.generation, 1);

        assert!(world.is_valid(&e0));
        assert!(world.is_valid(&e2));
        assert!(world.is_valid(&e1_new));

        assert!(!world.is_valid(&e1));
    }

    #[test]
    fn test_out_of_bounds_validation() {
        let world = World::new();
        let fake_entity = Entity::new(999, 0);
        assert!(
            !world.is_valid(&fake_entity),
            "Out of bounds index should be invalid"
        );
    }

    #[test]
    fn test_multiple_despawns_and_recycles() {
        let mut world = World::new();

        let e1 = world.spawn_entity();
        let _e2 = world.spawn_entity();
        let e3 = world.spawn_entity();

        world.despawn_entity(&e1);
        world.despawn_entity(&e3);

        let e4 = world.spawn_entity();
        assert!(world.is_valid(&e4));

        assert!(!world.is_valid(&e1));
        assert!(!world.is_valid(&e3));
    }

    #[test]
    fn test_attach_detach_on_valid_entities() {
        let mut world = World::new();
        let e = world.spawn_entity();

        let res_attach = world.attach_component(&e, String::from("Data"));
        assert!(res_attach.is_ok());

        let res_detach = world.detach_component::<String>(&e);
        assert!(res_detach.is_ok());
    }
}
