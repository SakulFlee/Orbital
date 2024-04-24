use hashbrown::HashMap;
use log::{debug, error};
use std::{fs, path::Path};
use wgpu::{ShaderModule, ShaderModuleDescriptor, ShaderSource};

use crate::{error::Error, runtime::Context};

pub enum ShaderType<'a> {
    WGSL(Option<&'a str>),
}

pub struct Shader {
    shader_module: ShaderModule,
    entrypoint_vertex: String,
    entrypoint_fragment: Option<String>,
}

impl Shader {
    pub fn from_file<S: Into<String>>(
        path: S,
        shader_type: ShaderType,
        entrypoint_vertex: String,
        entrypoint_fragment: Option<String>,
        context: &Context,
    ) -> Result<Self, Error> {
        const IMPORT_STATEMENT: &'static str = "#import";

        let mut file_map = HashMap::<String, String>::new();
        let mut files_to_check = Vec::<String>::new();

        files_to_check.push(path.into());
        while !files_to_check.is_empty() {
            // Get the next file to check and try reading it
            let file_path = files_to_check.pop().unwrap();
            let file_content = match fs::read_to_string(&file_path).map_err(|e| Error::IOError(e)) {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed loading shader file at '{}'!", file_path);
                    return Err(e);
                }
            };

            let parent = Path::new(&file_path)
                .parent()
                .expect("No parent on shader folder");

            // If that file is already added to our map, skip
            if file_map.contains_key(&file_path) {
                continue;
            }

            // Process the file line-by-line checking for import statements
            let mut processed_file_lines = Vec::<String>::new();
            for line in file_content.lines() {
                if line.starts_with(IMPORT_STATEMENT) {
                    // If the line starts with the import statement,
                    // extract that file path and put it into the
                    // to-be-checked vector
                    let import_file_path = line.replace(IMPORT_STATEMENT, "").trim().to_string();

                    let import_file = parent.join(import_file_path);
                    files_to_check.push(import_file.to_str().unwrap().into());
                } else {
                    let trimmed_line = line.trim();
                    if trimmed_line.chars().count() > 0 {
                        processed_file_lines.push(format!("{trimmed_line}\n"));
                    }
                }
            }

            // Make result string for this file (without import statements) and
            // add it to the file map.
            let result = processed_file_lines.iter().cloned().collect::<String>();
            file_map.insert(file_path, result);
        }

        // All files should be checked and imported now (or an error was thrown
        // because a file couldn't be found). All left to do now is combining
        // all files, without the import statements, into one big string.
        let final_result = file_map.values().cloned().collect::<String>();

        #[cfg(debug_assertions)]
        debug!("Shader result:\n---\n{final_result}---");

        Ok(Self::from_source_string(
            final_result,
            shader_type,
            entrypoint_vertex,
            entrypoint_fragment,
            context,
        ))
    }

    pub fn from_source_string(
        source_string: String,
        shader_type: ShaderType,
        entrypoint_vertex: String,
        entrypoint_fragment: Option<String>,
        context: &Context,
    ) -> Self {
        let shader_module: ShaderModule = match shader_type {
            ShaderType::WGSL(name) => {
                context
                    .device()
                    .create_shader_module(ShaderModuleDescriptor {
                        label: name,
                        source: ShaderSource::Wgsl(source_string.into()),
                    })
            }
        };

        Self {
            shader_module,
            entrypoint_vertex,
            entrypoint_fragment,
        }
    }

    pub fn module(&self) -> &ShaderModule {
        &self.shader_module
    }

    pub fn entrypoint_vertex(&self) -> &String {
        &self.entrypoint_vertex
    }

    pub fn entrypoint_fragment(&self) -> Option<&String> {
        self.entrypoint_fragment.as_ref()
    }
}
