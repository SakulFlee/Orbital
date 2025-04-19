use std::time::SystemTime;

use log::*;

#[cfg(not(test))]
use std::{fs, path::Path};

#[cfg(target_os = "android")]
pub fn init() {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Debug),
    );
}

#[cfg(not(target_os = "android"))]
pub fn init() {
    let default_log_level = if cfg!(debug_assertions) || cfg!(test) {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    #[allow(unused_mut)]
    let mut logger_chain_dispatch = fern::Dispatch::new()
        // Default level to accept
        .level(default_log_level)
        // WGPU Overwrite
        .level_for("wgpu_core", LevelFilter::Warn)
        .level_for("wgpu_hal", LevelFilter::Warn)
        .level_for("naga", LevelFilter::Warn)
        // Write to StdOut
        .chain(std::io::stdout());

    #[cfg(not(test))]
    {
        const START: u32 = 0;
        const END: u32 = 4;

        for i in END..=START {
            let log_file = format!("game-{}.log", i);
            let path = Path::new(&log_file);

            if path.exists() {
                if i == END {
                    fs::remove_file(&path).expect("failed removing last index log file");
                } else {
                    let next_log_file = format!("game-{}.log", i + 1);

                    fs::rename(path, next_log_file)
                        .expect("failed renaming log file to next index");
                }
            }
        }

        let current_log_file = format!("game-{}.log", START);
        logger_chain_dispatch = logger_chain_dispatch
            .chain(fern::log_file(current_log_file).expect("failed building file log"));
    }

    if let Err(e) = fern::Dispatch::new()
        // Setup formation
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .chain(logger_chain_dispatch)
        // Apply as global logger!
        .apply()
    {
        error!(
            "Failure creating logger. This is commonly due to a logger already being initialized beforehand. Error: {}",
            e
        );
    }

    info!("Logger initialized at max level set to {}", max_level());
}
