// ---------------------------------------------------------------------------
// Arity 1 — helper macros
// ---------------------------------------------------------------------------

macro_rules! impl_1_write {
    ($a:ident) => {
        impl<A: Clone + crate::Component, T: FnMut(&mut A) + Send + 'static>
            crate::system::system::IntoSystem<fn(&mut A)> for T
        {
            type System = Box<dyn crate::system::system::System>;
            fn into_system(self) -> Self::System {
                let mut f = self;
                Box::new(crate::system::system::FunctionSystem::new(
                    crate::system::system::FunctionSystemMetadata {
                        name: std::any::type_name::<T>(),
                        access: crate::system::access::ComponentAccess::new()
                            .writes::<A>(),
                    },
                    Box::new(move |world| {
                        let mut snap = {
                            let Some(store) = world.get_component_store::<A>() else { return; };
                            crate::system::merge::Snapshot::clone_from_store(&store)
                        };
                        for &eid in snap.dense.as_slice() {
                            if let Some(idx) = snap.sparse[eid] {
                                f(&mut snap.components[idx]);
                            }
                        }
                        snap.merge_into(world);
                    }),
                ))
            }
        }
    };
}

macro_rules! impl_1_read {
    ($a:ident) => {
        impl<A: crate::Component, T: FnMut(&A) + Send + 'static>
            crate::system::system::IntoSystem<fn(&A)> for T
        {
            type System = Box<dyn crate::system::system::System>;
            fn into_system(self) -> Self::System {
                let mut f = self;
                Box::new(crate::system::system::FunctionSystem::new(
                    crate::system::system::FunctionSystemMetadata {
                        name: std::any::type_name::<T>(),
                        access: crate::system::access::ComponentAccess::new()
                            .reads::<A>(),
                    },
                    Box::new(move |world| {
                        let Some(store) = world.get_component_store::<A>() else { return; };
                        for &eid in store.dense.as_slice() {
                            if let Some(idx) = store.sparse[eid] {
                                f(&store.components[idx]);
                            }
                        }
                    }),
                ))
            }
        }
    };
}

impl_1_write!(A);
impl_1_read!(A);

// ---------------------------------------------------------------------------
// Arity 2 — helper macros
// ---------------------------------------------------------------------------

macro_rules! impl_2_ww {
    ($a:ident, $b:ident) => {
        impl<A: Clone + crate::Component, B: Clone + crate::Component,
             T: FnMut(&mut A, &mut B) + Send + 'static>
            crate::system::system::IntoSystem<fn(&mut A, &mut B)> for T
        {
            type System = Box<dyn crate::system::system::System>;
            fn into_system(self) -> Self::System {
                let mut f = self;
                Box::new(crate::system::system::FunctionSystem::new(
                    crate::system::system::FunctionSystemMetadata {
                        name: std::any::type_name::<T>(),
                        access: crate::system::access::ComponentAccess::new()
                            .writes::<A>().writes::<B>(),
                    },
                    Box::new(move |world| {
                        let (mut snap_a, mut snap_b) = {
                            let Some(sa) = world.get_component_store::<A>() else { return; };
                            let Some(sb) = world.get_component_store::<B>() else { return; };
                            (
                                crate::system::merge::Snapshot::clone_from_store(&sa),
                                crate::system::merge::Snapshot::clone_from_store(&sb),
                            )
                        };
                        let pivot = if snap_a.dense.len() <= snap_b.dense.len()
                            { snap_a.dense.as_slice() } else { snap_b.dense.as_slice() };
                        for &eid in pivot {
                            if let (Some(ia), Some(ib)) = (snap_a.sparse[eid], snap_b.sparse[eid]) {
                                f(&mut snap_a.components[ia], &mut snap_b.components[ib]);
                            }
                        }
                        snap_a.merge_into(world);
                        snap_b.merge_into(world);
                    }),
                ))
            }
        }
    };
}

macro_rules! impl_2_wr {
    ($a:ident, $b:ident) => {
        impl<A: Clone + crate::Component, B: crate::Component,
             T: FnMut(&mut A, &B) + Send + 'static>
            crate::system::system::IntoSystem<fn(&mut A, &B)> for T
        {
            type System = Box<dyn crate::system::system::System>;
            fn into_system(self) -> Self::System {
                let mut f = self;
                Box::new(crate::system::system::FunctionSystem::new(
                    crate::system::system::FunctionSystemMetadata {
                        name: std::any::type_name::<T>(),
                        access: crate::system::access::ComponentAccess::new()
                            .writes::<A>().reads::<B>(),
                    },
                    Box::new(move |world| {
                        let Some(sb) = world.get_component_store::<B>() else { return; };
                        let mut snap_a = {
                            let Some(sa) = world.get_component_store::<A>() else { return; };
                            crate::system::merge::Snapshot::clone_from_store(&sa)
                        };
                        let pivot = if snap_a.dense.len() <= sb.dense.len()
                            { snap_a.dense.as_slice() } else { sb.dense.as_slice() };
                        for &eid in pivot {
                            if let (Some(ia), Some(ib)) = (snap_a.sparse[eid], sb.sparse[eid]) {
                                f(&mut snap_a.components[ia], &sb.components[ib]);
                            }
                        }
                        snap_a.merge_into(world);
                    }),
                ))
            }
        }
    };
}

macro_rules! impl_2_rw {
    ($a:ident, $b:ident) => {
        impl<A: crate::Component, B: Clone + crate::Component,
             T: FnMut(&A, &mut B) + Send + 'static>
            crate::system::system::IntoSystem<fn(&A, &mut B)> for T
        {
            type System = Box<dyn crate::system::system::System>;
            fn into_system(self) -> Self::System {
                let mut f = self;
                Box::new(crate::system::system::FunctionSystem::new(
                    crate::system::system::FunctionSystemMetadata {
                        name: std::any::type_name::<T>(),
                        access: crate::system::access::ComponentAccess::new()
                            .reads::<A>().writes::<B>(),
                    },
                    Box::new(move |world| {
                        let Some(sa) = world.get_component_store::<A>() else { return; };
                        let mut snap_b = {
                            let Some(sb) = world.get_component_store::<B>() else { return; };
                            crate::system::merge::Snapshot::clone_from_store(&sb)
                        };
                        let pivot = if sa.dense.len() <= snap_b.dense.len()
                            { sa.dense.as_slice() } else { snap_b.dense.as_slice() };
                        for &eid in pivot {
                            if let (Some(ia), Some(ib)) = (sa.sparse[eid], snap_b.sparse[eid]) {
                                f(&sa.components[ia], &mut snap_b.components[ib]);
                            }
                        }
                        snap_b.merge_into(world);
                    }),
                ))
            }
        }
    };
}

macro_rules! impl_2_rr {
    ($a:ident, $b:ident) => {
        impl<A: crate::Component, B: crate::Component,
             T: FnMut(&A, &B) + Send + 'static>
            crate::system::system::IntoSystem<fn(&A, &B)> for T
        {
            type System = Box<dyn crate::system::system::System>;
            fn into_system(self) -> Self::System {
                let mut f = self;
                Box::new(crate::system::system::FunctionSystem::new(
                    crate::system::system::FunctionSystemMetadata {
                        name: std::any::type_name::<T>(),
                        access: crate::system::access::ComponentAccess::new()
                            .reads::<A>().reads::<B>(),
                    },
                    Box::new(move |world| {
                        let Some(sa) = world.get_component_store::<A>() else { return; };
                        let Some(sb) = world.get_component_store::<B>() else { return; };
                        let pivot = if sa.dense.len() <= sb.dense.len()
                            { sa.dense.as_slice() } else { sb.dense.as_slice() };
                        for &eid in pivot {
                            if let (Some(ia), Some(ib)) = (sa.sparse[eid], sb.sparse[eid]) {
                                f(&sa.components[ia], &sb.components[ib]);
                            }
                        }
                    }),
                ))
            }
        }
    };
}

impl_2_ww!(A, B);
impl_2_wr!(A, B);
impl_2_rw!(A, B);
impl_2_rr!(A, B);

// ---------------------------------------------------------------------------
// Arity 3 — all 8 patterns inline
// ---------------------------------------------------------------------------

impl<A: Clone + crate::Component, B: Clone + crate::Component, C: Clone + crate::Component,
     T: FnMut(&mut A, &mut B, &mut C) + Send + 'static>
    crate::system::system::IntoSystem<fn(&mut A, &mut B, &mut C)> for T
{
    type System = Box<dyn crate::system::system::System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(crate::system::system::FunctionSystem::new(
            crate::system::system::FunctionSystemMetadata {
                name: std::any::type_name::<T>(),
                access: crate::system::access::ComponentAccess::new()
                    .writes::<A>().writes::<B>().writes::<C>(),
            },
            Box::new(move |world| {
                let (mut snap_a, mut snap_b, mut snap_c) = {
                    let Some(sa) = world.get_component_store::<A>() else { return; };
                    let Some(sb) = world.get_component_store::<B>() else { return; };
                    let Some(sc) = world.get_component_store::<C>() else { return; };
                    (
                        crate::system::merge::Snapshot::clone_from_store(&sa),
                        crate::system::merge::Snapshot::clone_from_store(&sb),
                        crate::system::merge::Snapshot::clone_from_store(&sc),
                    )
                };
                let pivot = *[snap_a.dense.as_slice(), snap_b.dense.as_slice(), snap_c.dense.as_slice()]
                    .iter().min_by_key(|s| s.len()).unwrap();
                for &eid in pivot {
                    if let (Some(ia), Some(ib), Some(ic)) = (snap_a.sparse[eid], snap_b.sparse[eid], snap_c.sparse[eid]) {
                        f(&mut snap_a.components[ia], &mut snap_b.components[ib], &mut snap_c.components[ic]);
                    }
                }
                snap_a.merge_into(world);
                snap_b.merge_into(world);
                snap_c.merge_into(world);
            }),
        ))
    }
}

impl<A: Clone + crate::Component, B: Clone + crate::Component, C: crate::Component,
     T: FnMut(&mut A, &mut B, &C) + Send + 'static>
    crate::system::system::IntoSystem<fn(&mut A, &mut B, &C)> for T
{
    type System = Box<dyn crate::system::system::System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(crate::system::system::FunctionSystem::new(
            crate::system::system::FunctionSystemMetadata {
                name: std::any::type_name::<T>(),
                access: crate::system::access::ComponentAccess::new()
                    .writes::<A>().writes::<B>().reads::<C>(),
            },
            Box::new(move |world| {
                let Some(sc) = world.get_component_store::<C>() else { return; };
                let (mut snap_a, mut snap_b) = {
                    let Some(sa) = world.get_component_store::<A>() else { return; };
                    let Some(sb) = world.get_component_store::<B>() else { return; };
                    (
                        crate::system::merge::Snapshot::clone_from_store(&sa),
                        crate::system::merge::Snapshot::clone_from_store(&sb),
                    )
                };
                let pivot = *[snap_a.dense.as_slice(), snap_b.dense.as_slice(), sc.dense.as_slice()]
                    .iter().min_by_key(|s| s.len()).unwrap();
                for &eid in pivot {
                    if let (Some(ia), Some(ib), Some(ic)) = (snap_a.sparse[eid], snap_b.sparse[eid], sc.sparse[eid]) {
                        f(&mut snap_a.components[ia], &mut snap_b.components[ib], &sc.components[ic]);
                    }
                }
                snap_a.merge_into(world);
                snap_b.merge_into(world);
            }),
        ))
    }
}

impl<A: Clone + crate::Component, B: crate::Component, C: Clone + crate::Component,
     T: FnMut(&mut A, &B, &mut C) + Send + 'static>
    crate::system::system::IntoSystem<fn(&mut A, &B, &mut C)> for T
{
    type System = Box<dyn crate::system::system::System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(crate::system::system::FunctionSystem::new(
            crate::system::system::FunctionSystemMetadata {
                name: std::any::type_name::<T>(),
                access: crate::system::access::ComponentAccess::new()
                    .writes::<A>().reads::<B>().writes::<C>(),
            },
            Box::new(move |world| {
                let Some(sb) = world.get_component_store::<B>() else { return; };
                let (mut snap_a, mut snap_c) = {
                    let Some(sa) = world.get_component_store::<A>() else { return; };
                    let Some(sc) = world.get_component_store::<C>() else { return; };
                    (
                        crate::system::merge::Snapshot::clone_from_store(&sa),
                        crate::system::merge::Snapshot::clone_from_store(&sc),
                    )
                };
                let pivot = *[snap_a.dense.as_slice(), sb.dense.as_slice(), snap_c.dense.as_slice()]
                    .iter().min_by_key(|s| s.len()).unwrap();
                for &eid in pivot {
                    if let (Some(ia), Some(ib), Some(ic)) = (snap_a.sparse[eid], sb.sparse[eid], snap_c.sparse[eid]) {
                        f(&mut snap_a.components[ia], &sb.components[ib], &mut snap_c.components[ic]);
                    }
                }
                snap_a.merge_into(world);
                snap_c.merge_into(world);
            }),
        ))
    }
}

impl<A: Clone + crate::Component, B: crate::Component, C: crate::Component,
     T: FnMut(&mut A, &B, &C) + Send + 'static>
    crate::system::system::IntoSystem<fn(&mut A, &B, &C)> for T
{
    type System = Box<dyn crate::system::system::System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(crate::system::system::FunctionSystem::new(
            crate::system::system::FunctionSystemMetadata {
                name: std::any::type_name::<T>(),
                access: crate::system::access::ComponentAccess::new()
                    .writes::<A>().reads::<B>().reads::<C>(),
            },
            Box::new(move |world| {
                let Some(sb) = world.get_component_store::<B>() else { return; };
                let Some(sc) = world.get_component_store::<C>() else { return; };
                let mut snap_a = {
                    let Some(sa) = world.get_component_store::<A>() else { return; };
                    crate::system::merge::Snapshot::clone_from_store(&sa)
                };
                let pivot = *[snap_a.dense.as_slice(), sb.dense.as_slice(), sc.dense.as_slice()]
                    .iter().min_by_key(|s| s.len()).unwrap();
                for &eid in pivot {
                    if let (Some(ia), Some(ib), Some(ic)) = (snap_a.sparse[eid], sb.sparse[eid], sc.sparse[eid]) {
                        f(&mut snap_a.components[ia], &sb.components[ib], &sc.components[ic]);
                    }
                }
                snap_a.merge_into(world);
            }),
        ))
    }
}

impl<A: crate::Component, B: Clone + crate::Component, C: Clone + crate::Component,
     T: FnMut(&A, &mut B, &mut C) + Send + 'static>
    crate::system::system::IntoSystem<fn(&A, &mut B, &mut C)> for T
{
    type System = Box<dyn crate::system::system::System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(crate::system::system::FunctionSystem::new(
            crate::system::system::FunctionSystemMetadata {
                name: std::any::type_name::<T>(),
                access: crate::system::access::ComponentAccess::new()
                    .reads::<A>().writes::<B>().writes::<C>(),
            },
            Box::new(move |world| {
                let Some(sa) = world.get_component_store::<A>() else { return; };
                let (mut snap_b, mut snap_c) = {
                    let Some(sb) = world.get_component_store::<B>() else { return; };
                    let Some(sc) = world.get_component_store::<C>() else { return; };
                    (
                        crate::system::merge::Snapshot::clone_from_store(&sb),
                        crate::system::merge::Snapshot::clone_from_store(&sc),
                    )
                };
                let pivot = *[sa.dense.as_slice(), snap_b.dense.as_slice(), snap_c.dense.as_slice()]
                    .iter().min_by_key(|s| s.len()).unwrap();
                for &eid in pivot {
                    if let (Some(ia), Some(ib), Some(ic)) = (sa.sparse[eid], snap_b.sparse[eid], snap_c.sparse[eid]) {
                        f(&sa.components[ia], &mut snap_b.components[ib], &mut snap_c.components[ic]);
                    }
                }
                snap_b.merge_into(world);
                snap_c.merge_into(world);
            }),
        ))
    }
}

impl<A: crate::Component, B: Clone + crate::Component, C: crate::Component,
     T: FnMut(&A, &mut B, &C) + Send + 'static>
    crate::system::system::IntoSystem<fn(&A, &mut B, &C)> for T
{
    type System = Box<dyn crate::system::system::System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(crate::system::system::FunctionSystem::new(
            crate::system::system::FunctionSystemMetadata {
                name: std::any::type_name::<T>(),
                access: crate::system::access::ComponentAccess::new()
                    .reads::<A>().writes::<B>().reads::<C>(),
            },
            Box::new(move |world| {
                let Some(sa) = world.get_component_store::<A>() else { return; };
                let Some(sc) = world.get_component_store::<C>() else { return; };
                let mut snap_b = {
                    let Some(sb) = world.get_component_store::<B>() else { return; };
                    crate::system::merge::Snapshot::clone_from_store(&sb)
                };
                let pivot = *[sa.dense.as_slice(), snap_b.dense.as_slice(), sc.dense.as_slice()]
                    .iter().min_by_key(|s| s.len()).unwrap();
                for &eid in pivot {
                    if let (Some(ia), Some(ib), Some(ic)) = (sa.sparse[eid], snap_b.sparse[eid], sc.sparse[eid]) {
                        f(&sa.components[ia], &mut snap_b.components[ib], &sc.components[ic]);
                    }
                }
                snap_b.merge_into(world);
            }),
        ))
    }
}

impl<A: crate::Component, B: crate::Component, C: Clone + crate::Component,
     T: FnMut(&A, &B, &mut C) + Send + 'static>
    crate::system::system::IntoSystem<fn(&A, &B, &mut C)> for T
{
    type System = Box<dyn crate::system::system::System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(crate::system::system::FunctionSystem::new(
            crate::system::system::FunctionSystemMetadata {
                name: std::any::type_name::<T>(),
                access: crate::system::access::ComponentAccess::new()
                    .reads::<A>().reads::<B>().writes::<C>(),
            },
            Box::new(move |world| {
                let Some(sa) = world.get_component_store::<A>() else { return; };
                let Some(sb) = world.get_component_store::<B>() else { return; };
                let mut snap_c = {
                    let Some(sc) = world.get_component_store::<C>() else { return; };
                    crate::system::merge::Snapshot::clone_from_store(&sc)
                };
                let pivot = *[sa.dense.as_slice(), sb.dense.as_slice(), snap_c.dense.as_slice()]
                    .iter().min_by_key(|s| s.len()).unwrap();
                for &eid in pivot {
                    if let (Some(ia), Some(ib), Some(ic)) = (sa.sparse[eid], sb.sparse[eid], snap_c.sparse[eid]) {
                        f(&sa.components[ia], &sb.components[ib], &mut snap_c.components[ic]);
                    }
                }
                snap_c.merge_into(world);
            }),
        ))
    }
}

impl<A: crate::Component, B: crate::Component, C: crate::Component,
     T: FnMut(&A, &B, &C) + Send + 'static>
    crate::system::system::IntoSystem<fn(&A, &B, &C)> for T
{
    type System = Box<dyn crate::system::system::System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(crate::system::system::FunctionSystem::new(
            crate::system::system::FunctionSystemMetadata {
                name: std::any::type_name::<T>(),
                access: crate::system::access::ComponentAccess::new()
                    .reads::<A>().reads::<B>().reads::<C>(),
            },
            Box::new(move |world| {
                let Some(sa) = world.get_component_store::<A>() else { return; };
                let Some(sb) = world.get_component_store::<B>() else { return; };
                let Some(sc) = world.get_component_store::<C>() else { return; };
                let pivot = *[sa.dense.as_slice(), sb.dense.as_slice(), sc.dense.as_slice()]
                    .iter().min_by_key(|s| s.len()).unwrap();
                for &eid in pivot {
                    if let (Some(ia), Some(ib), Some(ic)) = (sa.sparse[eid], sb.sparse[eid], sc.sparse[eid]) {
                        f(&sa.components[ia], &sb.components[ib], &sc.components[ic]);
                    }
                }
            }),
        ))
    }
}

// ---------------------------------------------------------------------------
// Arity 4 — all 16 patterns inline
// For each write param: snapshot from store.
// For each read param: use the store directly.
// Pivot = smallest dense array among all stores/snapshots.
// After iteration: merge each snapshot back.
// ---------------------------------------------------------------------------

// wwww
impl<A: Clone + crate::Component, B: Clone + crate::Component, C: Clone + crate::Component, D: Clone + crate::Component,
     T: FnMut(&mut A, &mut B, &mut C, &mut D) + Send + 'static>
    crate::system::system::IntoSystem<fn(&mut A, &mut B, &mut C, &mut D)> for T
{
    type System = Box<dyn crate::system::system::System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(crate::system::system::FunctionSystem::new(
            crate::system::system::FunctionSystemMetadata {
                name: std::any::type_name::<T>(),
                access: crate::system::access::ComponentAccess::new()
                    .writes::<A>().writes::<B>().writes::<C>().writes::<D>(),
            },
            Box::new(move |world| {
                let (mut snap_a, mut snap_b, mut snap_c, mut snap_d) = {
                    let Some(sa) = world.get_component_store::<A>() else { return; };
                    let Some(sb) = world.get_component_store::<B>() else { return; };
                    let Some(sc) = world.get_component_store::<C>() else { return; };
                    let Some(sd) = world.get_component_store::<D>() else { return; };
                    (
                        crate::system::merge::Snapshot::clone_from_store(&sa),
                        crate::system::merge::Snapshot::clone_from_store(&sb),
                        crate::system::merge::Snapshot::clone_from_store(&sc),
                        crate::system::merge::Snapshot::clone_from_store(&sd),
                    )
                };
                let pivot = *[snap_a.dense.as_slice(), snap_b.dense.as_slice(), snap_c.dense.as_slice(), snap_d.dense.as_slice()]
                    .iter().min_by_key(|s| s.len()).unwrap();
                for &eid in pivot {
                    if let (Some(ia), Some(ib), Some(ic), Some(id)) = (snap_a.sparse[eid], snap_b.sparse[eid], snap_c.sparse[eid], snap_d.sparse[eid]) {
                        f(&mut snap_a.components[ia], &mut snap_b.components[ib], &mut snap_c.components[ic], &mut snap_d.components[id]);
                    }
                }
                snap_a.merge_into(world); snap_b.merge_into(world);
                snap_c.merge_into(world); snap_d.merge_into(world);
            }),
        ))
    }
}

// wwwr
impl<A: Clone + crate::Component, B: Clone + crate::Component, C: Clone + crate::Component, D: crate::Component,
     T: FnMut(&mut A, &mut B, &mut C, &D) + Send + 'static>
    crate::system::system::IntoSystem<fn(&mut A, &mut B, &mut C, &D)> for T
{
    type System = Box<dyn crate::system::system::System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(crate::system::system::FunctionSystem::new(
            crate::system::system::FunctionSystemMetadata {
                name: std::any::type_name::<T>(),
                access: crate::system::access::ComponentAccess::new()
                    .writes::<A>().writes::<B>().writes::<C>().reads::<D>(),
            },
            Box::new(move |world| {
                let Some(sd) = world.get_component_store::<D>() else { return; };
                let (mut snap_a, mut snap_b, mut snap_c) = {
                    let Some(sa) = world.get_component_store::<A>() else { return; };
                    let Some(sb) = world.get_component_store::<B>() else { return; };
                    let Some(sc) = world.get_component_store::<C>() else { return; };
                    (
                        crate::system::merge::Snapshot::clone_from_store(&sa),
                        crate::system::merge::Snapshot::clone_from_store(&sb),
                        crate::system::merge::Snapshot::clone_from_store(&sc),
                    )
                };
                let pivot = *[snap_a.dense.as_slice(), snap_b.dense.as_slice(), snap_c.dense.as_slice(), sd.dense.as_slice()]
                    .iter().min_by_key(|s| s.len()).unwrap();
                for &eid in pivot {
                    if let (Some(ia), Some(ib), Some(ic), Some(id)) = (snap_a.sparse[eid], snap_b.sparse[eid], snap_c.sparse[eid], sd.sparse[eid]) {
                        f(&mut snap_a.components[ia], &mut snap_b.components[ib], &mut snap_c.components[ic], &sd.components[id]);
                    }
                }
                snap_a.merge_into(world); snap_b.merge_into(world); snap_c.merge_into(world);
            }),
        ))
    }
}

// wwrw
impl<A: Clone + crate::Component, B: Clone + crate::Component, C: crate::Component, D: Clone + crate::Component,
     T: FnMut(&mut A, &mut B, &C, &mut D) + Send + 'static>
    crate::system::system::IntoSystem<fn(&mut A, &mut B, &C, &mut D)> for T
{
    type System = Box<dyn crate::system::system::System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(crate::system::system::FunctionSystem::new(
            crate::system::system::FunctionSystemMetadata {
                name: std::any::type_name::<T>(),
                access: crate::system::access::ComponentAccess::new()
                    .writes::<A>().writes::<B>().reads::<C>().writes::<D>(),
            },
            Box::new(move |world| {
                let Some(sc) = world.get_component_store::<C>() else { return; };
                let (mut snap_a, mut snap_b, mut snap_d) = {
                    let Some(sa) = world.get_component_store::<A>() else { return; };
                    let Some(sb) = world.get_component_store::<B>() else { return; };
                    let Some(sd) = world.get_component_store::<D>() else { return; };
                    (
                        crate::system::merge::Snapshot::clone_from_store(&sa),
                        crate::system::merge::Snapshot::clone_from_store(&sb),
                        crate::system::merge::Snapshot::clone_from_store(&sd),
                    )
                };
                let pivot = *[snap_a.dense.as_slice(), snap_b.dense.as_slice(), sc.dense.as_slice(), snap_d.dense.as_slice()]
                    .iter().min_by_key(|s| s.len()).unwrap();
                for &eid in pivot {
                    if let (Some(ia), Some(ib), Some(ic), Some(id)) = (snap_a.sparse[eid], snap_b.sparse[eid], sc.sparse[eid], snap_d.sparse[eid]) {
                        f(&mut snap_a.components[ia], &mut snap_b.components[ib], &sc.components[ic], &mut snap_d.components[id]);
                    }
                }
                snap_a.merge_into(world); snap_b.merge_into(world); snap_d.merge_into(world);
            }),
        ))
    }
}

// wwrr
impl<A: Clone + crate::Component, B: Clone + crate::Component, C: crate::Component, D: crate::Component,
     T: FnMut(&mut A, &mut B, &C, &D) + Send + 'static>
    crate::system::system::IntoSystem<fn(&mut A, &mut B, &C, &D)> for T
{
    type System = Box<dyn crate::system::system::System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(crate::system::system::FunctionSystem::new(
            crate::system::system::FunctionSystemMetadata {
                name: std::any::type_name::<T>(),
                access: crate::system::access::ComponentAccess::new()
                    .writes::<A>().writes::<B>().reads::<C>().reads::<D>(),
            },
            Box::new(move |world| {
                let Some(sc) = world.get_component_store::<C>() else { return; };
                let Some(sd) = world.get_component_store::<D>() else { return; };
                let (mut snap_a, mut snap_b) = {
                    let Some(sa) = world.get_component_store::<A>() else { return; };
                    let Some(sb) = world.get_component_store::<B>() else { return; };
                    (
                        crate::system::merge::Snapshot::clone_from_store(&sa),
                        crate::system::merge::Snapshot::clone_from_store(&sb),
                    )
                };
                let pivot = *[snap_a.dense.as_slice(), snap_b.dense.as_slice(), sc.dense.as_slice(), sd.dense.as_slice()]
                    .iter().min_by_key(|s| s.len()).unwrap();
                for &eid in pivot {
                    if let (Some(ia), Some(ib), Some(ic), Some(id)) = (snap_a.sparse[eid], snap_b.sparse[eid], sc.sparse[eid], sd.sparse[eid]) {
                        f(&mut snap_a.components[ia], &mut snap_b.components[ib], &sc.components[ic], &sd.components[id]);
                    }
                }
                snap_a.merge_into(world); snap_b.merge_into(world);
            }),
        ))
    }
}

// wrww
impl<A: Clone + crate::Component, B: crate::Component, C: Clone + crate::Component, D: Clone + crate::Component,
     T: FnMut(&mut A, &B, &mut C, &mut D) + Send + 'static>
    crate::system::system::IntoSystem<fn(&mut A, &B, &mut C, &mut D)> for T
{
    type System = Box<dyn crate::system::system::System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(crate::system::system::FunctionSystem::new(
            crate::system::system::FunctionSystemMetadata {
                name: std::any::type_name::<T>(),
                access: crate::system::access::ComponentAccess::new()
                    .writes::<A>().reads::<B>().writes::<C>().writes::<D>(),
            },
            Box::new(move |world| {
                let Some(sb) = world.get_component_store::<B>() else { return; };
                let (mut snap_a, mut snap_c, mut snap_d) = {
                    let Some(sa) = world.get_component_store::<A>() else { return; };
                    let Some(sc) = world.get_component_store::<C>() else { return; };
                    let Some(sd) = world.get_component_store::<D>() else { return; };
                    (
                        crate::system::merge::Snapshot::clone_from_store(&sa),
                        crate::system::merge::Snapshot::clone_from_store(&sc),
                        crate::system::merge::Snapshot::clone_from_store(&sd),
                    )
                };
                let pivot = *[snap_a.dense.as_slice(), sb.dense.as_slice(), snap_c.dense.as_slice(), snap_d.dense.as_slice()]
                    .iter().min_by_key(|s| s.len()).unwrap();
                for &eid in pivot {
                    if let (Some(ia), Some(ib), Some(ic), Some(id)) = (snap_a.sparse[eid], sb.sparse[eid], snap_c.sparse[eid], snap_d.sparse[eid]) {
                        f(&mut snap_a.components[ia], &sb.components[ib], &mut snap_c.components[ic], &mut snap_d.components[id]);
                    }
                }
                snap_a.merge_into(world); snap_c.merge_into(world); snap_d.merge_into(world);
            }),
        ))
    }
}

// wrwr
impl<A: Clone + crate::Component, B: crate::Component, C: Clone + crate::Component, D: crate::Component,
     T: FnMut(&mut A, &B, &mut C, &D) + Send + 'static>
    crate::system::system::IntoSystem<fn(&mut A, &B, &mut C, &D)> for T
{
    type System = Box<dyn crate::system::system::System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(crate::system::system::FunctionSystem::new(
            crate::system::system::FunctionSystemMetadata {
                name: std::any::type_name::<T>(),
                access: crate::system::access::ComponentAccess::new()
                    .writes::<A>().reads::<B>().writes::<C>().reads::<D>(),
            },
            Box::new(move |world| {
                let Some(sb) = world.get_component_store::<B>() else { return; };
                let Some(sd) = world.get_component_store::<D>() else { return; };
                let (mut snap_a, mut snap_c) = {
                    let Some(sa) = world.get_component_store::<A>() else { return; };
                    let Some(sc) = world.get_component_store::<C>() else { return; };
                    (
                        crate::system::merge::Snapshot::clone_from_store(&sa),
                        crate::system::merge::Snapshot::clone_from_store(&sc),
                    )
                };
                let pivot = *[snap_a.dense.as_slice(), sb.dense.as_slice(), snap_c.dense.as_slice(), sd.dense.as_slice()]
                    .iter().min_by_key(|s| s.len()).unwrap();
                for &eid in pivot {
                    if let (Some(ia), Some(ib), Some(ic), Some(id)) = (snap_a.sparse[eid], sb.sparse[eid], snap_c.sparse[eid], sd.sparse[eid]) {
                        f(&mut snap_a.components[ia], &sb.components[ib], &mut snap_c.components[ic], &sd.components[id]);
                    }
                }
                snap_a.merge_into(world); snap_c.merge_into(world);
            }),
        ))
    }
}

// wrrw
impl<A: Clone + crate::Component, B: crate::Component, C: crate::Component, D: Clone + crate::Component,
     T: FnMut(&mut A, &B, &C, &mut D) + Send + 'static>
    crate::system::system::IntoSystem<fn(&mut A, &B, &C, &mut D)> for T
{
    type System = Box<dyn crate::system::system::System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(crate::system::system::FunctionSystem::new(
            crate::system::system::FunctionSystemMetadata {
                name: std::any::type_name::<T>(),
                access: crate::system::access::ComponentAccess::new()
                    .writes::<A>().reads::<B>().reads::<C>().writes::<D>(),
            },
            Box::new(move |world| {
                let Some(sb) = world.get_component_store::<B>() else { return; };
                let Some(sc) = world.get_component_store::<C>() else { return; };
                let (mut snap_a, mut snap_d) = {
                    let Some(sa) = world.get_component_store::<A>() else { return; };
                    let Some(sd) = world.get_component_store::<D>() else { return; };
                    (
                        crate::system::merge::Snapshot::clone_from_store(&sa),
                        crate::system::merge::Snapshot::clone_from_store(&sd),
                    )
                };
                let pivot = *[snap_a.dense.as_slice(), sb.dense.as_slice(), sc.dense.as_slice(), snap_d.dense.as_slice()]
                    .iter().min_by_key(|s| s.len()).unwrap();
                for &eid in pivot {
                    if let (Some(ia), Some(ib), Some(ic), Some(id)) = (snap_a.sparse[eid], sb.sparse[eid], sc.sparse[eid], snap_d.sparse[eid]) {
                        f(&mut snap_a.components[ia], &sb.components[ib], &sc.components[ic], &mut snap_d.components[id]);
                    }
                }
                snap_a.merge_into(world); snap_d.merge_into(world);
            }),
        ))
    }
}

// wrrr
impl<A: Clone + crate::Component, B: crate::Component, C: crate::Component, D: crate::Component,
     T: FnMut(&mut A, &B, &C, &D) + Send + 'static>
    crate::system::system::IntoSystem<fn(&mut A, &B, &C, &D)> for T
{
    type System = Box<dyn crate::system::system::System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(crate::system::system::FunctionSystem::new(
            crate::system::system::FunctionSystemMetadata {
                name: std::any::type_name::<T>(),
                access: crate::system::access::ComponentAccess::new()
                    .writes::<A>().reads::<B>().reads::<C>().reads::<D>(),
            },
            Box::new(move |world| {
                let Some(sb) = world.get_component_store::<B>() else { return; };
                let Some(sc) = world.get_component_store::<C>() else { return; };
                let Some(sd) = world.get_component_store::<D>() else { return; };
                let mut snap_a = {
                    let Some(sa) = world.get_component_store::<A>() else { return; };
                    crate::system::merge::Snapshot::clone_from_store(&sa)
                };
                let pivot = *[snap_a.dense.as_slice(), sb.dense.as_slice(), sc.dense.as_slice(), sd.dense.as_slice()]
                    .iter().min_by_key(|s| s.len()).unwrap();
                for &eid in pivot {
                    if let (Some(ia), Some(ib), Some(ic), Some(id)) = (snap_a.sparse[eid], sb.sparse[eid], sc.sparse[eid], sd.sparse[eid]) {
                        f(&mut snap_a.components[ia], &sb.components[ib], &sc.components[ic], &sd.components[id]);
                    }
                }
                snap_a.merge_into(world);
            }),
        ))
    }
}

// rwww
impl<A: crate::Component, B: Clone + crate::Component, C: Clone + crate::Component, D: Clone + crate::Component,
     T: FnMut(&A, &mut B, &mut C, &mut D) + Send + 'static>
    crate::system::system::IntoSystem<fn(&A, &mut B, &mut C, &mut D)> for T
{
    type System = Box<dyn crate::system::system::System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(crate::system::system::FunctionSystem::new(
            crate::system::system::FunctionSystemMetadata {
                name: std::any::type_name::<T>(),
                access: crate::system::access::ComponentAccess::new()
                    .reads::<A>().writes::<B>().writes::<C>().writes::<D>(),
            },
            Box::new(move |world| {
                let Some(sa) = world.get_component_store::<A>() else { return; };
                let (mut snap_b, mut snap_c, mut snap_d) = {
                    let Some(sb) = world.get_component_store::<B>() else { return; };
                    let Some(sc) = world.get_component_store::<C>() else { return; };
                    let Some(sd) = world.get_component_store::<D>() else { return; };
                    (
                        crate::system::merge::Snapshot::clone_from_store(&sb),
                        crate::system::merge::Snapshot::clone_from_store(&sc),
                        crate::system::merge::Snapshot::clone_from_store(&sd),
                    )
                };
                let pivot = *[sa.dense.as_slice(), snap_b.dense.as_slice(), snap_c.dense.as_slice(), snap_d.dense.as_slice()]
                    .iter().min_by_key(|s| s.len()).unwrap();
                for &eid in pivot {
                    if let (Some(ia), Some(ib), Some(ic), Some(id)) = (sa.sparse[eid], snap_b.sparse[eid], snap_c.sparse[eid], snap_d.sparse[eid]) {
                        f(&sa.components[ia], &mut snap_b.components[ib], &mut snap_c.components[ic], &mut snap_d.components[id]);
                    }
                }
                snap_b.merge_into(world); snap_c.merge_into(world); snap_d.merge_into(world);
            }),
        ))
    }
}

// rwwr
impl<A: crate::Component, B: Clone + crate::Component, C: Clone + crate::Component, D: crate::Component,
     T: FnMut(&A, &mut B, &mut C, &D) + Send + 'static>
    crate::system::system::IntoSystem<fn(&A, &mut B, &mut C, &D)> for T
{
    type System = Box<dyn crate::system::system::System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(crate::system::system::FunctionSystem::new(
            crate::system::system::FunctionSystemMetadata {
                name: std::any::type_name::<T>(),
                access: crate::system::access::ComponentAccess::new()
                    .reads::<A>().writes::<B>().writes::<C>().reads::<D>(),
            },
            Box::new(move |world| {
                let Some(sa) = world.get_component_store::<A>() else { return; };
                let Some(sd) = world.get_component_store::<D>() else { return; };
                let (mut snap_b, mut snap_c) = {
                    let Some(sb) = world.get_component_store::<B>() else { return; };
                    let Some(sc) = world.get_component_store::<C>() else { return; };
                    (
                        crate::system::merge::Snapshot::clone_from_store(&sb),
                        crate::system::merge::Snapshot::clone_from_store(&sc),
                    )
                };
                let pivot = *[sa.dense.as_slice(), snap_b.dense.as_slice(), snap_c.dense.as_slice(), sd.dense.as_slice()]
                    .iter().min_by_key(|s| s.len()).unwrap();
                for &eid in pivot {
                    if let (Some(ia), Some(ib), Some(ic), Some(id)) = (sa.sparse[eid], snap_b.sparse[eid], snap_c.sparse[eid], sd.sparse[eid]) {
                        f(&sa.components[ia], &mut snap_b.components[ib], &mut snap_c.components[ic], &sd.components[id]);
                    }
                }
                snap_b.merge_into(world); snap_c.merge_into(world);
            }),
        ))
    }
}

// rwrw
impl<A: crate::Component, B: Clone + crate::Component, C: crate::Component, D: Clone + crate::Component,
     T: FnMut(&A, &mut B, &C, &mut D) + Send + 'static>
    crate::system::system::IntoSystem<fn(&A, &mut B, &C, &mut D)> for T
{
    type System = Box<dyn crate::system::system::System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(crate::system::system::FunctionSystem::new(
            crate::system::system::FunctionSystemMetadata {
                name: std::any::type_name::<T>(),
                access: crate::system::access::ComponentAccess::new()
                    .reads::<A>().writes::<B>().reads::<C>().writes::<D>(),
            },
            Box::new(move |world| {
                let Some(sa) = world.get_component_store::<A>() else { return; };
                let Some(sc) = world.get_component_store::<C>() else { return; };
                let (mut snap_b, mut snap_d) = {
                    let Some(sb) = world.get_component_store::<B>() else { return; };
                    let Some(sd) = world.get_component_store::<D>() else { return; };
                    (
                        crate::system::merge::Snapshot::clone_from_store(&sb),
                        crate::system::merge::Snapshot::clone_from_store(&sd),
                    )
                };
                let pivot = *[sa.dense.as_slice(), snap_b.dense.as_slice(), sc.dense.as_slice(), snap_d.dense.as_slice()]
                    .iter().min_by_key(|s| s.len()).unwrap();
                for &eid in pivot {
                    if let (Some(ia), Some(ib), Some(ic), Some(id)) = (sa.sparse[eid], snap_b.sparse[eid], sc.sparse[eid], snap_d.sparse[eid]) {
                        f(&sa.components[ia], &mut snap_b.components[ib], &sc.components[ic], &mut snap_d.components[id]);
                    }
                }
                snap_b.merge_into(world); snap_d.merge_into(world);
            }),
        ))
    }
}

// rwrr
impl<A: crate::Component, B: Clone + crate::Component, C: crate::Component, D: crate::Component,
     T: FnMut(&A, &mut B, &C, &D) + Send + 'static>
    crate::system::system::IntoSystem<fn(&A, &mut B, &C, &D)> for T
{
    type System = Box<dyn crate::system::system::System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(crate::system::system::FunctionSystem::new(
            crate::system::system::FunctionSystemMetadata {
                name: std::any::type_name::<T>(),
                access: crate::system::access::ComponentAccess::new()
                    .reads::<A>().writes::<B>().reads::<C>().reads::<D>(),
            },
            Box::new(move |world| {
                let Some(sa) = world.get_component_store::<A>() else { return; };
                let Some(sc) = world.get_component_store::<C>() else { return; };
                let Some(sd) = world.get_component_store::<D>() else { return; };
                let mut snap_b = {
                    let Some(sb) = world.get_component_store::<B>() else { return; };
                    crate::system::merge::Snapshot::clone_from_store(&sb)
                };
                let pivot = *[sa.dense.as_slice(), snap_b.dense.as_slice(), sc.dense.as_slice(), sd.dense.as_slice()]
                    .iter().min_by_key(|s| s.len()).unwrap();
                for &eid in pivot {
                    if let (Some(ia), Some(ib), Some(ic), Some(id)) = (sa.sparse[eid], snap_b.sparse[eid], sc.sparse[eid], sd.sparse[eid]) {
                        f(&sa.components[ia], &mut snap_b.components[ib], &sc.components[ic], &sd.components[id]);
                    }
                }
                snap_b.merge_into(world);
            }),
        ))
    }
}

// rrww
impl<A: crate::Component, B: crate::Component, C: Clone + crate::Component, D: Clone + crate::Component,
     T: FnMut(&A, &B, &mut C, &mut D) + Send + 'static>
    crate::system::system::IntoSystem<fn(&A, &B, &mut C, &mut D)> for T
{
    type System = Box<dyn crate::system::system::System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(crate::system::system::FunctionSystem::new(
            crate::system::system::FunctionSystemMetadata {
                name: std::any::type_name::<T>(),
                access: crate::system::access::ComponentAccess::new()
                    .reads::<A>().reads::<B>().writes::<C>().writes::<D>(),
            },
            Box::new(move |world| {
                let Some(sa) = world.get_component_store::<A>() else { return; };
                let Some(sb) = world.get_component_store::<B>() else { return; };
                let (mut snap_c, mut snap_d) = {
                    let Some(sc) = world.get_component_store::<C>() else { return; };
                    let Some(sd) = world.get_component_store::<D>() else { return; };
                    (
                        crate::system::merge::Snapshot::clone_from_store(&sc),
                        crate::system::merge::Snapshot::clone_from_store(&sd),
                    )
                };
                let pivot = *[sa.dense.as_slice(), sb.dense.as_slice(), snap_c.dense.as_slice(), snap_d.dense.as_slice()]
                    .iter().min_by_key(|s| s.len()).unwrap();
                for &eid in pivot {
                    if let (Some(ia), Some(ib), Some(ic), Some(id)) = (sa.sparse[eid], sb.sparse[eid], snap_c.sparse[eid], snap_d.sparse[eid]) {
                        f(&sa.components[ia], &sb.components[ib], &mut snap_c.components[ic], &mut snap_d.components[id]);
                    }
                }
                snap_c.merge_into(world); snap_d.merge_into(world);
            }),
        ))
    }
}

// rrwr
impl<A: crate::Component, B: crate::Component, C: Clone + crate::Component, D: crate::Component,
     T: FnMut(&A, &B, &mut C, &D) + Send + 'static>
    crate::system::system::IntoSystem<fn(&A, &B, &mut C, &D)> for T
{
    type System = Box<dyn crate::system::system::System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(crate::system::system::FunctionSystem::new(
            crate::system::system::FunctionSystemMetadata {
                name: std::any::type_name::<T>(),
                access: crate::system::access::ComponentAccess::new()
                    .reads::<A>().reads::<B>().writes::<C>().reads::<D>(),
            },
            Box::new(move |world| {
                let Some(sa) = world.get_component_store::<A>() else { return; };
                let Some(sb) = world.get_component_store::<B>() else { return; };
                let Some(sd) = world.get_component_store::<D>() else { return; };
                let mut snap_c = {
                    let Some(sc) = world.get_component_store::<C>() else { return; };
                    crate::system::merge::Snapshot::clone_from_store(&sc)
                };
                let pivot = *[sa.dense.as_slice(), sb.dense.as_slice(), snap_c.dense.as_slice(), sd.dense.as_slice()]
                    .iter().min_by_key(|s| s.len()).unwrap();
                for &eid in pivot {
                    if let (Some(ia), Some(ib), Some(ic), Some(id)) = (sa.sparse[eid], sb.sparse[eid], snap_c.sparse[eid], sd.sparse[eid]) {
                        f(&sa.components[ia], &sb.components[ib], &mut snap_c.components[ic], &sd.components[id]);
                    }
                }
                snap_c.merge_into(world);
            }),
        ))
    }
}

// rrrw
impl<A: crate::Component, B: crate::Component, C: crate::Component, D: Clone + crate::Component,
     T: FnMut(&A, &B, &C, &mut D) + Send + 'static>
    crate::system::system::IntoSystem<fn(&A, &B, &C, &mut D)> for T
{
    type System = Box<dyn crate::system::system::System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(crate::system::system::FunctionSystem::new(
            crate::system::system::FunctionSystemMetadata {
                name: std::any::type_name::<T>(),
                access: crate::system::access::ComponentAccess::new()
                    .reads::<A>().reads::<B>().reads::<C>().writes::<D>(),
            },
            Box::new(move |world| {
                let Some(sa) = world.get_component_store::<A>() else { return; };
                let Some(sb) = world.get_component_store::<B>() else { return; };
                let Some(sc) = world.get_component_store::<C>() else { return; };
                let mut snap_d = {
                    let Some(sd) = world.get_component_store::<D>() else { return; };
                    crate::system::merge::Snapshot::clone_from_store(&sd)
                };
                let pivot = *[sa.dense.as_slice(), sb.dense.as_slice(), sc.dense.as_slice(), snap_d.dense.as_slice()]
                    .iter().min_by_key(|s| s.len()).unwrap();
                for &eid in pivot {
                    if let (Some(ia), Some(ib), Some(ic), Some(id)) = (sa.sparse[eid], sb.sparse[eid], sc.sparse[eid], snap_d.sparse[eid]) {
                        f(&sa.components[ia], &sb.components[ib], &sc.components[ic], &mut snap_d.components[id]);
                    }
                }
                snap_d.merge_into(world);
            }),
        ))
    }
}

// rrrr
impl<A: crate::Component, B: crate::Component, C: crate::Component, D: crate::Component,
     T: FnMut(&A, &B, &C, &D) + Send + 'static>
    crate::system::system::IntoSystem<fn(&A, &B, &C, &D)> for T
{
    type System = Box<dyn crate::system::system::System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(crate::system::system::FunctionSystem::new(
            crate::system::system::FunctionSystemMetadata {
                name: std::any::type_name::<T>(),
                access: crate::system::access::ComponentAccess::new()
                    .reads::<A>().reads::<B>().reads::<C>().reads::<D>(),
            },
            Box::new(move |world| {
                let Some(sa) = world.get_component_store::<A>() else { return; };
                let Some(sb) = world.get_component_store::<B>() else { return; };
                let Some(sc) = world.get_component_store::<C>() else { return; };
                let Some(sd) = world.get_component_store::<D>() else { return; };
                let pivot = *[sa.dense.as_slice(), sb.dense.as_slice(), sc.dense.as_slice(), sd.dense.as_slice()]
                    .iter().min_by_key(|s| s.len()).unwrap();
                for &eid in pivot {
                    if let (Some(ia), Some(ib), Some(ic), Some(id)) = (sa.sparse[eid], sb.sparse[eid], sc.sparse[eid], sd.sparse[eid]) {
                        f(&sa.components[ia], &sb.components[ib], &sc.components[ic], &sd.components[id]);
                    }
                }
            }),
        ))
    }
}
