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

    pub fn get_resource_mut<T: 'static + Send + Sync>(&self) -> Option<ResourceMutHandle<'_, T>> {
        let lock = self.resources.get(&TypeId::of::<T>())?;
        let mut guard = lock.write().ok()?;
        let ptr: *mut T = (*guard).downcast_mut::<T>()?;
        Some(ResourceMutHandle { _guard: guard, ptr })
    }

    pub fn is_valid(&self, entity: &Entity) -> bool {
        let idx = entity.index;
        idx < self.generations.len() && self.generations[idx] == entity.generation
    }

    /// Get the current generation for an entity index.
    /// Returns 0 if the index has never been used.
    pub fn generation(&self, index: usize) -> usize {
        self.generations.get(index).copied().unwrap_or(0)
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

    pub fn detach_component<C: Component>(&mut self, entity: &Entity) -> Result<(), ECSError> {
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
        let guard = self.component_stores[idx].read().expect("RwLock poisoned");
        let ptr: *const ComponentStore<C> =
            (*guard).as_any().downcast_ref::<ComponentStore<C>>()?;
        Some(ReadStoreHandle {
            _guard: guard,
            ptr,
            _marker: PhantomData,
        })
    }

    pub fn get_component_store_mut<C: Component>(&self) -> Option<WriteStoreHandle<'_, C>> {
        let idx = *self.component_ids.get(&TypeId::of::<C>())?;
        let mut guard = self.component_stores[idx].write().expect("RwLock poisoned");
        let ptr: *mut ComponentStore<C> =
            (*guard).as_any_mut().downcast_mut::<ComponentStore<C>>()?;
        Some(WriteStoreHandle {
            _guard: guard,
            ptr,
            _marker: PhantomData,
        })
    }

    /// The number of entities that are currently alive.
    ///
    /// This is `generations.len() - free_indices.len()`; it counts indices that
    /// have been used, including any that were despawned and are awaiting reuse.
    pub fn entity_count(&self) -> usize {
        self.generations.len() - self.free_indices.len()
    }

    /// Iterates over every live entity.
    ///
    /// Despawned indices are skipped, and each yielded [`Entity`] carries the
    /// current generation, so the result can be fed back into
    /// [`is_valid`](World::is_valid) or component accessors.
    pub fn entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.generations
            .iter()
            .enumerate()
            .filter(|(index, _)| !self.free_indices.contains(index))
            .map(|(index, generation)| Entity::new(index, *generation))
    }

    /// Iterates over every registered component store, paired with the
    /// [`TypeId`] of the component type it holds.
    ///
    /// A store is registered lazily on the first
    /// [`attach_component`](World::attach_component) for its component type, so
    /// this only ever yields types that have actually been attached at least
    /// once.
    ///
    /// The iteration order is unspecified and may differ between calls. Sort
    /// by [`WorldComponentStorage::component_type_name`] if you need a stable
    /// order.
    pub fn component_stores(
        &self,
    ) -> impl Iterator<Item = (TypeId, &RwLock<Box<dyn WorldComponentStorage>>)> + '_ {
        self.component_ids
            .iter()
            .map(|(type_id, index)| (*type_id, &self.component_stores[*index]))
    }

    /// Looks up the [`TypeId`] of a component type without attaching one.
    pub fn type_id_of<C: 'static>(&self) -> Option<TypeId> {
        let type_id = TypeId::of::<C>();
        self.component_ids.contains_key(&type_id).then_some(type_id)
    }

    /// The value of the component of type `type_id` held by `entity`, as a
    /// type-erased [`Debug`].
    ///
    /// Returns `None` if the type has no store, or the entity does not hold it.
    /// This is the read path an inspector uses to display a component it has
    /// no static knowledge of.
    ///
    /// The returned handle keeps the store's read lock alive, so do not call
    /// [`set_component_any`](World::set_component_any) or
    /// [`insert_component_any`](World::insert_component_any) on the same
    /// component type while it is alive.
    pub fn component_debug(
        &self,
        type_id: TypeId,
        entity: &Entity,
    ) -> Option<ReadComponentHandle<'_>> {
        let store_idx = *self.component_ids.get(&type_id)?;
        let guard = self.component_stores[store_idx]
            .read()
            .expect("RwLock poisoned");

        guard.component_any(entity.index)?;

        let store: *const dyn WorldComponentStorage = &**guard;

        Some(ReadComponentHandle {
            _guard: guard,
            store,
            index: entity.index,
        })
    }

    /// [`component_debug`](World::component_debug) for `entity`, formatted
    /// with [`Debug`].
    ///
    /// A convenience wrapper for callers that just want to show the value, such
    /// as a debug inspector, and would rather not hold the read lock that
    /// [`component_debug`](World::component_debug) keeps alive.
    pub fn component_debug_string(&self, type_id: TypeId, entity: &Entity) -> Option<String> {
        Some(format!("{:?}", self.component_debug(type_id, entity)?))
    }

    /// Whether `entity` holds a component of type `type_id`.
    pub fn entity_has_component(&self, type_id: TypeId, entity: &Entity) -> bool {
        let Some(store_idx) = self.component_ids.get(&type_id) else {
            return false;
        };

        let Ok(store) = self.component_stores[*store_idx].read() else {
            return false;
        };

        store.component_any(entity.index).is_some()
    }

    /// Every component type currently held by `entity`, paired with its Rust
    /// type name.
    ///
    /// Sorted by type name, so the result is stable across calls. This is the
    /// "expand an entity" step of an inspector: the caller still needs
    /// [`component_debug`](World::component_debug) to read each value, but it
    /// does not have to know `TypeId`s up front.
    pub fn entity_components(&self, entity: &Entity) -> Vec<(TypeId, &'static str)> {
        let mut components = self
            .component_stores()
            .filter_map(|(_, store)| {
                let guard = store.read().ok()?;
                guard
                    .entity_indices()
                    .contains(&entity.index)
                    .then(|| (guard.component_type_id(), guard.component_type_name()))
            })
            .collect::<Vec<_>>();

        components.sort_unstable_by(|(_, left), (_, right)| left.cmp(right));
        components
    }

    /// Replaces the component of type `type_id` held by `entity` with
    /// `component`.
    ///
    /// `component` is downcast to the type the store holds; a type mismatch
    /// returns `Err(ECSError::ComponentTypeMismatch)` and leaves the world
    /// untouched. Attaching a component the entity does not already have
    /// returns `Err(ECSError::ComponentNotAttached)` — use
    /// [`insert_component_any`](World::insert_component_any) to add one.
    pub fn set_component_any(
        &self,
        type_id: TypeId,
        entity: &Entity,
        component: Box<dyn Any + Send + Sync>,
    ) -> Result<(), ECSError> {
        if !self.is_valid(entity) {
            return Err(ECSError::InvalidEntity(*entity));
        }

        let store_idx = *self
            .component_ids
            .get(&type_id)
            .ok_or(ECSError::ComponentStoreNotExisting)?;

        let mut store = self.component_stores[store_idx]
            .write()
            .expect("RwLock poisoned");

        if !store.entity_indices().contains(&entity.index) {
            return Err(ECSError::ComponentNotAttached(*entity));
        }

        if store.set_component_any(entity.index, component) {
            Ok(())
        } else {
            Err(ECSError::ComponentTypeMismatch(type_id))
        }
    }

    /// Attaches a component of type `type_id` to `entity`, or replaces the
    /// existing one.
    ///
    /// Behaves like [`set_component_any`](World::set_component_any) except that
    /// it does not require the entity to already hold the component.
    pub fn insert_component_any(
        &self,
        type_id: TypeId,
        entity: &Entity,
        component: Box<dyn Any + Send + Sync>,
    ) -> Result<(), ECSError> {
        if !self.is_valid(entity) {
            return Err(ECSError::InvalidEntity(*entity));
        }

        let store_idx = *self
            .component_ids
            .get(&type_id)
            .ok_or(ECSError::ComponentStoreNotExisting)?;

        let mut store = self.component_stores[store_idx]
            .write()
            .expect("RwLock poisoned");

        if store.set_or_insert_component_any(entity.index, component) {
            Ok(())
        } else {
            Err(ECSError::ComponentTypeMismatch(type_id))
        }
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

/// A type-erased, read-locked view of a single component value.
///
/// Returned by [`World::component_debug`]. Dereferences to `dyn Debug`, so the
/// value can be formatted without knowing its concrete type. Holding one keeps
/// the owning store read-locked for as long as the handle lives, which is what
/// makes it sound to hand out a reference without a concrete lifetime.
pub struct ReadComponentHandle<'a> {
    _guard: RwLockReadGuard<'a, Box<dyn WorldComponentStorage>>,
    store: *const dyn WorldComponentStorage,
    index: usize,
}

impl<'a> ReadComponentHandle<'a> {
    /// The value as a type-erased [`Any`], for callers that *do* know the
    /// concrete component type and want to
    /// [`downcast_ref`](Any::downcast_ref) it.
    pub fn as_any(&self) -> &'a dyn Any {
        // SAFETY: see the `Deref` impl; the guard keeps the value alive for `'a`.
        unsafe {
            (*self.store)
                .component_any(self.index)
                .expect("component disappeared while its read lock was held")
        }
    }
}

impl<'a> Deref for ReadComponentHandle<'a> {
    type Target = dyn Debug + 'a;

    fn deref(&self) -> &Self::Target {
        // SAFETY: `store` points at the `WorldComponentStorage` owned by
        // `_guard`, which is held for `'a`, so the value can neither be removed
        // nor replaced while this handle is alive.
        unsafe {
            (*self.store)
                .component_debug(self.index)
                .expect("component disappeared while its read lock was held")
        }
    }
}

impl std::fmt::Debug for ReadComponentHandle<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&**self, f)
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
    #[allow(clippy::mut_from_ref)]
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

    #[derive(Debug, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }

    #[derive(Debug, PartialEq)]
    struct Health(u32);

    #[test]
    fn entity_enumeration_skips_despawned_and_reports_count() {
        let mut world = World::new();

        let e0 = world.spawn_entity();
        let e1 = world.spawn_entity();
        let e2 = world.spawn_entity();

        assert_eq!(world.entity_count(), 3);
        assert_eq!(world.entities().collect::<Vec<_>>(), vec![e0, e1, e2]);

        world.despawn_entity(&e1);

        assert_eq!(world.entity_count(), 2);
        assert_eq!(world.entities().collect::<Vec<_>>(), vec![e0, e2]);

        // The freed index is reused, with a bumped generation.
        let e1_new = world.spawn_entity();
        assert_eq!(e1_new.index, e1.index);
        assert_eq!(e1_new.generation, 1);
        assert_eq!(world.entities().collect::<Vec<_>>(), vec![e0, e1_new, e2]);
    }

    #[test]
    fn component_store_enumeration_reports_types() {
        let mut world = World::new();
        let e = world.spawn_entity();

        assert_eq!(world.component_stores().count(), 0);

        world
            .attach_component(&e, Position { x: 1.0, y: 2.0 })
            .expect("Attachment failure");
        world
            .attach_component(&e, Health(7))
            .expect("Attachment failure");

        let mut stores = world
            .component_stores()
            .map(|(_, store)| {
                let guard = store.read().expect("RwLock poisoned");
                (
                    guard.component_type_id(),
                    guard.component_type_name(),
                    guard.entity_indices().to_vec(),
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(stores.len(), 2);
        stores.sort_unstable_by(|(_, left, _), (_, right, _)| left.cmp(right));

        assert_eq!(stores[0].0, TypeId::of::<Health>());
        assert_eq!(stores[0].1, "orbital_ecs::world::tests::Health");
        assert_eq!(stores[0].2, vec![e.index]);
        assert_eq!(stores[1].0, TypeId::of::<Position>());
        assert_eq!(stores[1].1, "orbital_ecs::world::tests::Position");
        assert_eq!(stores[1].2, vec![e.index]);

        assert!(world.type_id_of::<Position>().is_some());
        assert!(world.type_id_of::<u8>().is_none());
    }

    #[test]
    fn entity_components_lists_only_held_types_sorted_by_name() {
        let mut world = World::new();
        let e0 = world.spawn_entity();
        let e1 = world.spawn_entity();

        world
            .attach_component(&e0, Position { x: 1.0, y: 2.0 })
            .expect("Attachment failure");
        world
            .attach_component(&e1, Position { x: 3.0, y: 4.0 })
            .expect("Attachment failure");
        world
            .attach_component(&e1, Health(9))
            .expect("Attachment failure");

        assert_eq!(
            world.entity_components(&e0),
            vec![(
                TypeId::of::<Position>(),
                "orbital_ecs::world::tests::Position"
            )]
        );

        // Sorted by name: "…::Health" before "…::Position".
        assert_eq!(
            world.entity_components(&e1),
            vec![
                (TypeId::of::<Health>(), "orbital_ecs::world::tests::Health"),
                (
                    TypeId::of::<Position>(),
                    "orbital_ecs::world::tests::Position"
                ),
            ]
        );

        // Stable across repeated calls despite the HashMap-backed store index.
        assert_eq!(world.entity_components(&e1), world.entity_components(&e1));
    }

    #[test]
    fn dynamic_component_read() {
        let mut world = World::new();
        let e = world.spawn_entity();
        let other = world.spawn_entity();

        world
            .attach_component(&e, Position { x: 1.5, y: -2.5 })
            .expect("Attachment failure");

        let position = TypeId::of::<Position>();

        assert!(world.entity_has_component(position, &e));
        assert!(!world.entity_has_component(position, &other));
        assert!(!world.entity_has_component(TypeId::of::<Health>(), &e));

        assert_eq!(
            format!(
                "{:?}",
                world.component_debug(position, &e).expect("Missing")
            ),
            "Position { x: 1.5, y: -2.5 }"
        );
        assert_eq!(
            world.component_debug_string(position, &e),
            Some("Position { x: 1.5, y: -2.5 }".to_string())
        );

        // Missing entity, and a type that has no store at all.
        assert!(world.component_debug(position, &other).is_none());
        assert!(world.component_debug(TypeId::of::<Health>(), &e).is_none());

        // The type-erased value can be downcast back to the concrete type.
        let handle = world.component_debug(position, &e).expect("Missing");
        assert_eq!(
            handle
                .as_any()
                .downcast_ref::<Position>()
                .expect("Downcast failed"),
            &Position { x: 1.5, y: -2.5 }
        );
    }

    #[test]
    fn dynamic_component_write() {
        let mut world = World::new();
        let e = world.spawn_entity();
        let bare = world.spawn_entity();

        let position = TypeId::of::<Position>();
        let health = TypeId::of::<Health>();

        // No store registered for the type at all.
        assert!(matches!(
            world.set_component_any(position, &e, Box::new(Position { x: 0.0, y: 0.0 })),
            Err(ECSError::ComponentStoreNotExisting)
        ));

        // Store exists, but this entity does not hold the component yet.
        world
            .attach_component(&e, Position { x: 1.0, y: 2.0 })
            .expect("Attachment failure");
        assert!(matches!(
            world.set_component_any(position, &bare, Box::new(Position { x: 0.0, y: 0.0 })),
            Err(ECSError::ComponentNotAttached(_))
        ));

        world
            .set_component_any(position, &e, Box::new(Position { x: 9.0, y: 8.0 }))
            .expect("Set failure");

        assert_eq!(
            world.component_debug_string(position, &e),
            Some("Position { x: 9.0, y: 8.0 }".to_string())
        );

        // `insert_component_any` replaces rather than duplicating.
        world
            .insert_component_any(position, &e, Box::new(Position { x: 0.5, y: 0.5 }))
            .expect("Insert failure");
        assert_eq!(
            world.component_debug_string(position, &e),
            Some("Position { x: 0.5, y: 0.5 }".to_string())
        );

        // A type mismatch is rejected and leaves the value untouched.
        assert!(matches!(
            world.set_component_any(position, &e, Box::new(Health(1))),
            Err(ECSError::ComponentTypeMismatch(_))
        ));
        assert!(matches!(
            world.insert_component_any(position, &e, Box::new(Health(1))),
            Err(ECSError::ComponentTypeMismatch(_))
        ));
        assert_eq!(
            world.component_debug_string(position, &e),
            Some("Position { x: 0.5, y: 0.5 }".to_string())
        );

        // No store registered for the type.
        assert!(matches!(
            world.set_component_any(health, &e, Box::new(Health(1))),
            Err(ECSError::ComponentStoreNotExisting)
        ));
        // Stale entity.
        world.despawn_entity(&bare);
        assert!(matches!(
            world.set_component_any(position, &bare, Box::new(Position { x: 0.0, y: 0.0 })),
            Err(ECSError::InvalidEntity(_))
        ));
    }

    #[test]
    fn reattaching_a_component_replaces_it_in_place() {
        let mut world = World::new();
        let e0 = world.spawn_entity();
        let e1 = world.spawn_entity();

        world
            .attach_component(&e0, Health(1))
            .expect("Attachment failure");
        world
            .attach_component(&e1, Health(2))
            .expect("Attachment failure");

        world
            .attach_component(&e0, Health(42))
            .expect("Attachment failure");

        // No orphaned entries: still one value per entity.
        let store = world
            .get_component_store::<Health>()
            .expect("Store failure");
        assert_eq!(store.dense.len(), 2);
        assert_eq!(store.components.len(), 2);

        assert_eq!(store.get_component(e0.index), Some(&Health(42)));
        assert_eq!(store.get_component(e1.index), Some(&Health(2)));

        // And the reverse lookup still maps both entities.
        let mut indices = store.dense.clone();
        indices.sort_unstable();
        assert_eq!(indices, vec![e0.index, e1.index]);
    }
}
