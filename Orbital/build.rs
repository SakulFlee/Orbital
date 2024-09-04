use std::process::Command;

const MODELS_DIR: &str = "../Assets/Models/";

fn main() {
    blender_pbr_spheres();
    blender_model_files();
}

fn blender_pbr_spheres() {
    println!("cargo::rerun-if-changed={}/pbr_sphere_gen.py", MODELS_DIR);

    let mut handle = Command::new("blender")
        .arg("--background")
        .arg("--python")
        .arg("pbr_sphere_gen.py")
        .current_dir(MODELS_DIR)
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

fn blender_model_files() {
    for entry in glob::glob(&format!("{}/*.blend", MODELS_DIR)).unwrap() {
        match entry {
            Ok(path) => blender_convert_to_gtlf(
                path.into_os_string()
                    .into_string()
                    .expect("Path conversion failure")
                    .as_str(),
            ),
            Err(e) => panic!("Invalid glob node: {:?}", e),
        }
    }
}

fn blender_convert_to_gtlf(filepath: &str) {
    println!("cargo::rerun-if-changed={}", filepath);

    let mut handle = Command::new("blender")
        .arg("--background")
        .arg(filepath)
        .arg("--python")
        .arg("../Assets/Models/blender_gltf_export.py")
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
