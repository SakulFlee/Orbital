use crate::system::access::ComponentAccess;

pub trait System: Send {
    fn name(&self) -> &str;
    fn access(&self) -> &ComponentAccess;
    fn run(&mut self, world: &crate::World);
}

pub struct FunctionSystemMetadata {
    pub name: &'static str,
    pub access: ComponentAccess,
}

pub struct FunctionSystem {
    pub metadata: FunctionSystemMetadata,
    run_fn: Box<dyn FnMut(&crate::World) + Send>,
}

impl FunctionSystem {
    pub fn new(
        metadata: FunctionSystemMetadata,
        run_fn: Box<dyn FnMut(&crate::World) + Send>,
    ) -> Self {
        Self { metadata, run_fn }
    }
}

impl System for FunctionSystem {
    fn name(&self) -> &str {
        self.metadata.name
    }
    fn access(&self) -> &ComponentAccess {
        &self.metadata.access
    }
    fn run(&mut self, world: &crate::World) {
        (self.run_fn)(world);
    }
}

pub trait IntoSystem<Marker>: Sized {
    type System: System + 'static;
    fn into_system(self) -> Self::System;
}

impl System for Box<dyn System> {
    fn name(&self) -> &str {
        (**self).name()
    }
    fn access(&self) -> &ComponentAccess {
        (**self).access()
    }
    fn run(&mut self, world: &crate::World) {
        (**self).run(world)
    }
}
