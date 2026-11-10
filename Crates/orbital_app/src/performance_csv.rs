//! CSV capture for the per-second performance timing window.
//!
//! The recorder lives in the runtime rather than the ECS because `Write` is
//! not `Sync`; keeping the handle here avoids an extra synchronization layer
//! around a file that is written once per second.

use std::io::{self, Write};

use orbital_ecs_bridge::TIMING_STAGES;
use orbital_file_manager::FileManager;
use serde_json::Value;

/// Storage-relative path used for performance captures.
pub const PERFORMANCE_CSV_PATH: &str = "Logs/performance.csv";

/// Sidecar written next to the CSV with the capture environment.
pub const PERFORMANCE_METADATA_PATH: &str = "Logs/performance_meta.json";

/// Names of the cache/creation counters appended to each CSV row.
pub const PERFORMANCE_COUNTER_NAMES: [&str; 6] = [
    "shader_modules_delta",
    "pipelines_delta",
    "unique_wgsl",
    "material_hits_delta",
    "material_misses_delta",
    "sky_updates_delta",
];

/// Cumulative surface acquisition results since the capture started.
#[derive(Debug, Clone, Copy, Default)]
pub struct SurfaceStatusCounters {
    pub success: u64,
    pub suboptimal: u64,
    pub timeout: u64,
    pub occluded: u64,
    pub outdated: u64,
    pub lost: u64,
    pub validation: u64,
}

const SURFACE_STATUS_NAMES: [&str; 7] = [
    "surface_success_total",
    "surface_suboptimal_total",
    "surface_timeout_total",
    "surface_occluded_total",
    "surface_outdated_total",
    "surface_lost_total",
    "surface_validation_total",
];

/// Streams one CSV row per timing window.
pub struct PerformanceCsvRecorder {
    writer: Box<dyn Write + Send>,
}

impl PerformanceCsvRecorder {
    /// Creates the CSV file, writes its header, and writes the metadata
    /// sidecar when one is available.
    pub fn start(path: &str, metadata: Option<&Value>) -> Result<Self, String> {
        let file_manager = FileManager::global().map_err(|error| error.to_string())?;
        let writer = file_manager
            .open_write(path)
            .map_err(|error| error.to_string())?;
        let mut recorder = Self { writer };
        recorder.write_header().map_err(|error| error.to_string())?;

        if let Some(metadata) = metadata
            && let Err(error) = write_metadata(metadata)
        {
            log::warn!("performance metadata: failed to write sidecar: {error}");
        }

        Ok(recorder)
    }

    fn write_header(&mut self) -> io::Result<()> {
        write!(self.writer, "elapsed_s,fps,cdt_ms")?;
        for name in TIMING_STAGES {
            write!(self.writer, ",{}", name.replace(' ', "_"))?;
        }
        for name in PERFORMANCE_COUNTER_NAMES {
            write!(self.writer, ",{name}")?;
        }
        for name in SURFACE_STATUS_NAMES {
            write!(self.writer, ",{name}")?;
        }
        write!(self.writer, ",visible_overlays,visible_overlay_names")?;
        writeln!(self.writer)
    }

    /// Appends one sample and flushes it immediately.
    ///
    /// Flushing per row keeps the file useful if the process is killed and
    /// costs almost nothing at one row per second.
    #[allow(clippy::too_many_arguments)]
    pub fn write_sample(
        &mut self,
        elapsed_s: f64,
        fps: u64,
        cdt_ms: f64,
        averages: &[f64],
        counters: [u64; PERFORMANCE_COUNTER_NAMES.len()],
        surface: SurfaceStatusCounters,
        visible_overlays: u64,
        visible_overlay_names: &str,
    ) -> io::Result<()> {
        write!(self.writer, "{elapsed_s:.3},{fps},{cdt_ms:.3}")?;
        for average in averages {
            write!(self.writer, ",{average:.3}")?;
        }
        for counter in counters {
            write!(self.writer, ",{counter}")?;
        }
        for value in [
            surface.success,
            surface.suboptimal,
            surface.timeout,
            surface.occluded,
            surface.outdated,
            surface.lost,
            surface.validation,
        ] {
            write!(self.writer, ",{value}")?;
        }

        // Panel titles are fixed and do not contain commas, but keep the CSV
        // valid if a caller ever supplies a free-form overlay name.
        let names = visible_overlay_names.replace([',', '\n', '\r'], ";");
        write!(self.writer, ",{visible_overlays},{names}")?;
        writeln!(self.writer)?;
        self.writer.flush()
    }

    /// Flushes and closes the file.
    pub fn finish(mut self) {
        let _ = self.writer.flush();
    }
}

fn write_metadata(metadata: &Value) -> Result<(), String> {
    let file_manager = FileManager::global().map_err(|error| error.to_string())?;
    let mut writer = file_manager
        .open_write(PERFORMANCE_METADATA_PATH)
        .map_err(|error| error.to_string())?;
    let bytes = serde_json::to_vec_pretty(metadata).map_err(|error| error.to_string())?;
    writer
        .write_all(&bytes)
        .map_err(|error| error.to_string())?;
    writer.flush().map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use std::io::{self, Write};
    use std::sync::{Arc, Mutex};

    use super::*;

    struct SharedWriter(Arc<Mutex<Vec<u8>>>);

    impl Write for SharedWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn sample_has_one_column_per_stage_and_counter() {
        let output = Arc::new(Mutex::new(Vec::new()));
        let mut recorder = PerformanceCsvRecorder {
            writer: Box::new(SharedWriter(Arc::clone(&output))),
        };

        recorder.write_header().unwrap();
        recorder
            .write_sample(
                1.5,
                60,
                16.7,
                &vec![0.0; TIMING_STAGES.len()],
                [1, 2, 3, 4, 5, 6],
                SurfaceStatusCounters {
                    success: 7,
                    ..Default::default()
                },
                2,
                "Performance|Rendering",
            )
            .unwrap();

        let text = String::from_utf8(output.lock().unwrap().clone()).unwrap();
        let mut lines = text.lines();
        let header = lines.next().unwrap();
        let row = lines.next().unwrap();

        assert_eq!(
            header.split(',').count(),
            3 + TIMING_STAGES.len() + PERFORMANCE_COUNTER_NAMES.len() + 9
        );
        assert_eq!(header.split(',').count(), row.split(',').count());
        assert!(row.starts_with("1.500,60,16.700"));
        assert!(row.ends_with("1,2,3,4,5,6,7,0,0,0,0,0,0,2,Performance|Rendering"));
    }
}
