#[macro_export]
macro_rules! impl_query_data {
    ($idx0:tt => $acc0:ident < $t0:ident>) => {
        impl<$t0: $crate::Component> $crate::query::data::QueryData
            for ($crate::query::marker::$acc0<$t0>,)
        {
            type Item<'a> = ($crate::fetch_item_ty!($acc0, 'a, $t0),);
            type State<'a> = ($crate::fetch_state_ty!($acc0, 'a, $t0),);
            fn init_state<'a>(world: &'a $crate::World) -> Self::State<'a> {
                ($crate::fetch_store!($acc0, world, $t0),)
            }
            fn pivot_dense<'a, 'b>(state: &'a Self::State<'b>) -> &'a [usize] {
                match &state.0 {
                    Some(s) => &s.dense,
                    None => &[],
                }
            }
            fn get_item<'a, 'b>(
                state: &'a Self::State<'b>,
                entity_id: usize,
            ) -> Option<Self::Item<'a>> {
                let store0 = state.0.as_ref()?;
                let idx0 = store0.sparse.get(entity_id).copied().flatten()?;
                Some(($crate::fetch_item!($acc0, store0, idx0),))
            }
        }
    };

    ($idx0:tt => $acc0:ident < $t0:ident>, $idx1:tt => $acc1:ident < $t1:ident>) => {
        impl<
            $t0: $crate::Component,
            $t1: $crate::Component,
        > $crate::query::data::QueryData
            for ($crate::query::marker::$acc0<$t0>, $crate::query::marker::$acc1<$t1>)
        {
            type Item<'a> = (
                $crate::fetch_item_ty!($acc0, 'a, $t0),
                $crate::fetch_item_ty!($acc1, 'a, $t1),
            );
            type State<'a> = (
                $crate::fetch_state_ty!($acc0, 'a, $t0),
                $crate::fetch_state_ty!($acc1, 'a, $t1),
            );
            fn init_state<'a>(world: &'a $crate::World) -> Self::State<'a> {
                (
                    $crate::fetch_store!($acc0, world, $t0),
                    $crate::fetch_store!($acc1, world, $t1),
                )
            }
            fn pivot_dense<'a, 'b>(state: &'a Self::State<'b>) -> &'a [usize] {
                let s0 = match &state.0 {
                    Some(s) => s,
                    None => return &[],
                };
                let s1 = match &state.1 {
                    Some(s) => s,
                    None => return &[],
                };
                if s0.dense.len() <= s1.dense.len() {
                    &s0.dense
                } else {
                    &s1.dense
                }
            }
            fn get_item<'a, 'b>(
                state: &'a Self::State<'b>,
                entity_id: usize,
            ) -> Option<Self::Item<'a>> {
                let store0 = state.0.as_ref()?;
                let store1 = state.1.as_ref()?;
                let idx0 = store0.sparse.get(entity_id).copied().flatten()?;
                let idx1 = store1.sparse.get(entity_id).copied().flatten()?;
                Some((
                    $crate::fetch_item!($acc0, store0, idx0),
                    $crate::fetch_item!($acc1, store1, idx1),
                ))
            }
        }
    };

    (
        $idx0:tt => $acc0:ident < $t0:ident>,
        $idx1:tt => $acc1:ident < $t1:ident>,
        $idx2:tt => $acc2:ident < $t2:ident>
    ) => {
        impl<
            $t0: $crate::Component,
            $t1: $crate::Component,
            $t2: $crate::Component,
        > $crate::query::data::QueryData
            for (
                $crate::query::marker::$acc0<$t0>,
                $crate::query::marker::$acc1<$t1>,
                $crate::query::marker::$acc2<$t2>,
            )
        {
            type Item<'a> = (
                $crate::fetch_item_ty!($acc0, 'a, $t0),
                $crate::fetch_item_ty!($acc1, 'a, $t1),
                $crate::fetch_item_ty!($acc2, 'a, $t2),
            );
            type State<'a> = (
                $crate::fetch_state_ty!($acc0, 'a, $t0),
                $crate::fetch_state_ty!($acc1, 'a, $t1),
                $crate::fetch_state_ty!($acc2, 'a, $t2),
            );
            fn init_state<'a>(world: &'a $crate::World) -> Self::State<'a> {
                (
                    $crate::fetch_store!($acc0, world, $t0),
                    $crate::fetch_store!($acc1, world, $t1),
                    $crate::fetch_store!($acc2, world, $t2),
                )
            }
            fn pivot_dense<'a, 'b>(state: &'a Self::State<'b>) -> &'a [usize] {
                let s0 = match &state.0 {
                    Some(s) => s,
                    None => return &[],
                };
                let s1 = match &state.1 {
                    Some(s) => s,
                    None => return &[],
                };
                let s2 = match &state.2 {
                    Some(s) => s,
                    None => return &[],
                };
                let l0 = s0.dense.len();
                let l1 = s1.dense.len();
                let l2 = s2.dense.len();
                if l0 <= l1 && l0 <= l2 {
                    &s0.dense
                } else if l1 <= l0 && l1 <= l2 {
                    &s1.dense
                } else {
                    &s2.dense
                }
            }
            fn get_item<'a, 'b>(
                state: &'a Self::State<'b>,
                entity_id: usize,
            ) -> Option<Self::Item<'a>> {
                let store0 = state.0.as_ref()?;
                let store1 = state.1.as_ref()?;
                let store2 = state.2.as_ref()?;
                let idx0 = store0.sparse.get(entity_id).copied().flatten()?;
                let idx1 = store1.sparse.get(entity_id).copied().flatten()?;
                let idx2 = store2.sparse.get(entity_id).copied().flatten()?;
                Some((
                    $crate::fetch_item!($acc0, store0, idx0),
                    $crate::fetch_item!($acc1, store1, idx1),
                    $crate::fetch_item!($acc2, store2, idx2),
                ))
            }
        }
    };

    (
        $idx0:tt => $acc0:ident < $t0:ident>,
        $idx1:tt => $acc1:ident < $t1:ident>,
        $idx2:tt => $acc2:ident < $t2:ident>,
        $idx3:tt => $acc3:ident < $t3:ident>
    ) => {
        impl<
            $t0: $crate::Component,
            $t1: $crate::Component,
            $t2: $crate::Component,
            $t3: $crate::Component,
        > $crate::query::data::QueryData
            for (
                $crate::query::marker::$acc0<$t0>,
                $crate::query::marker::$acc1<$t1>,
                $crate::query::marker::$acc2<$t2>,
                $crate::query::marker::$acc3<$t3>,
            )
        {
            type Item<'a> = (
                $crate::fetch_item_ty!($acc0, 'a, $t0),
                $crate::fetch_item_ty!($acc1, 'a, $t1),
                $crate::fetch_item_ty!($acc2, 'a, $t2),
                $crate::fetch_item_ty!($acc3, 'a, $t3),
            );
            type State<'a> = (
                $crate::fetch_state_ty!($acc0, 'a, $t0),
                $crate::fetch_state_ty!($acc1, 'a, $t1),
                $crate::fetch_state_ty!($acc2, 'a, $t2),
                $crate::fetch_state_ty!($acc3, 'a, $t3),
            );
            fn init_state<'a>(world: &'a $crate::World) -> Self::State<'a> {
                (
                    $crate::fetch_store!($acc0, world, $t0),
                    $crate::fetch_store!($acc1, world, $t1),
                    $crate::fetch_store!($acc2, world, $t2),
                    $crate::fetch_store!($acc3, world, $t3),
                )
            }
            fn pivot_dense<'a, 'b>(state: &'a Self::State<'b>) -> &'a [usize] {
                let s0 = match &state.0 {
                    Some(s) => s,
                    None => return &[],
                };
                let s1 = match &state.1 {
                    Some(s) => s,
                    None => return &[],
                };
                let s2 = match &state.2 {
                    Some(s) => s,
                    None => return &[],
                };
                let s3 = match &state.3 {
                    Some(s) => s,
                    None => return &[],
                };
                let l0 = s0.dense.len();
                let l1 = s1.dense.len();
                let l2 = s2.dense.len();
                let l3 = s3.dense.len();
                if l0 <= l1 && l0 <= l2 && l0 <= l3 {
                    &s0.dense
                } else if l1 <= l0 && l1 <= l2 && l1 <= l3 {
                    &s1.dense
                } else if l2 <= l0 && l2 <= l1 && l2 <= l3 {
                    &s2.dense
                } else {
                    &s3.dense
                }
            }
            fn get_item<'a, 'b>(
                state: &'a Self::State<'b>,
                entity_id: usize,
            ) -> Option<Self::Item<'a>> {
                let store0 = state.0.as_ref()?;
                let store1 = state.1.as_ref()?;
                let store2 = state.2.as_ref()?;
                let store3 = state.3.as_ref()?;
                let idx0 = store0.sparse.get(entity_id).copied().flatten()?;
                let idx1 = store1.sparse.get(entity_id).copied().flatten()?;
                let idx2 = store2.sparse.get(entity_id).copied().flatten()?;
                let idx3 = store3.sparse.get(entity_id).copied().flatten()?;
                Some((
                    $crate::fetch_item!($acc0, store0, idx0),
                    $crate::fetch_item!($acc1, store1, idx1),
                    $crate::fetch_item!($acc2, store2, idx2),
                    $crate::fetch_item!($acc3, store3, idx3),
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

impl_query_data!(0 => Read<A>, 1 => Read<B>, 2 => Read<C>, 3 => Read<D>);
impl_query_data!(0 => Read<A>, 1 => Read<B>, 2 => Read<C>, 3 => Write<D>);
impl_query_data!(0 => Read<A>, 1 => Read<B>, 2 => Write<C>, 3 => Read<D>);
impl_query_data!(0 => Read<A>, 1 => Read<B>, 2 => Write<C>, 3 => Write<D>);
impl_query_data!(0 => Read<A>, 1 => Write<B>, 2 => Read<C>, 3 => Read<D>);
impl_query_data!(0 => Read<A>, 1 => Write<B>, 2 => Read<C>, 3 => Write<D>);
impl_query_data!(0 => Read<A>, 1 => Write<B>, 2 => Write<C>, 3 => Read<D>);
impl_query_data!(0 => Read<A>, 1 => Write<B>, 2 => Write<C>, 3 => Write<D>);
impl_query_data!(0 => Write<A>, 1 => Read<B>, 2 => Read<C>, 3 => Read<D>);
impl_query_data!(0 => Write<A>, 1 => Read<B>, 2 => Read<C>, 3 => Write<D>);
impl_query_data!(0 => Write<A>, 1 => Read<B>, 2 => Write<C>, 3 => Read<D>);
impl_query_data!(0 => Write<A>, 1 => Read<B>, 2 => Write<C>, 3 => Write<D>);
impl_query_data!(0 => Write<A>, 1 => Write<B>, 2 => Read<C>, 3 => Read<D>);
impl_query_data!(0 => Write<A>, 1 => Write<B>, 2 => Read<C>, 3 => Write<D>);
impl_query_data!(0 => Write<A>, 1 => Write<B>, 2 => Write<C>, 3 => Read<D>);
impl_query_data!(0 => Write<A>, 1 => Write<B>, 2 => Write<C>, 3 => Write<D>);
