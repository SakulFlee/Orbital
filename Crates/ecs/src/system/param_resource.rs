use std::ops::{Deref, DerefMut};

use crate::Component;
use crate::system::access::ComponentAccess;
use crate::system::merge::Snapshot;
use crate::system::system::{FunctionSystem, FunctionSystemMetadata, IntoSystem, System};

// ---------------------------------------------------------------------------
// Res<T> — read-only resource handle
// ResMut<T> — writable resource handle
// ---------------------------------------------------------------------------

pub struct Res<'a, T: 'static>(&'a T);

impl<T: 'static> Clone for Res<'_, T> {
    fn clone(&self) -> Self {
        Res(self.0)
    }
}
impl<T: 'static> Copy for Res<'_, T> {}

impl<T: 'static> Deref for Res<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.0
    }
}

pub struct ResMut<'a, T: 'static>(&'a mut T);

impl<T: 'static> Deref for ResMut<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.0
    }
}

impl<T: 'static> DerefMut for ResMut<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.0
    }
}

// ---------------------------------------------------------------------------
// Arity 1
// ---------------------------------------------------------------------------

// Res<A> — read resource
impl<A: 'static + Send + Sync, F: for<'a> FnMut(Res<'a, A>) + Send + 'static>
    IntoSystem<fn(Res<'_, A>)> for F
{
    type System = Box<dyn System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(FunctionSystem::new(
            FunctionSystemMetadata {
                name: std::any::type_name::<F>(),
                access: ComponentAccess::new().reads::<A>(),
            },
            Box::new(move |world, _commands| {
                let Some(handle) = world.get_resource::<A>() else {
                    return;
                };
                f(Res(&*handle));
            }),
        ))
    }
}

// ResMut<A> — write resource
impl<A: 'static + Send + Sync, F: for<'a> FnMut(ResMut<'a, A>) + Send + 'static>
    IntoSystem<fn(ResMut<'_, A>)> for F
{
    type System = Box<dyn System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(FunctionSystem::new(
            FunctionSystemMetadata {
                name: std::any::type_name::<F>(),
                access: ComponentAccess::new().writes::<A>(),
            },
            Box::new(move |world, _commands| {
                let Some(mut handle) = world.get_resource_mut::<A>() else {
                    return;
                };
                f(ResMut(&mut *handle));
            }),
        ))
    }
}

// ---------------------------------------------------------------------------
// Arity 2 — helpers
// ---------------------------------------------------------------------------

macro_rules! impl_2_read_comp_res_read {
    ($a:ident, $b:ident) => {
        impl<
            A: Component,
            B: 'static + Send + Sync,
            F: for<'a> FnMut(&A, Res<'a, B>) + Send + 'static,
        > IntoSystem<fn(&A, Res<'_, B>)> for F
        {
            type System = Box<dyn System>;
            fn into_system(self) -> Self::System {
                let mut f = self;
                Box::new(FunctionSystem::new(
                    FunctionSystemMetadata {
                        name: std::any::type_name::<F>(),
                        access: ComponentAccess::new().reads::<A>().reads::<B>(),
                    },
                    Box::new(move |world, _commands| {
                        let Some(handle) = world.get_resource::<B>() else {
                            return;
                        };
                        let Some(sa) = world.get_component_store::<A>() else {
                            return;
                        };
                        let rb = Res(&*handle);
                        for &eid in sa.dense.as_slice() {
                            if let Some(ia) = sa.sparse[eid] {
                                f(&sa.components[ia], rb);
                            }
                        }
                    }),
                ))
            }
        }
    };
}

macro_rules! impl_2_read_comp_res_write {
    ($a:ident, $b:ident) => {
        impl<
            A: Component,
            B: 'static + Send + Sync,
            F: for<'a> FnMut(&A, ResMut<'a, B>) + Send + 'static,
        > IntoSystem<fn(&A, ResMut<'_, B>)> for F
        {
            type System = Box<dyn System>;
            fn into_system(self) -> Self::System {
                let mut f = self;
                Box::new(FunctionSystem::new(
                    FunctionSystemMetadata {
                        name: std::any::type_name::<F>(),
                        access: ComponentAccess::new().reads::<A>().writes::<B>(),
                    },
                    Box::new(move |world, _commands| {
                        let Some(mut handle) = world.get_resource_mut::<B>() else {
                            return;
                        };
                        let Some(sa) = world.get_component_store::<A>() else {
                            return;
                        };
                        for &eid in sa.dense.as_slice() {
                            if let Some(ia) = sa.sparse[eid] {
                                f(&sa.components[ia], ResMut(&mut *handle));
                            }
                        }
                    }),
                ))
            }
        }
    };
}

macro_rules! impl_2_write_comp_res_read {
    ($a:ident, $b:ident) => {
        impl<
            A: Clone + Component,
            B: 'static + Send + Sync,
            F: for<'a> FnMut(&mut A, Res<'a, B>) + Send + 'static,
        > IntoSystem<fn(&mut A, Res<'_, B>)> for F
        {
            type System = Box<dyn System>;
            fn into_system(self) -> Self::System {
                let mut f = self;
                Box::new(FunctionSystem::new(
                    FunctionSystemMetadata {
                        name: std::any::type_name::<F>(),
                        access: ComponentAccess::new().writes::<A>().reads::<B>(),
                    },
                    Box::new(move |world, _commands| {
                        let Some(handle) = world.get_resource::<B>() else {
                            return;
                        };
                        let mut snap_a = {
                            let Some(sa) = world.get_component_store::<A>() else {
                                return;
                            };
                            Snapshot::clone_from_store(&sa)
                        };
                        let rb = Res(&*handle);
                        for &eid in snap_a.dense.as_slice() {
                            if let Some(ia) = snap_a.sparse[eid] {
                                f(&mut snap_a.components[ia], rb);
                            }
                        }
                        snap_a.merge_into(world);
                    }),
                ))
            }
        }
    };
}

macro_rules! impl_2_write_comp_res_write {
    ($a:ident, $b:ident) => {
        impl<
            A: Clone + Component,
            B: 'static + Send + Sync,
            F: for<'a> FnMut(&mut A, ResMut<'a, B>) + Send + 'static,
        > IntoSystem<fn(&mut A, ResMut<'_, B>)> for F
        {
            type System = Box<dyn System>;
            fn into_system(self) -> Self::System {
                let mut f = self;
                Box::new(FunctionSystem::new(
                    FunctionSystemMetadata {
                        name: std::any::type_name::<F>(),
                        access: ComponentAccess::new().writes::<A>().writes::<B>(),
                    },
                    Box::new(move |world, _commands| {
                        let Some(mut handle) = world.get_resource_mut::<B>() else {
                            return;
                        };
                        let mut snap_a = {
                            let Some(sa) = world.get_component_store::<A>() else {
                                return;
                            };
                            Snapshot::clone_from_store(&sa)
                        };
                        for &eid in snap_a.dense.as_slice() {
                            if let Some(ia) = snap_a.sparse[eid] {
                                f(&mut snap_a.components[ia], ResMut(&mut *handle));
                            }
                        }
                        snap_a.merge_into(world);
                    }),
                ))
            }
        }
    };
}

macro_rules! impl_2_res_read_res_read {
    ($a:ident, $b:ident) => {
        impl<
            A: 'static + Send + Sync,
            B: 'static + Send + Sync,
            F: for<'a, 'b> FnMut(Res<'a, A>, Res<'b, B>) + Send + 'static,
        > IntoSystem<fn(Res<'_, A>, Res<'_, B>)> for F
        {
            type System = Box<dyn System>;
            fn into_system(self) -> Self::System {
                let mut f = self;
                Box::new(FunctionSystem::new(
                    FunctionSystemMetadata {
                        name: std::any::type_name::<F>(),
                        access: ComponentAccess::new().reads::<A>().reads::<B>(),
                    },
                    Box::new(move |world, _commands| {
                        let Some(ha) = world.get_resource::<A>() else {
                            return;
                        };
                        let Some(hb) = world.get_resource::<B>() else {
                            return;
                        };
                        f(Res(&*ha), Res(&*hb));
                    }),
                ))
            }
        }
    };
}

macro_rules! impl_2_res_read_res_write {
    ($a:ident, $b:ident) => {
        impl<
            A: 'static + Send + Sync,
            B: 'static + Send + Sync,
            F: for<'a, 'b> FnMut(Res<'a, A>, ResMut<'b, B>) + Send + 'static,
        > IntoSystem<fn(Res<'_, A>, ResMut<'_, B>)> for F
        {
            type System = Box<dyn System>;
            fn into_system(self) -> Self::System {
                let mut f = self;
                Box::new(FunctionSystem::new(
                    FunctionSystemMetadata {
                        name: std::any::type_name::<F>(),
                        access: ComponentAccess::new().reads::<A>().writes::<B>(),
                    },
                    Box::new(move |world, _commands| {
                        let Some(ha) = world.get_resource::<A>() else {
                            return;
                        };
                        let Some(mut hb) = world.get_resource_mut::<B>() else {
                            return;
                        };
                        f(Res(&*ha), ResMut(&mut *hb));
                    }),
                ))
            }
        }
    };
}

macro_rules! impl_2_res_write_res_read {
    ($a:ident, $b:ident) => {
        impl<
            A: 'static + Send + Sync,
            B: 'static + Send + Sync,
            F: for<'a, 'b> FnMut(ResMut<'a, A>, Res<'b, B>) + Send + 'static,
        > IntoSystem<fn(ResMut<'_, A>, Res<'_, B>)> for F
        {
            type System = Box<dyn System>;
            fn into_system(self) -> Self::System {
                let mut f = self;
                Box::new(FunctionSystem::new(
                    FunctionSystemMetadata {
                        name: std::any::type_name::<F>(),
                        access: ComponentAccess::new().writes::<A>().reads::<B>(),
                    },
                    Box::new(move |world, _commands| {
                        let Some(mut ha) = world.get_resource_mut::<A>() else {
                            return;
                        };
                        let Some(hb) = world.get_resource::<B>() else {
                            return;
                        };
                        f(ResMut(&mut *ha), Res(&*hb));
                    }),
                ))
            }
        }
    };
}

macro_rules! impl_2_res_write_res_write {
    ($a:ident, $b:ident) => {
        impl<
            A: 'static + Send + Sync,
            B: 'static + Send + Sync,
            F: for<'a, 'b> FnMut(ResMut<'a, A>, ResMut<'b, B>) + Send + 'static,
        > IntoSystem<fn(ResMut<'_, A>, ResMut<'_, B>)> for F
        {
            type System = Box<dyn System>;
            fn into_system(self) -> Self::System {
                let mut f = self;
                Box::new(FunctionSystem::new(
                    FunctionSystemMetadata {
                        name: std::any::type_name::<F>(),
                        access: ComponentAccess::new().writes::<A>().writes::<B>(),
                    },
                    Box::new(move |world, _commands| {
                        let Some(mut ha) = world.get_resource_mut::<A>() else {
                            return;
                        };
                        let Some(mut hb) = world.get_resource_mut::<B>() else {
                            return;
                        };
                        f(ResMut(&mut *ha), ResMut(&mut *hb));
                    }),
                ))
            }
        }
    };
}

macro_rules! impl_2_res_write_comp_read {
    ($a:ident, $b:ident) => {
        impl<
            A: 'static + Send + Sync,
            B: Component,
            F: for<'a> FnMut(ResMut<'a, A>, &B) + Send + 'static,
        > IntoSystem<fn(ResMut<'_, A>, &B)> for F
        {
            type System = Box<dyn System>;
            fn into_system(self) -> Self::System {
                let mut f = self;
                Box::new(FunctionSystem::new(
                    FunctionSystemMetadata {
                        name: std::any::type_name::<F>(),
                        access: ComponentAccess::new().writes::<A>().reads::<B>(),
                    },
                    Box::new(move |world, _commands| {
                        let Some(mut handle) = world.get_resource_mut::<A>() else {
                            return;
                        };
                        let Some(sb) = world.get_component_store::<B>() else {
                            return;
                        };
                        for &eid in sb.dense.as_slice() {
                            if let Some(ib) = sb.sparse[eid] {
                                f(ResMut(&mut *handle), &sb.components[ib]);
                            }
                        }
                    }),
                ))
            }
        }
    };
}

macro_rules! impl_2_res_write_comp_write {
    ($a:ident, $b:ident) => {
        impl<
            A: 'static + Send + Sync,
            B: Clone + Component,
            F: for<'a> FnMut(ResMut<'a, A>, &mut B) + Send + 'static,
        > IntoSystem<fn(ResMut<'_, A>, &mut B)> for F
        {
            type System = Box<dyn System>;
            fn into_system(self) -> Self::System {
                let mut f = self;
                Box::new(FunctionSystem::new(
                    FunctionSystemMetadata {
                        name: std::any::type_name::<F>(),
                        access: ComponentAccess::new().writes::<A>().writes::<B>(),
                    },
                    Box::new(move |world, _commands| {
                        let Some(mut handle) = world.get_resource_mut::<A>() else {
                            return;
                        };
                        let mut snap_b = {
                            let Some(sb) = world.get_component_store::<B>() else {
                                return;
                            };
                            Snapshot::clone_from_store(&sb)
                        };
                        for &eid in snap_b.dense.as_slice() {
                            if let Some(ib) = snap_b.sparse[eid] {
                                f(ResMut(&mut *handle), &mut snap_b.components[ib]);
                            }
                        }
                        snap_b.merge_into(world);
                    }),
                ))
            }
        }
    };
}

// ---------------------------------------------------------------------------
// Arity 2 — invocations
// ---------------------------------------------------------------------------

impl_2_read_comp_res_read!(A, B);
impl_2_read_comp_res_write!(A, B);
impl_2_write_comp_res_read!(A, B);
impl_2_write_comp_res_write!(A, B);
impl_2_res_read_res_read!(A, B);
impl_2_res_read_res_write!(A, B);
impl_2_res_write_res_read!(A, B);
impl_2_res_write_res_write!(A, B);
impl_2_res_write_comp_read!(A, B);
impl_2_res_write_comp_write!(A, B);

// ---------------------------------------------------------------------------
// Arity 3 — Res<A> + Res<B> + &mut C   (read resource + read resource + write component)
// ---------------------------------------------------------------------------

impl<
    A: 'static + Send + Sync,
    B: 'static + Send + Sync,
    C: Clone + Component,
    F: for<'a, 'b> FnMut(Res<'a, A>, Res<'b, B>, &mut C) + Send + 'static,
> IntoSystem<fn(Res<'_, A>, Res<'_, B>, &mut C)> for F
{
    type System = Box<dyn System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(FunctionSystem::new(
            FunctionSystemMetadata {
                name: std::any::type_name::<F>(),
                access: ComponentAccess::new()
                    .reads::<A>()
                    .reads::<B>()
                    .writes::<C>(),
            },
            Box::new(move |world, _commands| {
                let ha = match world.get_resource::<A>() {
                    Some(h) => h,
                    None => return,
                };
                let hb = match world.get_resource::<B>() {
                    Some(h) => h,
                    None => return,
                };
                let mut snap_c = {
                    let Some(sc) = world.get_component_store::<C>() else {
                        return;
                    };
                    Snapshot::clone_from_store(&sc)
                };
                let ra = Res(&*ha);
                let rb = Res(&*hb);
                for &eid in snap_c.dense.as_slice() {
                    if let Some(ic) = snap_c.sparse[eid] {
                        f(ra, rb, &mut snap_c.components[ic]);
                    }
                }
                snap_c.merge_into(world);
            }),
        ))
    }
}

// ---------------------------------------------------------------------------
// Arity 4 — Res<A> + Res<B> + &mut C + &mut D   (2 read resources + 2 write components)
// ---------------------------------------------------------------------------

impl<
    A: 'static + Send + Sync,
    B: 'static + Send + Sync,
    C: Clone + Component,
    D: Clone + Component,
    F: for<'a, 'b> FnMut(Res<'a, A>, Res<'b, B>, &mut C, &mut D) + Send + 'static,
> IntoSystem<fn(Res<'_, A>, Res<'_, B>, &mut C, &mut D)> for F
{
    type System = Box<dyn System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(FunctionSystem::new(
            FunctionSystemMetadata {
                name: std::any::type_name::<F>(),
                access: ComponentAccess::new()
                    .reads::<A>()
                    .reads::<B>()
                    .writes::<C>()
                    .writes::<D>(),
            },
            Box::new(move |world, _commands| {
                let ha = match world.get_resource::<A>() {
                    Some(h) => h,
                    None => return,
                };
                let hb = match world.get_resource::<B>() {
                    Some(h) => h,
                    None => return,
                };
                let (mut snap_c, mut snap_d) = {
                    let Some(sc) = world.get_component_store::<C>() else {
                        return;
                    };
                    let Some(sd) = world.get_component_store::<D>() else {
                        return;
                    };
                    (
                        Snapshot::clone_from_store(&sc),
                        Snapshot::clone_from_store(&sd),
                    )
                };
                let pivot = if snap_c.dense.len() <= snap_d.dense.len() {
                    snap_c.dense.as_slice()
                } else {
                    snap_d.dense.as_slice()
                };
                let ra = Res(&*ha);
                let rb = Res(&*hb);
                for &eid in pivot {
                    if let (Some(ic), Some(id)) = (snap_c.sparse[eid], snap_d.sparse[eid]) {
                        f(
                            ra,
                            rb,
                            &mut snap_c.components[ic],
                            &mut snap_d.components[id],
                        );
                    }
                }
                snap_c.merge_into(world);
                snap_d.merge_into(world);
            }),
        ))
    }
}

// ---------------------------------------------------------------------------
// Arity 3 — Res<A> + &B + &C   (read resource + read component + read component)
// ---------------------------------------------------------------------------

impl<
    A: 'static + Send + Sync,
    B: Component,
    C: Component,
    F: for<'a> FnMut(Res<'a, A>, &B, &C) + Send + 'static,
> IntoSystem<fn(Res<'_, A>, &B, &C)> for F
{
    type System = Box<dyn System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(FunctionSystem::new(
            FunctionSystemMetadata {
                name: std::any::type_name::<F>(),
                access: ComponentAccess::new()
                    .reads::<A>()
                    .reads::<B>()
                    .reads::<C>(),
            },
            Box::new(move |world, _commands| {
                let Some(handle) = world.get_resource::<A>() else {
                    return;
                };
                let Some(sb) = world.get_component_store::<B>() else {
                    return;
                };
                let Some(sc) = world.get_component_store::<C>() else {
                    return;
                };
                let pivot = if sb.dense.len() <= sc.dense.len() {
                    sb.dense.as_slice()
                } else {
                    sc.dense.as_slice()
                };
                let ra = Res(&*handle);
                for &eid in pivot {
                    if let (Some(ib), Some(ic)) = (sb.sparse[eid], sc.sparse[eid]) {
                        f(ra, &sb.components[ib], &sc.components[ic]);
                    }
                }
            }),
        ))
    }
}

// ---------------------------------------------------------------------------
// Arity 3 — Res<A> + Res<B> + &C   (read resource + read resource + read component)
// ---------------------------------------------------------------------------

impl<
    A: 'static + Send + Sync,
    B: 'static + Send + Sync,
    C: Component,
    F: for<'a, 'b> FnMut(Res<'a, A>, Res<'b, B>, &C) + Send + 'static,
> IntoSystem<fn(Res<'_, A>, Res<'_, B>, &C)> for F
{
    type System = Box<dyn System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(FunctionSystem::new(
            FunctionSystemMetadata {
                name: std::any::type_name::<F>(),
                access: ComponentAccess::new()
                    .reads::<A>()
                    .reads::<B>()
                    .reads::<C>(),
            },
            Box::new(move |world, _commands| {
                let Some(ha) = world.get_resource::<A>() else {
                    return;
                };
                let Some(hb) = world.get_resource::<B>() else {
                    return;
                };
                let Some(sc) = world.get_component_store::<C>() else {
                    return;
                };
                let ra = Res(&*ha);
                let rb = Res(&*hb);
                for &eid in sc.dense.as_slice() {
                    if let Some(ic) = sc.sparse[eid] {
                        f(ra, rb, &sc.components[ic]);
                    }
                }
            }),
        ))
    }
}

// ---------------------------------------------------------------------------
// Arity 3 — Res<A> + &B + Res<C>   (read resource + read component + read resource)
// ---------------------------------------------------------------------------

impl<
    A: 'static + Send + Sync,
    B: Component,
    C: 'static + Send + Sync,
    F: for<'a, 'b> FnMut(Res<'a, A>, &B, Res<'b, C>) + Send + 'static,
> IntoSystem<fn(Res<'_, A>, &B, Res<'_, C>)> for F
{
    type System = Box<dyn System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(FunctionSystem::new(
            FunctionSystemMetadata {
                name: std::any::type_name::<F>(),
                access: ComponentAccess::new()
                    .reads::<A>()
                    .reads::<B>()
                    .reads::<C>(),
            },
            Box::new(move |world, _commands| {
                let Some(ha) = world.get_resource::<A>() else {
                    return;
                };
                let Some(hc) = world.get_resource::<C>() else {
                    return;
                };
                let Some(sb) = world.get_component_store::<B>() else {
                    return;
                };
                let ra = Res(&*ha);
                let rc = Res(&*hc);
                for &eid in sb.dense.as_slice() {
                    if let Some(ib) = sb.sparse[eid] {
                        f(ra, &sb.components[ib], rc);
                    }
                }
            }),
        ))
    }
}

// ---------------------------------------------------------------------------
// Arity 3 — &A + Res<B> + Res<C>   (read component + read resource + read resource)
// ---------------------------------------------------------------------------

impl<
    A: Component,
    B: 'static + Send + Sync,
    C: 'static + Send + Sync,
    F: for<'a, 'b> FnMut(&A, Res<'a, B>, Res<'b, C>) + Send + 'static,
> IntoSystem<fn(&A, Res<'_, B>, Res<'_, C>)> for F
{
    type System = Box<dyn System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(FunctionSystem::new(
            FunctionSystemMetadata {
                name: std::any::type_name::<F>(),
                access: ComponentAccess::new()
                    .reads::<A>()
                    .reads::<B>()
                    .reads::<C>(),
            },
            Box::new(move |world, _commands| {
                let Some(hb) = world.get_resource::<B>() else {
                    return;
                };
                let Some(hc) = world.get_resource::<C>() else {
                    return;
                };
                let Some(sa) = world.get_component_store::<A>() else {
                    return;
                };
                let rb = Res(&*hb);
                let rc = Res(&*hc);
                for &eid in sa.dense.as_slice() {
                    if let Some(ia) = sa.sparse[eid] {
                        f(&sa.components[ia], rb, rc);
                    }
                }
            }),
        ))
    }
}

// ---------------------------------------------------------------------------
// Arity 3 — ResMut<A> + &B + &C   (write resource + read component + read component)
// ---------------------------------------------------------------------------

impl<
    A: 'static + Send + Sync,
    B: Component,
    C: Component,
    F: for<'a> FnMut(ResMut<'a, A>, &B, &C) + Send + 'static,
> IntoSystem<fn(ResMut<'_, A>, &B, &C)> for F
{
    type System = Box<dyn System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(FunctionSystem::new(
            FunctionSystemMetadata {
                name: std::any::type_name::<F>(),
                access: ComponentAccess::new()
                    .writes::<A>()
                    .reads::<B>()
                    .reads::<C>(),
            },
            Box::new(move |world, _commands| {
                let Some(mut handle) = world.get_resource_mut::<A>() else {
                    return;
                };
                let Some(sb) = world.get_component_store::<B>() else {
                    return;
                };
                let Some(sc) = world.get_component_store::<C>() else {
                    return;
                };
                let pivot = if sb.dense.len() <= sc.dense.len() {
                    sb.dense.as_slice()
                } else {
                    sc.dense.as_slice()
                };
                for &eid in pivot {
                    if let (Some(ib), Some(ic)) = (sb.sparse[eid], sc.sparse[eid]) {
                        f(ResMut(&mut *handle), &sb.components[ib], &sc.components[ic]);
                    }
                }
            }),
        ))
    }
}

// ---------------------------------------------------------------------------
// Arity 3 — &A + ResMut<B> + &C   (read component + write resource + read component)
// ---------------------------------------------------------------------------

impl<
    A: Component,
    B: 'static + Send + Sync,
    C: Component,
    F: for<'a> FnMut(&A, ResMut<'a, B>, &C) + Send + 'static,
> IntoSystem<fn(&A, ResMut<'_, B>, &C)> for F
{
    type System = Box<dyn System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(FunctionSystem::new(
            FunctionSystemMetadata {
                name: std::any::type_name::<F>(),
                access: ComponentAccess::new()
                    .reads::<A>()
                    .writes::<B>()
                    .reads::<C>(),
            },
            Box::new(move |world, _commands| {
                let Some(mut hb) = world.get_resource_mut::<B>() else {
                    return;
                };
                let Some(sa) = world.get_component_store::<A>() else {
                    return;
                };
                let Some(sc) = world.get_component_store::<C>() else {
                    return;
                };
                let pivot = if sa.dense.len() <= sc.dense.len() {
                    sa.dense.as_slice()
                } else {
                    sc.dense.as_slice()
                };
                for &eid in pivot {
                    if let (Some(ia), Some(ic)) = (sa.sparse[eid], sc.sparse[eid]) {
                        f(&sa.components[ia], ResMut(&mut *hb), &sc.components[ic]);
                    }
                }
            }),
        ))
    }
}

// ---------------------------------------------------------------------------
// Arity 3 — &A + &B + ResMut<C>   (read component + read component + write resource)
// ---------------------------------------------------------------------------

impl<
    A: Component,
    B: Component,
    C: 'static + Send + Sync,
    F: for<'a> FnMut(&A, &B, ResMut<'a, C>) + Send + 'static,
> IntoSystem<fn(&A, &B, ResMut<'_, C>)> for F
{
    type System = Box<dyn System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(FunctionSystem::new(
            FunctionSystemMetadata {
                name: std::any::type_name::<F>(),
                access: ComponentAccess::new()
                    .reads::<A>()
                    .reads::<B>()
                    .writes::<C>(),
            },
            Box::new(move |world, _commands| {
                let Some(mut hc) = world.get_resource_mut::<C>() else {
                    return;
                };
                let Some(sa) = world.get_component_store::<A>() else {
                    return;
                };
                let Some(sb) = world.get_component_store::<B>() else {
                    return;
                };
                let pivot = if sa.dense.len() <= sb.dense.len() {
                    sa.dense.as_slice()
                } else {
                    sb.dense.as_slice()
                };
                for &eid in pivot {
                    if let (Some(ia), Some(ib)) = (sa.sparse[eid], sb.sparse[eid]) {
                        f(&sa.components[ia], &sb.components[ib], ResMut(&mut *hc));
                    }
                }
            }),
        ))
    }
}

// ---------------------------------------------------------------------------
// Arity 4 — Res<A> + &B + &C + &D   (read resource + read comp + read comp + read comp)
// ---------------------------------------------------------------------------

impl<
    A: 'static + Send + Sync,
    B: Component,
    C: Component,
    D: Component,
    F: for<'a> FnMut(Res<'a, A>, &B, &C, &D) + Send + 'static,
> IntoSystem<fn(Res<'_, A>, &B, &C, &D)> for F
{
    type System = Box<dyn System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(FunctionSystem::new(
            FunctionSystemMetadata {
                name: std::any::type_name::<F>(),
                access: ComponentAccess::new()
                    .reads::<A>()
                    .reads::<B>()
                    .reads::<C>()
                    .reads::<D>(),
            },
            Box::new(move |world, _commands| {
                let Some(handle) = world.get_resource::<A>() else {
                    return;
                };
                let Some(sb) = world.get_component_store::<B>() else {
                    return;
                };
                let Some(sc) = world.get_component_store::<C>() else {
                    return;
                };
                let Some(sd) = world.get_component_store::<D>() else {
                    return;
                };
                let pivot = *[
                    sb.dense.as_slice(),
                    sc.dense.as_slice(),
                    sd.dense.as_slice(),
                ]
                .iter()
                .min_by_key(|s| s.len())
                .unwrap();
                let ra = Res(&*handle);
                for &eid in pivot {
                    if let (Some(ib), Some(ic), Some(id)) =
                        (sb.sparse[eid], sc.sparse[eid], sd.sparse[eid])
                    {
                        f(
                            ra,
                            &sb.components[ib],
                            &sc.components[ic],
                            &sd.components[id],
                        );
                    }
                }
            }),
        ))
    }
}

// ---------------------------------------------------------------------------
// Arity 4 — ResMut<A> + &B + &C + &D   (write resource + read comp + read comp + read comp)
// ---------------------------------------------------------------------------

impl<
    A: 'static + Send + Sync,
    B: Component,
    C: Component,
    D: Component,
    F: for<'a> FnMut(ResMut<'a, A>, &B, &C, &D) + Send + 'static,
> IntoSystem<fn(ResMut<'_, A>, &B, &C, &D)> for F
{
    type System = Box<dyn System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(FunctionSystem::new(
            FunctionSystemMetadata {
                name: std::any::type_name::<F>(),
                access: ComponentAccess::new()
                    .writes::<A>()
                    .reads::<B>()
                    .reads::<C>()
                    .reads::<D>(),
            },
            Box::new(move |world, _commands| {
                let Some(mut handle) = world.get_resource_mut::<A>() else {
                    return;
                };
                let Some(sb) = world.get_component_store::<B>() else {
                    return;
                };
                let Some(sc) = world.get_component_store::<C>() else {
                    return;
                };
                let Some(sd) = world.get_component_store::<D>() else {
                    return;
                };
                let pivot = *[
                    sb.dense.as_slice(),
                    sc.dense.as_slice(),
                    sd.dense.as_slice(),
                ]
                .iter()
                .min_by_key(|s| s.len())
                .unwrap();
                for &eid in pivot {
                    if let (Some(ib), Some(ic), Some(id)) =
                        (sb.sparse[eid], sc.sparse[eid], sd.sparse[eid])
                    {
                        f(
                            ResMut(&mut *handle),
                            &sb.components[ib],
                            &sc.components[ic],
                            &sd.components[id],
                        );
                    }
                }
            }),
        ))
    }
}

// ---------------------------------------------------------------------------
// Commands parameter support
// ---------------------------------------------------------------------------

// fn(&mut Commands) — commands-only system
// The system receives a mutable reference to the schedule's commands buffer.
impl<F: FnMut(&mut crate::Commands) + Send + 'static> IntoSystem<fn(&mut crate::Commands)> for F {
    type System = Box<dyn crate::system::system::System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(crate::system::system::FunctionSystem::new(
            crate::system::system::FunctionSystemMetadata {
                name: std::any::type_name::<F>(),
                access: crate::system::access::ComponentAccess::new(),
            },
            Box::new(move |_world, commands| {
                f(commands);
            }),
        ))
    }
}

// fn(&mut Commands, Res<A>) — commands + read resource
impl<A: 'static + Send + Sync, F: for<'a> FnMut(&mut crate::Commands, Res<'a, A>) + Send + 'static>
    IntoSystem<fn(&mut crate::Commands, Res<'_, A>)> for F
{
    type System = Box<dyn crate::system::system::System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(crate::system::system::FunctionSystem::new(
            crate::system::system::FunctionSystemMetadata {
                name: std::any::type_name::<F>(),
                access: crate::system::access::ComponentAccess::new().reads::<A>(),
            },
            Box::new(move |world, commands| {
                let handle = match world.get_resource::<A>() {
                    Some(h) => h,
                    None => return,
                };
                f(commands, Res(&*handle));
            }),
        ))
    }
}

// fn(&mut Commands, ResMut<A>) — commands + write resource
impl<
    A: 'static + Send + Sync,
    F: for<'a> FnMut(&mut crate::Commands, ResMut<'a, A>) + Send + 'static,
> IntoSystem<fn(&mut crate::Commands, ResMut<'_, A>)> for F
{
    type System = Box<dyn crate::system::system::System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(crate::system::system::FunctionSystem::new(
            crate::system::system::FunctionSystemMetadata {
                name: std::any::type_name::<F>(),
                access: crate::system::access::ComponentAccess::new().writes::<A>(),
            },
            Box::new(move |world, commands| {
                let mut handle = match world.get_resource_mut::<A>() {
                    Some(h) => h,
                    None => return,
                };
                f(commands, ResMut(&mut *handle));
            }),
        ))
    }
}
