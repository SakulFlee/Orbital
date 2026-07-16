mod error;
pub use error::ShaderPreprocessorError;

#[cfg(test)]
mod tests;

use std::{
    collections::HashMap,
    fs::{canonicalize, read_to_string},
    path::PathBuf,
};

use glob::glob;
use log::debug;

pub struct ShaderPreprocessor {
    known_imports: HashMap<String, String>,
}

impl ShaderPreprocessor {
    pub const IMPORT_EXPRESSION_START: &'static str = "#import <";

    pub const IMPORT_EXPRESSION_END: &'static str = ">";

    #[cfg(debug_assertions)]
    pub const SHADER_LIB_IMPORT_FOLDER_PATH_DEBUG_BUILD: &'static str = "../../Assets/Shaders";

    #[cfg(not(debug_assertions))]
    pub const SHADER_LIB_IMPORT_FOLDER_PATH: &'static str = "Assets/shaders";

    pub fn new_with_defaults() -> Result<Self, ShaderPreprocessorError> {
        let mut s = Self {
            known_imports: HashMap::new(),
        };

        #[cfg(debug_assertions)]
        s.import_folder(Self::SHADER_LIB_IMPORT_FOLDER_PATH_DEBUG_BUILD)?;

        #[cfg(not(debug_assertions))]
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

    pub fn add_file_import<D: Into<String>, P: Into<PathBuf>>(
        &mut self,
        directive: Option<D>,
        path: P,
    ) -> Result<(), ShaderPreprocessorError> {
        let path: PathBuf = path.into();

        let directive: String = directive.map(|x| x.into()).unwrap_or(
            path.file_stem()
                .expect("A filename must be present")
                .to_str()
                .ok_or(ShaderPreprocessorError::NonUTF8FileName {
                    file_name: path.clone().into_os_string(),
                })?
                .to_string(),
        );

        let content = read_to_string(&path).map_err(ShaderPreprocessorError::IOError)?;

        self.add_import(directive, content);

        Ok(())
    }

    pub fn import_folder<S: Into<String>>(
        &mut self,
        path: S,
    ) -> Result<(), ShaderPreprocessorError> {
        const PATTERN: &str = "**/*.wgsl";

        let path_into = path.into();

        let mut pattern_path = path_into.clone();
        if !pattern_path.ends_with("/") {
            pattern_path.push('/');
        }

        pattern_path += PATTERN;
        for entry in glob(&pattern_path)
            .map_err(ShaderPreprocessorError::PatternError)?
            .filter_map(Result::ok)
        {
            let directive = &entry
                .strip_prefix(&path_into)
                .expect("Base got merged into pattern. It cannot not be here.")
                .to_str()
                .ok_or(ShaderPreprocessorError::NonUTF8FileName {
                    file_name: entry.clone().into_os_string(),
                })?
                .replace("\\", "/")
                .replace(".wgsl", "")
                .to_lowercase();

            let content = read_to_string(&entry).map_err(ShaderPreprocessorError::IOError)?;
            debug!(
                "Imported content for directive '{}' ({:?}):\n{}\n",
                directive,
                canonicalize(&entry)
                    .expect("Debug print for canonicalized relative path failed ...?"),
                content
            );

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
