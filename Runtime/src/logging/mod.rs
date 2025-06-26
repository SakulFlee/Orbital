pub use log::*;
use std::sync::Once;
use std::{fs, path::Path, time::SystemTime};

#[cfg(target_os = "android")]
pub fn init() {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Debug),
    );
}

#[cfg(not(target_os = "android"))]
pub fn init() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let default_log_level = if cfg!(debug_assertions) {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        };

        const START: u32 = 0;
        const END: u32 = 4;

        for i in (START..=END).rev() {
            let log_file = format!("game-{}.log", i);
            let path = Path::new(&log_file);

            if path.exists() {
                if i == END {
                    fs::remove_file(path).expect("failed removing last index log file");
                } else {
                    let next_log_file = format!("game-{}.log", i + 1);

                    fs::rename(path, next_log_file).expect("failed renaming log file to next index");
                }
            }
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
            .chain(
                fern::Dispatch::new()
                    // Default level to accept
                    .level(default_log_level)
                    // WGPU Overwrite
                    .level_for("wgpu_core", LevelFilter::Warn)
                    .level_for("wgpu_hal", LevelFilter::Warn)
                    .level_for("naga", LevelFilter::Warn)
                    // Write to StdOut
                    .chain(std::io::stdout())
                    .chain(
                        fern::log_file(format!("game-{}.log", START))
                            .expect("failed building file log"),
                    ),
            )
            // Apply as global logger!
            .apply()
        {
            error!(
            "Failure creating logger. This is commonly due to a logger already being initialized beforehand. Error: {}",
            e
        );
        }

        info!("Logger initialized at max level set to {}", max_level());
    });
}

#[cfg(not(target_os = "android"))]
pub fn test_init() {
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
        .chain(
            fern::Dispatch::new()
                // Default level to accept
                .level(LevelFilter::Debug)
                // WGPU Overwrite
                .level_for("wgpu_core", LevelFilter::Warn)
                .level_for("wgpu_hal", LevelFilter::Warn)
                .level_for("naga", LevelFilter::Warn)
                // Write to StdOut
                .chain(std::io::stdout()),
        )
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
