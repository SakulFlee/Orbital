mod error;
pub use error::ShaderPreprocessorError;

#[cfg(test)]
mod tests;

use std::collections::HashMap;

use orbital_file_manager::FileManager;
use log::debug;

pub struct ShaderPreprocessor {
    known_imports: HashMap<String, String>,
}

impl ShaderPreprocessor {
    pub const IMPORT_EXPRESSION_START: &'static str = "#import <";

    pub const IMPORT_EXPRESSION_END: &'static str = ">";

    /// Asset-relative folder (no `Assets/` prefix) scanned for importable WGSL
    /// shader snippets. Resolved through the
    /// [`FileManager`](orbital_file_manager::FileManager).
    pub const SHADER_LIB_IMPORT_FOLDER_PATH: &'static str = "Shaders";

    pub fn new_with_defaults() -> Result<Self, ShaderPreprocessorError> {
        let mut s = Self {
            known_imports: HashMap::new(),
        };

        s.import_folder(Self::SHADER_LIB_IMPORT_FOLDER_PATH)?;

        Ok(s)
    }

    pub fn new_empty() -> Self {
        Self {
            known_imports: HashMap::new(),
        }
    }

    pub fn add_import<S0: Into<String>, S1: Into<String>>(&mut self, directive: S0, content: S1) {
        self.known_imports.insert(directive.into(), content.into());
    }

    pub fn add_file_import<S0: Into<String>, S1: Into<String>>(
        &mut self,
        directive: Option<S0>,
        path: S1,
    ) -> Result<(), ShaderPreprocessorError> {
        let path = path.into();
        let directive: String = directive.map(Into::into).unwrap_or(
            path.rsplit('/')
                .next()
                .and_then(|name| name.split('.').next())
                .unwrap_or(&path)
                .to_string(),
        );

        let file_manager = FileManager::global().map_err(ShaderPreprocessorError::Fs)?;
        let content = file_manager
            .read_asset_to_string(&path)
            .map_err(ShaderPreprocessorError::Fs)?;

        self.add_import(directive, content);

        Ok(())
    }

    pub fn import_folder<S: Into<String>>(
        &mut self,
        path: S,
    ) -> Result<(), ShaderPreprocessorError> {
        let path = path.into();

        let file_manager = FileManager::global().map_err(ShaderPreprocessorError::Fs)?;
        let files = file_manager
            .list_asset_dir(&path)
            .map_err(ShaderPreprocessorError::Fs)?;

        for file in files {
            let directive = file
                .strip_suffix(".wgsl")
                .unwrap_or(&file)
                .replace('\\', "/")
                .to_lowercase();

            let full_path = if path.is_empty() {
                file.clone()
            } else {
                format!("{path}/{file}")
            };

            let content = file_manager
                .read_asset_to_string(&full_path)
                .map_err(ShaderPreprocessorError::Fs)?;
            debug!("Imported content for directive '{directive}' ({full_path}):\n{content}\n");

            self.add_import(directive, content);
        }

        Ok(())
    }

    pub fn parse_shader<S: Into<String>>(
        &self,
        source: S,
    ) -> Result<String, ShaderPreprocessorError> {
        let source = source.into();
        let imported_directives = Vec::new();
        self.parse_shader_(source, imported_directives)
    }

    fn parse_shader_(
        &self,
        source: String,
        imported_directives: Vec<&str>,
    ) -> Result<String, ShaderPreprocessorError> {
        let mut output = String::new();
        let mut imported_directives = imported_directives;
        let mut import_found = false;

        for line in source.lines() {
            if let Some(start) = line.find(Self::IMPORT_EXPRESSION_START)
                && let Some(end) = line.find(Self::IMPORT_EXPRESSION_END)
            {
                let directive = &line[start + Self::IMPORT_EXPRESSION_START.len()..end];
                if imported_directives.contains(&directive) {
                    continue;
                } else {
                    imported_directives.push(directive);
                    import_found = true;
                }

                let import = self.known_imports.get(directive).ok_or(
                    ShaderPreprocessorError::UnknownDirective {
                        directive: directive.to_string(),
                    },
                )?;

                if output.is_empty() {
                    output = import.clone();
                } else {
                    output = format!("{output}\n{import}");
                }

                continue;
            }

            if output.is_empty() {
                output = line.to_string();
            } else {
                output = format!("{output}\n{line}");
            }
        }

        if import_found {
            self.parse_shader_(output, imported_directives)
        } else {
            Ok(output)
        }
    }
}
