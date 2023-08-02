#[cfg_attr(
    all(
        feature = "64-threads",
        not(any(
            feature = "32-threads",
            feature = "16-threads",
            feature = "8-threads",
            feature = "4-threads"
        ))
    ),
    tokio::main(worker_threads = 64)
)]
#[cfg_attr(
    all(
        feature = "32-threads",
        not(any(
            feature = "64-threads",
            feature = "16-threads",
            feature = "8-threads",
            feature = "4-threads"
        ))
    ),
    tokio::main(worker_threads = 32)
)]
#[cfg_attr(
    all(
        feature = "16-threads",
        not(any(
            feature = "64-threads",
            feature = "32-threads",
            feature = "8-threads",
            feature = "4-threads"
        ))
    ),
    tokio::main(worker_threads = 16)
)]
#[cfg_attr(
    all(
        feature = "8-threads",
        not(any(
            feature = "64-threads",
            feature = "32-threads",
            feature = "16-threads",
            feature = "4-threads"
        ))
    ),
    tokio::main(worker_threads = 8)
)]
#[cfg_attr(
    all(
        feature = "4-threads",
        not(any(
            feature = "64-threads",
            feature = "32-threads",
            feature = "16-threads",
            feature = "8-threads",
        ))
    ),
    tokio::main(worker_threads = 4)
)]
#[cfg_attr(
    not(any(
        feature = "64-threads",
        feature = "32-threads",
        feature = "16-threads",
        feature = "8-threads",
        feature = "4-threads",
    )),
    tokio::main(worker_threads = 1)
)]
async fn main() {
    log_init().await;
}

async fn log_init() {
    env_logger::init();
    log::info!(
        "Logger initialized at max level set to {}",
        log::max_level()
    );

    print_thread_feature().await;

    log::info!("001 - Engine Init");
}

async fn print_thread_feature() {
    if cfg!(feature = "64-threads") {
        log::debug!("64-threads: Enabled");
    } else {
        log::debug!("64-threads: Disabled");
    }

    if cfg!(feature = "32-threads") {
        log::debug!("32-threads: Enabled");
    } else {
        log::debug!("32-threads: Disabled");
    }

    if cfg!(feature = "16-threads") {
        log::debug!("16-threads: Enabled");
    } else {
        log::debug!("16-threads: Disabled");
    }

    if cfg!(feature = "8-threads") {
        log::debug!("8-threads: Enabled");
    } else {
        log::debug!("8-threads: Disabled");
    }

    if cfg!(feature = "4-threads") {
        log::debug!("4-threads: Enabled");
    } else {
        log::debug!("4-threads: Disabled");
    }

    if cfg!(not(any(
        feature = "64-threads",
        feature = "32-threads",
        feature = "16-threads",
        feature = "8-threads",
        feature = "4-threads",
    ))) {
        log::debug!("1-threads (Default): Enabled");
    } else {
        log::debug!("1-threads (Default): Disabled");
    }
}
