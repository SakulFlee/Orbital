//! Pass-aware render-pipeline cache.
//!
//! Pipelines are keyed by program identity, pass kind, color format and depth
//! format. Programs are tracked weakly so pipelines for dropped materials are
//! pruned instead of leaking for the lifetime of the device.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Weak};

use wgpu::{Device, RenderPipeline, TextureFormat};

use crate::ShaderProgram;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PassKind {
    Forward,
    Gbuffer,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct PipelineKey {
    program: u64,
    pass: PassKind,
    color_formats: Vec<TextureFormat>,
    depth_format: Option<TextureFormat>,
}

#[derive(Debug, Default)]
pub struct PipelineCache {
    programs: HashMap<u64, Weak<ShaderProgram>>,
    pipelines: HashMap<PipelineKey, Arc<RenderPipeline>>,
}

impl PipelineCache {
    pub fn forward_pipeline(
        &mut self,
        program: &Arc<ShaderProgram>,
        color_format: TextureFormat,
        device: &Device,
    ) -> Arc<RenderPipeline> {
        self.programs.insert(program.id(), Arc::downgrade(program));
        self.prune();

        let key = PipelineKey {
            program: program.id(),
            pass: PassKind::Forward,
            color_formats: vec![color_format],
            depth_format: program.depth_format(),
        };

        self.pipelines
            .entry(key)
            .or_insert_with(|| {
                crate::PIPELINES_CREATED.fetch_add(1, Ordering::Relaxed);
                Arc::new(program.create_forward_pipeline(color_format, device))
            })
            .clone()
    }

    pub fn gbuffer_pipeline(
        &mut self,
        program: &Arc<ShaderProgram>,
        color_formats: &[TextureFormat],
        device: &Device,
    ) -> Option<Arc<RenderPipeline>> {
        program.entrypoint_fragment_gbuffer()?;

        self.programs.insert(program.id(), Arc::downgrade(program));
        self.prune();

        let key = PipelineKey {
            program: program.id(),
            pass: PassKind::Gbuffer,
            color_formats: color_formats.to_vec(),
            depth_format: program.depth_format(),
        };

        Some(
            self.pipelines
                .entry(key)
                .or_insert_with(|| {
                    crate::PIPELINES_CREATED.fetch_add(1, Ordering::Relaxed);
                    Arc::new(
                        program
                            .create_gbuffer_pipeline(color_formats, device)
                            .expect("G-buffer entrypoint checked above"),
                    )
                })
                .clone(),
        )
    }

    /// Drop programs that no material references and their pipelines.
    pub fn prune(&mut self) {
        self.programs
            .retain(|_, program| program.strong_count() > 0);

        let alive: HashSet<u64> = self.programs.keys().copied().collect();
        self.pipelines.retain(|key, _| alive.contains(&key.program));
    }

    pub fn program_count(&self) -> usize {
        self.programs.len()
    }

    pub fn pipeline_count(&self) -> usize {
        self.pipelines.len()
    }
}
