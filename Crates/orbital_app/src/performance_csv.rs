//! CSV capture for the per-second performance timing window.
//!
//! The recorder lives in the runtime rather than the ECS because `Write` is
//! not `Sync`; keeping the handle here avoids an extra synchronization layer
//! around a file that is written once per second.

use std::io::{self, Write};

use orbital_ecs_bridge::TIMING_STAGES;
use orbital_file_manager::FileManager;

/// Storage-relative path used for performance captures.
pub const PERFORMANCE_CSV_PATH: &str = "Logs/performance.csv";

/// Names of the cache/creation counters appended to each CSV row.
pub const PERFORMANCE_COUNTER_NAMES: [&str; 6] = [
    "shader_modules_delta",
    "pipelines_delta",
    "unique_wgsl",
    "material_hits_delta",
    "material_misses_delta",
    "sky_updates_delta",
];

/// Streams one CSV row per timing window.
pub struct PerformanceCsvRecorder {
    writer: Box<dyn Write + Send>,
}

impl PerformanceCsvRecorder {
    /// Creates the CSV file and writes its header.
    pub fn start(path: &str) -> Result<Self, String> {
        let file_manager = FileManager::global().map_err(|error| error.to_string())?;
        let writer = file_manager
            .open_write(path)
            .map_err(|error| error.to_string())?;
        let mut recorder = Self { writer };
        recorder.write_header().map_err(|error| error.to_string())?;
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
        writeln!(self.writer)
    }

    /// Appends one sample and flushes it immediately.
    ///
    /// Flushing per row keeps the file useful if the process is killed and
    /// costs almost nothing at one row per second.
    pub fn write_sample(
        &mut self,
        elapsed_s: f64,
        fps: u64,
        cdt_ms: f64,
        averages: &[f64],
        counters: [u64; PERFORMANCE_COUNTER_NAMES.len()],
    ) -> io::Result<()> {
        write!(self.writer, "{elapsed_s:.3},{fps},{cdt_ms:.3}")?;
        for average in averages {
            write!(self.writer, ",{average:.3}")?;
        }
        for counter in counters {
            write!(self.writer, ",{counter}")?;
        }
        writeln!(self.writer)?;
        self.writer.flush()
    }

    /// Flushes and closes the file.
    pub fn finish(mut self) {
        let _ = self.writer.flush();
    }
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
            )
            .unwrap();

        let text = String::from_utf8(output.lock().unwrap().clone()).unwrap();
        let mut lines = text.lines();
        let header = lines.next().unwrap();
        let row = lines.next().unwrap();

        assert_eq!(
            header.split(',').count(),
            3 + TIMING_STAGES.len() + PERFORMANCE_COUNTER_NAMES.len()
        );
        assert_eq!(header.split(',').count(), row.split(',').count());
        assert!(row.starts_with("1.500,60,16.700"));
        assert!(row.ends_with("1,2,3,4,5,6"));
    }
}
