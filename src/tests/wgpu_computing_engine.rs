use wgpu_engine::WGPUComputingEngine;

#[test]
fn init_test() {
    let computing_engine = WGPUComputingEngine::new();

    assert!(computing_engine.is_ok());
}

#[test]
fn init_with_score() {
    let computing_engine = WGPUComputingEngine::new_with_score_function(|_| 100);
    assert!(computing_engine.is_ok())
}
