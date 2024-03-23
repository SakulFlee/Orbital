use akimo_project::{
    error::RuntimeError,
    logging::*,
    runtime::{Runtime, RuntimeSettings},
};

fn main() -> Result<(), RuntimeError> {
    pollster::block_on(Runtime::liftoff(RuntimeSettings::default()))
}
