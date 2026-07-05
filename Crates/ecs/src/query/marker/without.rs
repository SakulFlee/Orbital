use std::marker::PhantomData;

use crate::Component;
use crate::query::filter::QueryFilter;

pub struct Without<T>(PhantomData<T>);

impl<T: Component> QueryFilter for Without<T> {
    type State<'a> = Option<crate::ReadStoreHandle<'a, T>>;
    fn init_state<'a>(world: &'a crate::World) -> Self::State<'a> {
        world.get_component_store::<T>()
    }
    fn matches<'a, 'b>(state: &'a Self::State<'b>, entity_id: usize) -> bool {
        !state
            .as_ref()
            .and_then(|s| s.sparse.get(entity_id))
            .is_some_and(|x| x.is_some())
    }
}
