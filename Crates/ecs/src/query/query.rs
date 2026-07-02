use crate::query::data::QueryData;
use crate::query::filter::{QueryFilter, NoFilter};
use crate::query::iter::QueryIter;
use crate::World;

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
