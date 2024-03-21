use akimo_project::{error::RuntimeError, logging::*, runtime::Runtime};

fn main() -> Result<(), RuntimeError> {
    log_init();

    info!("Akimo-Project: Engine");
    info!("(C) SakulFlee 2024");

    Runtime::liftoff()
}
