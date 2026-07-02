use crate::World;

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
