use std::ops::{Deref, DerefMut};

use crate::system::access::ComponentAccess;
use crate::system::merge::Snapshot;
use crate::system::system::{FunctionSystem, FunctionSystemMetadata, IntoSystem, System};
use crate::Component;

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
impl<
    A: 'static + Send + Sync,
    F: for<'a> FnMut(Res<'a, A>) + Send + 'static,
> IntoSystem<fn(Res<'_, A>)> for F
{
    type System = Box<dyn System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(FunctionSystem::new(
            FunctionSystemMetadata {
                name: std::any::type_name::<F>(),
                access: ComponentAccess::new().reads::<A>(),
            },
            Box::new(move |world| {
                let Some(handle) = world.get_resource::<A>() else { return; };
                f(Res(&*handle));
            }),
        ))
    }
}

// ResMut<A> — write resource
impl<
    A: 'static + Send + Sync,
    F: for<'a> FnMut(ResMut<'a, A>) + Send + 'static,
> IntoSystem<fn(ResMut<'_, A>)> for F
{
    type System = Box<dyn System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(FunctionSystem::new(
            FunctionSystemMetadata {
                name: std::any::type_name::<F>(),
                access: ComponentAccess::new().writes::<A>(),
            },
            Box::new(move |world| {
                let Some(mut handle) = world.get_resource_mut::<A>() else { return; };
                f(ResMut(&mut *handle));
            }),
        ))
    }
}

// ---------------------------------------------------------------------------
// Arity 2 — Res<A> + &B   (read resource + read component)
// ---------------------------------------------------------------------------

impl<
    A: 'static + Send + Sync,
    B: Component,
    F: for<'a> FnMut(Res<'a, A>, &B) + Send + 'static,
> IntoSystem<fn(Res<'_, A>, &B)> for F
{
    type System = Box<dyn System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(FunctionSystem::new(
            FunctionSystemMetadata {
                name: std::any::type_name::<F>(),
                access: ComponentAccess::new().reads::<A>().reads::<B>(),
            },
            Box::new(move |world| {
                let Some(handle) = world.get_resource::<A>() else { return; };
                let Some(sb) = world.get_component_store::<B>() else { return; };
                let ra = Res(&*handle);
                for &eid in sb.dense.as_slice() {
                    if let Some(ib) = sb.sparse[eid] {
                        f(ra, &sb.components[ib]);
                    }
                }
            }),
        ))
    }
}

// ---------------------------------------------------------------------------
// Arity 2 — Res<A> + &mut B   (read resource + write component)
// ---------------------------------------------------------------------------

impl<
    A: 'static + Send + Sync,
    B: Clone + Component,
    F: for<'a> FnMut(Res<'a, A>, &mut B) + Send + 'static,
> IntoSystem<fn(Res<'_, A>, &mut B)> for F
{
    type System = Box<dyn System>;
    fn into_system(self) -> Self::System {
        let mut f = self;
        Box::new(FunctionSystem::new(
            FunctionSystemMetadata {
                name: std::any::type_name::<F>(),
                access: ComponentAccess::new().reads::<A>().writes::<B>(),
            },
            Box::new(move |world| {
                let Some(handle) = world.get_resource::<A>() else { return; };
                let mut snap_b = {
                    let Some(sb) = world.get_component_store::<B>() else { return; };
                    Snapshot::clone_from_store(&sb)
                };
                let ra = Res(&*handle);
                for &eid in snap_b.dense.as_slice() {
                    if let Some(ib) = snap_b.sparse[eid] {
                        f(ra, &mut snap_b.components[ib]);
                    }
                }
                snap_b.merge_into(world);
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
            Box::new(move |world| {
                let Some(handle) = world.get_resource::<A>() else { return; };
                let Some(sb) = world.get_component_store::<B>() else { return; };
                let Some(sc) = world.get_component_store::<C>() else { return; };
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
