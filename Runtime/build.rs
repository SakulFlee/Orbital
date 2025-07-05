use std::path::PathBuf;
use std::process::Command;

const MODEL_FILES_DIR: &str = "../Examples/SharedAssets/ModelFiles";
const MODEL_SCRIPT_GLTF_EXPORT: &str =
    "../Examples/SharedAssets/ModelScripts/blender_gltf_export.py";
const MODEL_SCRIPT_PBR_SPHERE_GEN: &str =
    "../Examples/SharedAssets/ModelScripts/pbr_sphere_gen.py";
const MODELS_DIR: &str = "../Examples/SharedAssets/Models";

fn main() {
    // Only run script in CIs if `RUN_BUILD_SCRIPT` is explicitly set!
    let run_script = std::env::var("CI").is_err_and(|_| std::env::var("RUN_BUILD_SCRIPT").is_ok());
    if !run_script {
        println!("cargo::warning=Skipping build.rs as in CI and RUN_BUILD_SCRIPT isn't set!");
        return;
    }

    blender_pbr_spheres();
    blender_model_files();
}

fn blender_pbr_spheres() {
    let script_path = std::fs::canonicalize(MODEL_SCRIPT_PBR_SPHERE_GEN)
        .expect("Failed to canonicalize script path!");
    println!(
        "cargo::rerun-if-changed={}",
        script_path
            .to_str()
            .expect("Failed converting script path to string!")
    );

    let output_path =
        std::fs::canonicalize(MODELS_DIR).expect("Failed to canonicalize models output folder!");

    let mut handle = Command::new("blender")
        .arg("--background")
        .arg("--python")
        .arg(script_path)
        .current_dir(output_path)
        .spawn()
        .expect("Failed to run Blender command!");

    let exit_code = handle.wait().expect("Failed to wait for process to finish");
    if !exit_code.success() {
        panic!(
            "Failed generating PBR Spheres and export to glTF! Blender exited with code: {}",
            exit_code
        );
    }
}

fn blender_model_files() {
    let script_path = std::fs::canonicalize(MODEL_SCRIPT_GLTF_EXPORT)
        .expect("Failed to canonicalize script path!");
    println!(
        "cargo::rerun-if-changed={}",
        script_path
            .to_str()
            .expect("Failed converting script path to string!")
    );

    println!("{:?}", MODEL_FILES_DIR);
    let model_files_path_ =
        std::fs::canonicalize(MODEL_FILES_DIR).expect("Failed to canonicalize script path!");
    let model_files_path = model_files_path_
        .to_str()
        .expect("Conversion from PathBuf to String failed!");

    let output_path =
        std::fs::canonicalize(MODELS_DIR).expect("Failed to canonicalize models output folder!");

    for entry in glob::glob(&format!("{}/*.blend", model_files_path)).unwrap() {
        let path = entry.unwrap();

        blender_convert_to_gltf(
            path.into_os_string().into_string().unwrap().as_str(),
            &script_path,
            &output_path,
        );
    }
}

fn blender_convert_to_gltf(filepath: &str, script_path: &PathBuf, output_path: &PathBuf) {
    println!("cargo::rerun-if-changed={}", filepath);

    let mut handle = Command::new("blender")
        .arg("--background")
        .arg(filepath)
        .arg("--python")
        .arg(script_path)
        .current_dir(output_path)
        .spawn()
        .expect("Failed to run Blender command!");

    let exit_code = handle.wait().expect("Failed to wait for process to finish");
    if !exit_code.success() {
        panic!(
            "Failed converting Blender file to glTF! Blender exited with code: {}",
            exit_code
        );
    }
}
