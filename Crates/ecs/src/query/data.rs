use crate::World;

pub trait QueryData: Sized {
    type Item<'a>;
    type State<'a>;
    fn init_state<'a>(world: &'a World) -> Self::State<'a>;
    fn pivot_dense<'a, 'b>(state: &'a Self::State<'b>) -> &'a [usize];
    fn get_item<'a, 'b>(state: &'a Self::State<'b>, entity_id: usize) -> Option<Self::Item<'a>>;
}
