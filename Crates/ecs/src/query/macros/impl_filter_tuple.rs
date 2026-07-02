#[macro_export]
macro_rules! impl_filter_tuple {
    ($idx0:tt => $f0:ident < $t0:ident>) => {
        impl<$t0: $crate::Component> $crate::query::filter::QueryFilter
            for ($crate::query::marker::$f0<$t0>,)
        {
            type State<'a> = (
                <$crate::query::marker::$f0<$t0> as $crate::query::filter::QueryFilter>::State<'a>,
            );
            fn init_state<'a>(world: &'a $crate::World) -> Self::State<'a> {
                (<$crate::query::marker::$f0<$t0> as $crate::query::filter::QueryFilter>::init_state(world),)
            }
            fn matches<'a, 'b>(state: &'a Self::State<'b>, entity_id: usize) -> bool {
                <$crate::query::marker::$f0<$t0> as $crate::query::filter::QueryFilter>::matches(&state.0, entity_id)
            }
        }
    };

    ($idx0:tt => $f0:ident < $t0:ident>, $idx1:tt => $f1:ident < $t1:ident>) => {
        impl<
            $t0: $crate::Component,
            $t1: $crate::Component,
        > $crate::query::filter::QueryFilter
            for ($crate::query::marker::$f0<$t0>, $crate::query::marker::$f1<$t1>)
        {
            type State<'a> = (
                <$crate::query::marker::$f0<$t0> as $crate::query::filter::QueryFilter>::State<'a>,
                <$crate::query::marker::$f1<$t1> as $crate::query::filter::QueryFilter>::State<'a>,
            );
            fn init_state<'a>(world: &'a $crate::World) -> Self::State<'a> {
                (
                    <$crate::query::marker::$f0<$t0> as $crate::query::filter::QueryFilter>::init_state(world),
                    <$crate::query::marker::$f1<$t1> as $crate::query::filter::QueryFilter>::init_state(world),
                )
            }
            fn matches<'a, 'b>(state: &'a Self::State<'b>, entity_id: usize) -> bool {
                <$crate::query::marker::$f0<$t0> as $crate::query::filter::QueryFilter>::matches(&state.0, entity_id)
                    && <$crate::query::marker::$f1<$t1> as $crate::query::filter::QueryFilter>::matches(&state.1, entity_id)
            }
        }
    };

    (
        $idx0:tt => $f0:ident < $t0:ident>,
        $idx1:tt => $f1:ident < $t1:ident>,
        $idx2:tt => $f2:ident < $t2:ident>
    ) => {
        impl<
            $t0: $crate::Component,
            $t1: $crate::Component,
            $t2: $crate::Component,
        > $crate::query::filter::QueryFilter
            for (
                $crate::query::marker::$f0<$t0>,
                $crate::query::marker::$f1<$t1>,
                $crate::query::marker::$f2<$t2>,
            )
        {
            type State<'a> = (
                <$crate::query::marker::$f0<$t0> as $crate::query::filter::QueryFilter>::State<'a>,
                <$crate::query::marker::$f1<$t1> as $crate::query::filter::QueryFilter>::State<'a>,
                <$crate::query::marker::$f2<$t2> as $crate::query::filter::QueryFilter>::State<'a>,
            );
            fn init_state<'a>(world: &'a $crate::World) -> Self::State<'a> {
                (
                    <$crate::query::marker::$f0<$t0> as $crate::query::filter::QueryFilter>::init_state(world),
                    <$crate::query::marker::$f1<$t1> as $crate::query::filter::QueryFilter>::init_state(world),
                    <$crate::query::marker::$f2<$t2> as $crate::query::filter::QueryFilter>::init_state(world),
                )
            }
            fn matches<'a, 'b>(state: &'a Self::State<'b>, entity_id: usize) -> bool {
                <$crate::query::marker::$f0<$t0> as $crate::query::filter::QueryFilter>::matches(&state.0, entity_id)
                    && <$crate::query::marker::$f1<$t1> as $crate::query::filter::QueryFilter>::matches(&state.1, entity_id)
                    && <$crate::query::marker::$f2<$t2> as $crate::query::filter::QueryFilter>::matches(&state.2, entity_id)
            }
        }
    };

    (
        $idx0:tt => $f0:ident < $t0:ident>,
        $idx1:tt => $f1:ident < $t1:ident>,
        $idx2:tt => $f2:ident < $t2:ident>,
        $idx3:tt => $f3:ident < $t3:ident>
    ) => {
        impl<
            $t0: $crate::Component,
            $t1: $crate::Component,
            $t2: $crate::Component,
            $t3: $crate::Component,
        > $crate::query::filter::QueryFilter
            for (
                $crate::query::marker::$f0<$t0>,
                $crate::query::marker::$f1<$t1>,
                $crate::query::marker::$f2<$t2>,
                $crate::query::marker::$f3<$t3>,
            )
        {
            type State<'a> = (
                <$crate::query::marker::$f0<$t0> as $crate::query::filter::QueryFilter>::State<'a>,
                <$crate::query::marker::$f1<$t1> as $crate::query::filter::QueryFilter>::State<'a>,
                <$crate::query::marker::$f2<$t2> as $crate::query::filter::QueryFilter>::State<'a>,
                <$crate::query::marker::$f3<$t3> as $crate::query::filter::QueryFilter>::State<'a>,
            );
            fn init_state<'a>(world: &'a $crate::World) -> Self::State<'a> {
                (
                    <$crate::query::marker::$f0<$t0> as $crate::query::filter::QueryFilter>::init_state(world),
                    <$crate::query::marker::$f1<$t1> as $crate::query::filter::QueryFilter>::init_state(world),
                    <$crate::query::marker::$f2<$t2> as $crate::query::filter::QueryFilter>::init_state(world),
                    <$crate::query::marker::$f3<$t3> as $crate::query::filter::QueryFilter>::init_state(world),
                )
            }
            fn matches<'a, 'b>(state: &'a Self::State<'b>, entity_id: usize) -> bool {
                <$crate::query::marker::$f0<$t0> as $crate::query::filter::QueryFilter>::matches(&state.0, entity_id)
                    && <$crate::query::marker::$f1<$t1> as $crate::query::filter::QueryFilter>::matches(&state.1, entity_id)
                    && <$crate::query::marker::$f2<$t2> as $crate::query::filter::QueryFilter>::matches(&state.2, entity_id)
                    && <$crate::query::marker::$f3<$t3> as $crate::query::filter::QueryFilter>::matches(&state.3, entity_id)
            }
        }
    };
}

// ---------------------------------------------------------------------------
// Filter tuple invocations
// ---------------------------------------------------------------------------

impl_filter_tuple!(0 => With<A>);
impl_filter_tuple!(0 => Without<A>);
impl_filter_tuple!(0 => With<A>, 1 => With<B>);
impl_filter_tuple!(0 => With<A>, 1 => Without<B>);
impl_filter_tuple!(0 => Without<A>, 1 => With<B>);
impl_filter_tuple!(0 => Without<A>, 1 => Without<B>);
impl_filter_tuple!(0 => With<A>, 1 => With<B>, 2 => With<C>);
impl_filter_tuple!(0 => With<A>, 1 => Without<B>, 2 => With<C>);
impl_filter_tuple!(0 => With<A>, 1 => With<B>, 2 => Without<C>);
impl_filter_tuple!(0 => With<A>, 1 => Without<B>, 2 => Without<C>);
impl_filter_tuple!(0 => Without<A>, 1 => With<B>, 2 => With<C>);
impl_filter_tuple!(0 => Without<A>, 1 => With<B>, 2 => Without<C>);
impl_filter_tuple!(0 => Without<A>, 1 => Without<B>, 2 => With<C>);
impl_filter_tuple!(0 => Without<A>, 1 => Without<B>, 2 => Without<C>);
impl_filter_tuple!(0 => With<A>, 1 => With<B>, 2 => With<C>, 3 => With<D>);
impl_filter_tuple!(0 => With<A>, 1 => Without<B>, 2 => With<C>, 3 => Without<D>);
impl_filter_tuple!(0 => With<A>, 1 => Without<B>, 2 => Without<C>, 3 => Without<D>);
impl_filter_tuple!(0 => Without<A>, 1 => Without<B>, 2 => Without<C>, 3 => Without<D>);
