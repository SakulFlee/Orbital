use std::marker::PhantomData;

use crate::{Component, World};

// ---------------------------------------------------------------------------
// Marker types
// ---------------------------------------------------------------------------

pub struct Read<T>(PhantomData<T>);
pub struct Write<T>(PhantomData<T>);
pub struct With<T>(PhantomData<T>);
pub struct Without<T>(PhantomData<T>);

// ---------------------------------------------------------------------------
// QueryData trait — defines what components a query fetches
// ---------------------------------------------------------------------------

pub trait QueryData: Sized {
    type Item<'a>;
    type State<'a>;
    fn init_state<'a>(world: &'a World) -> Self::State<'a>;
    fn pivot_dense<'a, 'b>(state: &'a Self::State<'b>) -> &'a [usize];
    fn get_item<'a, 'b>(state: &'a Self::State<'b>, entity_id: usize) -> Option<Self::Item<'a>>;
}

// ---------------------------------------------------------------------------
// QueryFilter trait — defines With/Without filters
// ---------------------------------------------------------------------------

pub trait QueryFilter: Sized {
    type State<'a>;
    fn init_state<'a>(world: &'a World) -> Self::State<'a>;
    fn matches<'a, 'b>(state: &'a Self::State<'b>, entity_id: usize) -> bool;
}

pub struct NoFilter;

impl QueryFilter for NoFilter {
    type State<'a> = ();
    fn init_state<'a>(_world: &'a World) -> Self::State<'a> {}
    fn matches<'a, 'b>(_state: &'a Self::State<'b>, _entity_id: usize) -> bool {
        true
    }
}

// ---------------------------------------------------------------------------
// Query and QueryIter
// ---------------------------------------------------------------------------

pub struct Query<'w, D: QueryData, F: QueryFilter = NoFilter> {
    state: D::State<'w>,
    filter: F::State<'w>,
}

impl<'w, D: QueryData, F: QueryFilter> Query<'w, D, F> {
    pub fn new(world: &'w World) -> Self {
        Self {
            state: D::init_state(world),
            filter: F::init_state(world),
        }
    }

    pub fn iter<'a>(&'a mut self) -> QueryIter<'a, 'w, D, F> {
        let pivot = D::pivot_dense(&self.state);
        QueryIter {
            state: &self.state,
            filter: &self.filter,
            pivot,
            cursor: 0,
        }
    }
}

pub struct QueryIter<'a, 'b: 'a, D: QueryData, F: QueryFilter> {
    state: &'a D::State<'b>,
    filter: &'a F::State<'b>,
    pivot: &'a [usize],
    cursor: usize,
}

impl<'a, 'b: 'a, D: QueryData, F: QueryFilter> Iterator for QueryIter<'a, 'b, D, F> {
    type Item = D::Item<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.cursor < self.pivot.len() {
            let entity_id = self.pivot[self.cursor];
            self.cursor += 1;

            if !F::matches(self.filter, entity_id) {
                continue;
            }

            if let Some(item) = D::get_item(self.state, entity_id) {
                return Some(item);
            }
        }
        None
    }
}

// ---------------------------------------------------------------------------
// Helper macros
// ---------------------------------------------------------------------------

macro_rules! fetch_item {
    (Read, $handle:expr, $idx:expr) => {
        &$handle.components[$idx]
    };
    (Write, $handle:expr, $idx:expr) => {
        &mut $handle.get_mut_store().components[$idx]
    };
}

macro_rules! fetch_store {
    (Read, $world:expr, $T:ty) => {
        $world.get_component_store::<$T>().unwrap()
    };
    (Write, $world:expr, $T:ty) => {
        $world.get_component_store_mut::<$T>().unwrap()
    };
}

macro_rules! fetch_item_ty {
    (Read, $l:lifetime, $T:ty) => {
        & $l $T
    };
    (Write, $l:lifetime, $T:ty) => {
        & $l mut $T
    };
}

macro_rules! fetch_state_ty {
    (Read, $l:lifetime, $T:ty) => {
        crate::ReadStoreHandle<$l, $T>
    };
    (Write, $l:lifetime, $T:ty) => {
        crate::WriteStoreHandle<$l, $T>
    };
}

// ---------------------------------------------------------------------------
// Macro to implement QueryData for a tuple of Read/Write components
// ---------------------------------------------------------------------------

macro_rules! impl_query_data {
    // 1 element
    ($idx0:tt => $acc0:ident < $t0:ident>) => {
        impl<$t0: Component> QueryData for ($acc0<$t0>,) {
            type Item<'a> = (fetch_item_ty!($acc0, 'a, $t0),);
            type State<'a> = (fetch_state_ty!($acc0, 'a, $t0),);
            fn init_state<'a>(world: &'a World) -> Self::State<'a> {
                (fetch_store!($acc0, world, $t0),)
            }
            fn pivot_dense<'a, 'b>(state: &'a Self::State<'b>) -> &'a [usize] {
                &state.0.dense
            }
            fn get_item<'a, 'b>(
                state: &'a Self::State<'b>,
                entity_id: usize,
            ) -> Option<Self::Item<'a>> {
                let idx0 = state.0.sparse.get(entity_id).copied().flatten()?;
                Some((fetch_item!($acc0, state.0, idx0),))
            }
        }
    };

    // 2 elements
    ($idx0:tt => $acc0:ident < $t0:ident>, $idx1:tt => $acc1:ident < $t1:ident>) => {
        impl<$t0: Component, $t1: Component> QueryData for ($acc0<$t0>, $acc1<$t1>) {
            type Item<'a> = (
                fetch_item_ty!($acc0, 'a, $t0),
                fetch_item_ty!($acc1, 'a, $t1),
            );
            type State<'a> = (
                fetch_state_ty!($acc0, 'a, $t0),
                fetch_state_ty!($acc1, 'a, $t1),
            );
            fn init_state<'a>(world: &'a World) -> Self::State<'a> {
                (
                    fetch_store!($acc0, world, $t0),
                    fetch_store!($acc1, world, $t1),
                )
            }
            fn pivot_dense<'a, 'b>(state: &'a Self::State<'b>) -> &'a [usize] {
                if state.0.dense.len() <= state.1.dense.len() {
                    &state.0.dense
                } else {
                    &state.1.dense
                }
            }
            fn get_item<'a, 'b>(
                state: &'a Self::State<'b>,
                entity_id: usize,
            ) -> Option<Self::Item<'a>> {
                let idx0 = state.0.sparse.get(entity_id).copied().flatten()?;
                let idx1 = state.1.sparse.get(entity_id).copied().flatten()?;
                Some((
                    fetch_item!($acc0, state.0, idx0),
                    fetch_item!($acc1, state.1, idx1),
                ))
            }
        }
    };

    // 3 elements
    ($idx0:tt => $acc0:ident < $t0:ident>, $idx1:tt => $acc1:ident < $t1:ident>, $idx2:tt => $acc2:ident < $t2:ident>) => {
        impl<$t0: Component, $t1: Component, $t2: Component> QueryData
            for ($acc0<$t0>, $acc1<$t1>, $acc2<$t2>)
        {
            type Item<'a> = (
                fetch_item_ty!($acc0, 'a, $t0),
                fetch_item_ty!($acc1, 'a, $t1),
                fetch_item_ty!($acc2, 'a, $t2),
            );
            type State<'a> = (
                fetch_state_ty!($acc0, 'a, $t0),
                fetch_state_ty!($acc1, 'a, $t1),
                fetch_state_ty!($acc2, 'a, $t2),
            );
            fn init_state<'a>(world: &'a World) -> Self::State<'a> {
                (
                    fetch_store!($acc0, world, $t0),
                    fetch_store!($acc1, world, $t1),
                    fetch_store!($acc2, world, $t2),
                )
            }
            fn pivot_dense<'a, 'b>(state: &'a Self::State<'b>) -> &'a [usize] {
                let l0 = state.0.dense.len();
                let l1 = state.1.dense.len();
                let l2 = state.2.dense.len();
                if l0 <= l1 && l0 <= l2 {
                    &state.0.dense
                } else if l1 <= l0 && l1 <= l2 {
                    &state.1.dense
                } else {
                    &state.2.dense
                }
            }
            fn get_item<'a, 'b>(
                state: &'a Self::State<'b>,
                entity_id: usize,
            ) -> Option<Self::Item<'a>> {
                let idx0 = state.0.sparse.get(entity_id).copied().flatten()?;
                let idx1 = state.1.sparse.get(entity_id).copied().flatten()?;
                let idx2 = state.2.sparse.get(entity_id).copied().flatten()?;
                Some((
                    fetch_item!($acc0, state.0, idx0),
                    fetch_item!($acc1, state.1, idx1),
                    fetch_item!($acc2, state.2, idx2),
                ))
            }
        }
    };

    // 4 elements
    (
        $idx0:tt => $acc0:ident < $t0:ident>,
        $idx1:tt => $acc1:ident < $t1:ident>,
        $idx2:tt => $acc2:ident < $t2:ident>,
        $idx3:tt => $acc3:ident < $t3:ident>
    ) => {
        impl<
            $t0: Component,
            $t1: Component,
            $t2: Component,
            $t3: Component,
        > QueryData for ($acc0<$t0>, $acc1<$t1>, $acc2<$t2>, $acc3<$t3>) {
            type Item<'a> = (
                fetch_item_ty!($acc0, 'a, $t0),
                fetch_item_ty!($acc1, 'a, $t1),
                fetch_item_ty!($acc2, 'a, $t2),
                fetch_item_ty!($acc3, 'a, $t3),
            );
            type State<'a> = (
                fetch_state_ty!($acc0, 'a, $t0),
                fetch_state_ty!($acc1, 'a, $t1),
                fetch_state_ty!($acc2, 'a, $t2),
                fetch_state_ty!($acc3, 'a, $t3),
            );
            fn init_state<'a>(world: &'a World) -> Self::State<'a> {
                (
                    fetch_store!($acc0, world, $t0),
                    fetch_store!($acc1, world, $t1),
                    fetch_store!($acc2, world, $t2),
                    fetch_store!($acc3, world, $t3),
                )
            }
            fn pivot_dense<'a, 'b>(state: &'a Self::State<'b>) -> &'a [usize] {
                let l0 = state.0.dense.len();
                let l1 = state.1.dense.len();
                let l2 = state.2.dense.len();
                let l3 = state.3.dense.len();
                if l0 <= l1 && l0 <= l2 && l0 <= l3 {
                    &state.0.dense
                } else if l1 <= l0 && l1 <= l2 && l1 <= l3 {
                    &state.1.dense
                } else if l2 <= l0 && l2 <= l1 && l2 <= l3 {
                    &state.2.dense
                } else {
                    &state.3.dense
                }
            }
            fn get_item<'a, 'b>(
                state: &'a Self::State<'b>,
                entity_id: usize,
            ) -> Option<Self::Item<'a>> {
                let idx0 = state.0.sparse.get(entity_id).copied().flatten()?;
                let idx1 = state.1.sparse.get(entity_id).copied().flatten()?;
                let idx2 = state.2.sparse.get(entity_id).copied().flatten()?;
                let idx3 = state.3.sparse.get(entity_id).copied().flatten()?;
                Some((
                    fetch_item!($acc0, state.0, idx0),
                    fetch_item!($acc1, state.1, idx1),
                    fetch_item!($acc2, state.2, idx2),
                    fetch_item!($acc3, state.3, idx3),
                ))
            }
        }
    };
}

// ---------------------------------------------------------------------------
// Generate all Read/Write combinations for 1..4 elements
// ---------------------------------------------------------------------------

impl_query_data!(0 => Read<A>);
impl_query_data!(0 => Write<A>);

impl_query_data!(0 => Read<A>, 1 => Read<B>);
impl_query_data!(0 => Read<A>, 1 => Write<B>);
impl_query_data!(0 => Write<A>, 1 => Read<B>);
impl_query_data!(0 => Write<A>, 1 => Write<B>);

impl_query_data!(0 => Read<A>, 1 => Read<B>, 2 => Read<C>);
impl_query_data!(0 => Read<A>, 1 => Read<B>, 2 => Write<C>);
impl_query_data!(0 => Read<A>, 1 => Write<B>, 2 => Read<C>);
impl_query_data!(0 => Read<A>, 1 => Write<B>, 2 => Write<C>);
impl_query_data!(0 => Write<A>, 1 => Read<B>, 2 => Read<C>);
impl_query_data!(0 => Write<A>, 1 => Read<B>, 2 => Write<C>);
impl_query_data!(0 => Write<A>, 1 => Write<B>, 2 => Read<C>);
impl_query_data!(0 => Write<A>, 1 => Write<B>, 2 => Write<C>);

// ---------------------------------------------------------------------------
// Filter implementations
// ---------------------------------------------------------------------------

impl<T: Component> QueryFilter for With<T> {
    type State<'a> = Option<crate::ReadStoreHandle<'a, T>>;
    fn init_state<'a>(world: &'a World) -> Self::State<'a> {
        world.get_component_store::<T>()
    }
    fn matches<'a, 'b>(state: &'a Self::State<'b>, entity_id: usize) -> bool {
        state
            .as_ref()
            .and_then(|s| s.sparse.get(entity_id))
            .is_some_and(|x| x.is_some())
    }
}

impl<T: Component> QueryFilter for Without<T> {
    type State<'a> = Option<crate::ReadStoreHandle<'a, T>>;
    fn init_state<'a>(world: &'a World) -> Self::State<'a> {
        world.get_component_store::<T>()
    }
    fn matches<'a, 'b>(state: &'a Self::State<'b>, entity_id: usize) -> bool {
        !state
            .as_ref()
            .and_then(|s| s.sparse.get(entity_id))
            .is_some_and(|x| x.is_some())
    }
}

// ---------------------------------------------------------------------------
// Tuple filter impls via macro
// ---------------------------------------------------------------------------

macro_rules! impl_filter_tuple {
    // 1 element
    ($idx0:tt => $f0:ident < $t0:ident>) => {
        impl<$t0: Component> QueryFilter for ($f0<$t0>,) {
            type State<'a> = (<$f0<$t0> as QueryFilter>::State<'a>,);
            fn init_state<'a>(world: &'a World) -> Self::State<'a> {
                (<$f0<$t0> as QueryFilter>::init_state(world),)
            }
            fn matches<'a, 'b>(state: &'a Self::State<'b>, entity_id: usize) -> bool {
                <$f0<$t0> as QueryFilter>::matches(&state.0, entity_id)
            }
        }
    };

    // 2 elements
    ($idx0:tt => $f0:ident < $t0:ident>, $idx1:tt => $f1:ident < $t1:ident>) => {
        impl<$t0: Component, $t1: Component> QueryFilter for ($f0<$t0>, $f1<$t1>) {
            type State<'a> = (
                <$f0<$t0> as QueryFilter>::State<'a>,
                <$f1<$t1> as QueryFilter>::State<'a>,
            );
            fn init_state<'a>(world: &'a World) -> Self::State<'a> {
                (
                    <$f0<$t0> as QueryFilter>::init_state(world),
                    <$f1<$t1> as QueryFilter>::init_state(world),
                )
            }
            fn matches<'a, 'b>(state: &'a Self::State<'b>, entity_id: usize) -> bool {
                <$f0<$t0> as QueryFilter>::matches(&state.0, entity_id)
                    && <$f1<$t1> as QueryFilter>::matches(&state.1, entity_id)
            }
        }
    };

    // 3 elements
    (
        $idx0:tt => $f0:ident < $t0:ident>,
        $idx1:tt => $f1:ident < $t1:ident>,
        $idx2:tt => $f2:ident < $t2:ident>
    ) => {
        impl<$t0: Component, $t1: Component, $t2: Component> QueryFilter
            for ($f0<$t0>, $f1<$t1>, $f2<$t2>)
        {
            type State<'a> = (
                <$f0<$t0> as QueryFilter>::State<'a>,
                <$f1<$t1> as QueryFilter>::State<'a>,
                <$f2<$t2> as QueryFilter>::State<'a>,
            );
            fn init_state<'a>(world: &'a World) -> Self::State<'a> {
                (
                    <$f0<$t0> as QueryFilter>::init_state(world),
                    <$f1<$t1> as QueryFilter>::init_state(world),
                    <$f2<$t2> as QueryFilter>::init_state(world),
                )
            }
            fn matches<'a, 'b>(state: &'a Self::State<'b>, entity_id: usize) -> bool {
                <$f0<$t0> as QueryFilter>::matches(&state.0, entity_id)
                    && <$f1<$t1> as QueryFilter>::matches(&state.1, entity_id)
                    && <$f2<$t2> as QueryFilter>::matches(&state.2, entity_id)
            }
        }
    };

    // 4 elements
    (
        $idx0:tt => $f0:ident < $t0:ident>,
        $idx1:tt => $f1:ident < $t1:ident>,
        $idx2:tt => $f2:ident < $t2:ident>,
        $idx3:tt => $f3:ident < $t3:ident>
    ) => {
        impl<
            $t0: Component,
            $t1: Component,
            $t2: Component,
            $t3: Component,
        > QueryFilter for ($f0<$t0>, $f1<$t1>, $f2<$t2>, $f3<$t3>) {
            type State<'a> = (
                <$f0<$t0> as QueryFilter>::State<'a>,
                <$f1<$t1> as QueryFilter>::State<'a>,
                <$f2<$t2> as QueryFilter>::State<'a>,
                <$f3<$t3> as QueryFilter>::State<'a>,
            );
            fn init_state<'a>(world: &'a World) -> Self::State<'a> {
                (
                    <$f0<$t0> as QueryFilter>::init_state(world),
                    <$f1<$t1> as QueryFilter>::init_state(world),
                    <$f2<$t2> as QueryFilter>::init_state(world),
                    <$f3<$t3> as QueryFilter>::init_state(world),
                )
            }
            fn matches<'a, 'b>(state: &'a Self::State<'b>, entity_id: usize) -> bool {
                <$f0<$t0> as QueryFilter>::matches(&state.0, entity_id)
                    && <$f1<$t1> as QueryFilter>::matches(&state.1, entity_id)
                    && <$f2<$t2> as QueryFilter>::matches(&state.2, entity_id)
                    && <$f3<$t3> as QueryFilter>::matches(&state.3, entity_id)
            }
        }
    };
}

impl_filter_tuple!(0 => With<A>);
impl_filter_tuple!(0 => Without<A>);
impl_filter_tuple!(0 => With<A>, 1 => With<B>);
impl_filter_tuple!(0 => With<A>, 1 => Without<B>);
impl_filter_tuple!(0 => Without<A>, 1 => With<B>);
impl_filter_tuple!(0 => Without<A>, 1 => Without<B>);
impl_filter_tuple!(0 => With<A>, 1 => With<B>, 2 => With<C>);
impl_filter_tuple!(0 => With<A>, 1 => Without<B>, 2 => With<C>);
impl_filter_tuple!(0 => Without<A>, 1 => Without<B>, 2 => Without<C>);
impl_filter_tuple!(0 => With<A>, 1 => With<B>, 2 => With<C>, 3 => With<D>);
impl_filter_tuple!(0 => With<A>, 1 => Without<B>, 2 => With<C>, 3 => Without<D>);
impl_filter_tuple!(0 => Without<A>, 1 => Without<B>, 2 => Without<C>, 3 => Without<D>);

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::World;

    use super::*;

    #[derive(Debug)]
    struct Pos(f32, f32);
    #[derive(Debug)]
    struct Vel(f32, f32);
    #[derive(Debug)]
    struct Frozen;
    #[derive(Debug)]
    struct Name(String);

    fn setup_world() -> World {
        let mut world = World::new();

        let e1 = world.spawn_entity();
        world.attach_component(&e1, Pos(0.0, 0.0)).unwrap();
        world.attach_component(&e1, Vel(1.0, 0.0)).unwrap();
        world.attach_component(&e1, Name("moving".into())).unwrap();

        let e2 = world.spawn_entity();
        world.attach_component(&e2, Pos(10.0, 10.0)).unwrap();
        world.attach_component(&e2, Vel(0.0, -1.0)).unwrap();

        let e3 = world.spawn_entity();
        world.attach_component(&e3, Pos(5.0, 5.0)).unwrap();
        world.attach_component(&e3, Frozen).unwrap();

        world
    }

    #[test]
    fn query_basic() {
        let world = setup_world();
        let mut query: Query<(Read<Pos>, Read<Vel>)> = Query::new(&world);
        let mut count = 0;
        for (_pos, _vel) in query.iter() {
            count += 1;
        }
        assert_eq!(count, 2, "Only e1 and e2 have both Pos and Vel");
    }

    #[test]
    fn query_write() {
        let world = setup_world();
        {
            let mut query: Query<(Write<Pos>, Read<Vel>)> = Query::new(&world);
            for (pos, vel) in query.iter() {
                pos.0 += vel.0;
                pos.1 += vel.1;
            }
        }

        let store = world.get_component_store::<Pos>().unwrap();
        let p1 = store.get_component(0).unwrap();
        assert_eq!(p1.0, 1.0);
        assert_eq!(p1.1, 0.0);
    }

    #[test]
    fn query_with_filter() {
        let world = setup_world();
        let mut query: Query<(Read<Pos>,), With<Name>> = Query::new(&world);
        let count = query.iter().count();
        assert_eq!(count, 1, "Only e1 has both Pos and Name");
    }

    #[test]
    fn query_without_filter() {
        let world = setup_world();
        let mut query: Query<(Read<Pos>,), Without<Frozen>> = Query::new(&world);
        let count = query.iter().count();
        assert_eq!(count, 2, "e1 and e2 have Pos without Frozen");
    }

    #[test]
    fn query_combined_filter() {
        let world = setup_world();
        let mut query: Query<(Read<Pos>, Read<Vel>), (With<Name>, Without<Frozen>)> =
            Query::new(&world);
        let count = query.iter().count();
        assert_eq!(count, 1, "Only e1 has Pos, Vel, Name, and not Frozen");
    }

    #[test]
    fn query_no_match() {
        let world = setup_world();
        let mut query: Query<(Read<Vel>,), Without<Vel>> = Query::new(&world);
        let count = query.iter().count();
        assert_eq!(count, 0, "No entity can simultaneously have and not have Vel");
    }

    #[test]
    fn query_single_read() {
        let world = setup_world();
        let mut query: Query<(Read<Pos>,)> = Query::new(&world);
        let count = query.iter().count();
        assert_eq!(count, 3, "All 3 entities have Pos");
    }

    #[test]
    fn query_single_write() {
        let world = setup_world();
        {
            let mut query: Query<(Write<Pos>,)> = Query::new(&world);
            for (pos,) in query.iter() {
                pos.0 *= 2.0;
            }
        }
        let store = world.get_component_store::<Pos>().unwrap();
        assert_eq!(store.get_component(0).unwrap().0, 0.0);
        assert_eq!(store.get_component(1).unwrap().0, 20.0);
        assert_eq!(store.get_component(2).unwrap().0, 10.0);
    }

    #[test]
    fn query_three_components() {
        let world = setup_world();
        let mut query: Query<(Read<Pos>, Read<Vel>, Read<Name>)> = Query::new(&world);
        let count = query.iter().count();
        assert_eq!(count, 1, "Only e1 has all three");
    }
}
