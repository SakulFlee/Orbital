mod commands;
pub use commands::*;

mod encoder;
pub use encoder::*;

#[repr(u8)]
pub enum Command {
    Entity(EntityCommand) = 0,
    Component(ComponentCommand) = 1,
    System(SystemCommand) = 2,
}

pub enum EntityCommand {
    Spawn,
    Remove,
}

pub enum ComponentCommand {
    Attach,
    Detach,
    Update,
}

pub enum SystemCommand {
    Register,
    Remove,
}
