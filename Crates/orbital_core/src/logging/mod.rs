pub use log::*;
use std::sync::Once;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use std::{fs, path::Path, time::SystemTime};

use std::{
    collections::VecDeque,
    sync::{LazyLock, RwLock},
};

/// One captured log record.
///
/// Kept as structured data rather than a preformatted string so a consumer can
/// filter on [`level`](LogLine::level) or [`target`](LogLine::target) without
/// re-parsing text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogLine {
    /// Severity of the record.
    pub level: Level,
    /// Module path the record came from, e.g. `"orbital_app::module_runtime"`.
    pub target: String,
    /// The formatted message body.
    pub message: String,
}

/// How many lines [`log_buffer`] retains before dropping the oldest.
const LOG_BUFFER_CAPACITY: usize = 1024;

/// The single shared ring of recent log records.
static LOG_BUFFER: LazyLock<RwLock<VecDeque<LogLine>>> =
    LazyLock::new(|| RwLock::new(VecDeque::with_capacity(LOG_BUFFER_CAPACITY)));

/// A snapshot of the in-memory ring of recent log records, oldest first.
///
/// Fed by the `format` closure of each platform's `fern` dispatch, or by the
/// Android capture logger, so a debug overlay can show an in-game console.
/// Returns an empty buffer before [`init`] has run.
///
/// Clones the whole buffer. Prefer [`with_log_buffer`] when polling every
/// frame; this is here for the occasional one-off read.
pub fn log_buffer() -> Vec<LogLine> {
    LOG_BUFFER
        .read()
        .map(|buffer| buffer.iter().cloned().collect())
        .unwrap_or_default()
}

/// Borrows the log ring without copying it, oldest first.
///
/// Meant for a per-frame consumer such as an in-game console: it avoids
/// cloning every buffered line each time the UI redraws.
///
/// The callback must not log — the ring is read-locked while it runs.
pub fn with_log_buffer<T>(
    f: impl FnOnce(std::collections::vec_deque::Iter<'_, LogLine>) -> T,
) -> T {
    // A poisoned lock still holds readable data, so recover it rather than
    // handing the consumer nothing.
    let buffer = LOG_BUFFER
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    f(buffer.iter())
}

/// Appends a record to the global ring buffer, dropping the oldest line when
/// the buffer is full.
///
/// A poisoned lock means some consumer panicked while holding it; losing a log
/// line is preferable to aborting the logging call, so the write is skipped.
fn capture(line: LogLine) {
    let Ok(mut buffer) = LOG_BUFFER.write() else {
        return;
    };

    if buffer.len() == LOG_BUFFER_CAPACITY {
        buffer.pop_front();
    }

    buffer.push_back(line);
}

/// Captures a record into [`LOG_BUFFER`].
///
/// Called from each dispatch's `format` closure, after the dispatch's level
/// filter, so the ring only holds records that also reach the platform sink.
/// Android's capture logger calls it before forwarding to `android_logger`.
///
/// The `fern` dispatch cannot use a chained `log::Log` sink instead:
/// `FormatCallback::finish` rebuilds the record with the *decorated* message,
/// so a chained sink would only ever see the formatted line, not the raw one.
fn capture_record(record: &log::Record<'_>) {
    capture(LogLine {
        level: record.level(),
        target: record.target().to_string(),
        message: record.args().to_string(),
    });
}

/// Target-specific level overrides shared by every platform.
///
/// The shader/GPU stacks are extremely chatty at `Debug`; their per-frame
/// diagnostics are only useful when actively debugging them.
const TARGET_LEVEL_FILTERS: &[(&str, LevelFilter)] = &[
    ("wgpu_core", LevelFilter::Warn),
    ("wgpu_hal", LevelFilter::Warn),
    ("naga", LevelFilter::Warn),
];

/// The level filter used by every platform's `init` function.
fn default_level_filter() -> LevelFilter {
    if cfg!(debug_assertions) {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    }
}

/// Builds a `fern` dispatch with the shared default level and target filters.
///
/// Callers add their platform-specific formatter and sinks.
fn target_filtered_dispatch(level: LevelFilter) -> fern::Dispatch {
    let mut dispatch = fern::Dispatch::new().level(level);

    for (target, target_level) in TARGET_LEVEL_FILTERS {
        dispatch = dispatch.level_for(*target, *target_level);
    }

    dispatch
}

#[cfg(any(target_os = "android", test))]
fn filter_level_name(level: LevelFilter) -> &'static str {
    match level {
        LevelFilter::Off => "off",
        LevelFilter::Error => "error",
        LevelFilter::Warn => "warn",
        LevelFilter::Info => "info",
        LevelFilter::Debug => "debug",
        LevelFilter::Trace => "trace",
    }
}

/// The `env_filter` directive string used by `android_logger`.
#[cfg(any(target_os = "android", test))]
fn filter_directives(level: LevelFilter) -> String {
    let mut directives = filter_level_name(level).to_string();

    for (target, target_level) in TARGET_LEVEL_FILTERS {
        directives.push_str(&format!(",{target}={}", filter_level_name(*target_level)));
    }

    directives
}

#[cfg(target_os = "android")]
struct CaptureAndroidLogger(android_logger::AndroidLogger);

#[cfg(target_os = "android")]
impl Log for CaptureAndroidLogger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        self.0.enabled(metadata)
    }

    fn log(&self, record: &log::Record<'_>) {
        if self.0.enabled(record.metadata()) {
            capture_record(record);
            self.0.log(record);
        }
    }

    fn flush(&self) {
        self.0.flush();
    }
}

#[cfg(target_os = "android")]
pub fn init() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let level = default_level_filter();
        let config = android_logger::Config::default()
            .with_tag("rust_std_out")
            .with_max_level(level)
            .with_filter(
                android_logger::FilterBuilder::new()
                    .parse(&filter_directives(level))
                    .build(),
            );

        match log::set_boxed_logger(Box::new(CaptureAndroidLogger(
            android_logger::AndroidLogger::new(config),
        ))) {
            Ok(()) => log::set_max_level(level),
            Err(e) => error!(
                "Failure creating logger. This is commonly due to a logger already being initialized beforehand. Error: {e}"
            ),
        }

        info!("Logger initialized at max level set to {}", max_level());
    });
}

/// iOS logging: stdout only (no file rotation — the sandbox restricts
/// filesystem access to the bundle and Documents directories).
#[cfg(target_os = "ios")]
pub fn init() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let default_log_level = default_level_filter();

        // The level filter lives on the outer dispatch on purpose: fern runs
        // `format` only after `shallow_enabled` passes, so the buffer never
        // sees a record the log itself would have dropped.
        let dispatch = target_filtered_dispatch(default_log_level)
            .format(|out, message, record| {
                capture_record(record);
                out.finish(format_args!(
                    "[{} {}] {}",
                    record.level(),
                    record.target(),
                    message
                ))
            })
            .chain(std::io::stdout())
            .apply();

        if let Err(e) = dispatch {
            error!(
                "Failure creating logger. This is commonly due to a logger already being initialized beforehand. Error: {e}"
            );
        }

        info!("Logger initialized at max level set to {}", max_level());
    });
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub fn init() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let default_log_level = default_level_filter();

        const START: u32 = 0;
        const END: u32 = 4;

        for i in (START..=END).rev() {
            let log_file = format!("game-{i}.log");
            let path = Path::new(&log_file);

            if path.exists() {
                if i == END {
                    fs::remove_file(path).expect("failed removing last index log file");
                } else {
                    let next_log_file = format!("game-{}.log", i + 1);

                    fs::rename(path, next_log_file)
                        .expect("failed renaming log file to next index");
                }
            }
        }

        // See the iOS `init` for why the level filter is on the outer dispatch.
        let dispatch = target_filtered_dispatch(default_log_level)
            .format(|out, message, record| {
                capture_record(record);
                out.finish(format_args!(
                    "[{} {} {}] {}",
                    humantime::format_rfc3339_seconds(SystemTime::now()),
                    record.level(),
                    record.target(),
                    message
                ))
            })
            .chain(std::io::stdout())
            .chain(
                fern::log_file(format!("game-{START}.log"))
                    .expect("failed building file log"),
            )
            .apply();

        if let Err(e) = dispatch {
            error!(
                "Failure creating logger. This is commonly due to a logger already being initialized beforehand. Error: {e}"
            );
        }

        info!("Logger initialized at max level set to {}", max_level());
    });
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub fn test_init() {
    let dispatch = target_filtered_dispatch(LevelFilter::Debug)
        .format(|out, message, record| {
            capture_record(record);
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .chain(std::io::stdout())
        .apply();

    if let Err(e) = dispatch {
        error!(
            "Failure creating logger. This is commonly due to a logger already being initialized beforehand. Error: {e}"
        );
    }

    info!("Logger initialized at max level set to {}", max_level());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One test for the whole buffer: the ring is process-global, so two tests
    /// touching it would race and each would see the other's lines.
    #[test]
    fn log_buffer_captures_and_is_bounded() {
        test_init();

        let before = log_buffer().len();
        info!(target: "orbital_core::tests", "buffer capture probe");

        let buffer = log_buffer();
        assert!(
            buffer.len() > before,
            "TeeLogger should have captured the record"
        );

        let line = buffer
            .iter()
            .find(|line| line.message.contains("buffer capture probe"))
            .expect("Captured line not found");

        // The buffer holds the *raw* record, not the decorated output line.
        assert_eq!(line.level, Level::Info);
        assert_eq!(line.target, "orbital_core::tests");
        assert_eq!(line.message, "buffer capture probe");

        // Overflow by one; the ring must cap and drop the oldest.
        const SENTINEL_OLDEST: &str = "overflow probe oldest";
        capture(LogLine {
            level: Level::Warn,
            target: "orbital_core::tests".to_string(),
            message: SENTINEL_OLDEST.to_string(),
        });
        for index in 0..LOG_BUFFER_CAPACITY {
            capture(LogLine {
                level: Level::Warn,
                target: "orbital_core::tests".to_string(),
                message: format!("overflow probe {index}"),
            });
        }

        let buffer = log_buffer();
        assert_eq!(buffer.len(), LOG_BUFFER_CAPACITY);
        assert!(
            !buffer.iter().any(|line| line.message == SENTINEL_OLDEST),
            "oldest line should have been evicted"
        );
        assert_eq!(
            buffer.last().map(|line| line.message.as_str()),
            Some(format!("overflow probe {}", LOG_BUFFER_CAPACITY - 1).as_str()),
            "newest line should be retained"
        );

        // The borrowing accessor sees the same lines in the same order.
        let borrowed =
            with_log_buffer(|lines| lines.map(|line| line.message.clone()).collect::<Vec<_>>());
        assert_eq!(
            borrowed,
            buffer
                .iter()
                .map(|line| line.message.clone())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn filter_directives_match_the_shared_target_policy() {
        let directives = filter_directives(LevelFilter::Debug);

        assert!(directives.starts_with("debug"));
        assert!(directives.contains(",naga=warn"));
        assert!(directives.contains(",wgpu_core=warn"));
        assert!(directives.contains(",wgpu_hal=warn"));
    }
}
