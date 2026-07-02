use crate::query::data::QueryData;
use crate::query::filter::QueryFilter;

pub struct QueryIter<'a, 'b: 'a, D: QueryData, F: QueryFilter> {
    pub(crate) state: &'a D::State<'b>,
    pub(crate) filter: &'a F::State<'b>,
    pub(crate) pivot: &'a [usize],
    pub(crate) cursor: usize,
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
