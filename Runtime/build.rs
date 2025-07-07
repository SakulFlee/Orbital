use std::path::PathBuf;
use std::process::Command;
use std::str;

const MODEL_FILES_DIR: &str = "../Examples/SharedAssets/ModelFiles";
const MODEL_SCRIPT_GLTF_EXPORT: &str =
    "../Examples/SharedAssets/ModelScripts/blender_gltf_export.py";
const MODEL_SCRIPT_PBR_SPHERE_GEN: &str = "../Examples/SharedAssets/ModelScripts/pbr_sphere_gen.py";
const MODELS_DIR: &str = "../Examples/SharedAssets/Models";

fn main() {
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

    let output = Command::new("blender")
        .arg("--background")
        .arg("--python")
        .arg(&script_path) // Use &script_path as Command::arg takes AsRef<OsStr>
        .current_dir(&output_path) // Use &output_path
        .output() // Use output() to capture stdout and stderr
        .expect("Failed to run Blender command!");

    let stdout = str::from_utf8(&output.stdout).expect("Failed to convert stdout to string");
    let stderr = str::from_utf8(&output.stderr).expect("Failed to convert stderr to string");

    if !stdout.is_empty() {
        println!("cargo:warning=Blender PBR Spheres stdout:\n{stdout}");
    }
    if !stderr.is_empty() {
        println!("cargo:warning=Blender PBR Spheres stderr:\n{stderr}");
    }

    if !output.status.success() || !stdout.contains("### FINISHED ###") {
        panic!(
            "Failed generating PBR Spheres and export to glTF! Blender exited with code: {}\nStdout: {}\nStderr: {}",
            output.status, stdout, stderr
        );
    } else {
        println!("cargo:warn=Exported PBR Spheres successfully!");
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

    let model_files_path_ =
        std::fs::canonicalize(MODEL_FILES_DIR).expect("Failed to canonicalize script path!");
    let model_files_path = model_files_path_
        .to_str()
        .expect("Conversion from PathBuf to String failed!");

    let output_path =
        std::fs::canonicalize(MODELS_DIR).expect("Failed to canonicalize models output folder!");

    for entry in glob::glob(&format!("{model_files_path}/*.blend")).unwrap() {
        let path = entry.unwrap();

        blender_convert_to_gltf(path.to_str().unwrap(), &script_path, &output_path);
    }
}

fn blender_convert_to_gltf(filepath: &str, script_path: &PathBuf, output_path: &PathBuf) {
    println!("cargo::rerun-if-changed={filepath}");

    let output = Command::new("blender")
        .arg("--background")
        .arg(filepath)
        .arg("--python")
        .arg(script_path)
        .current_dir(output_path)
        .output()
        .expect("Failed to run Blender command!");

    let stdout = str::from_utf8(&output.stdout).expect("Failed to convert stdout to string");
    let stderr = str::from_utf8(&output.stderr).expect("Failed to convert stderr to string");

    if !stdout.is_empty() {
        println!("cargo:warning=Blender stdout for '{filepath}':\n{stdout}");
    }
    if !stderr.is_empty() {
        println!("cargo:warning=Blender stderr for '{filepath}':\n{stderr}");
    }

    if !output.status.success() || !stdout.contains("### FINISHED ###") {
        panic!(
            "Failed converting Blender file '{}' to glTF! Blender exited with code: {}\nStdout: {}\nStderr: {}",
            filepath, output.status, stdout, stderr
        );
    } else {
        println!("cargo:warn=Exported Blender file '{filepath}' successfully!");
    }
}
