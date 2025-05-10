use std::{
    collections::HashMap,
    fs::{canonicalize, read_to_string},
    path::PathBuf,
};

use glob::glob;
use log::debug;

mod error;
pub use error::ShaderPreprocessorError;

#[cfg(test)]
mod tests;

pub struct ShaderPreprocessor {
    known_imports: HashMap<String, String>,
}

impl ShaderPreprocessor {
    /// The expected start expression of a directive.
    /// The full expression should be:
    /// ```wgsl
    /// #import<$1>
    /// ```
    /// Where $1 is the name of your import.
    pub const IMPORT_EXPRESSION_START: &'static str = "#import <";

    /// The expected end expression of a directive.
    /// The full expression should be:
    /// ```wgsl
    /// #import<$1>
    /// ```
    /// Where $1 is the name of your import.
    pub const IMPORT_EXPRESSION_END: &'static str = ">";

    /// Path to the expected shader lib to be used for default importing.
    #[cfg(debug_assertions)]
    pub const SHADER_LIB_IMPORT_FOLDER_PATH_DEBUG_BUILD: &'static str = "../../Assets/Shaders";

    /// Path to the expected shader lib to be used for default importing.
    #[cfg(not(debug_assertions))]
    pub const SHADER_LIB_IMPORT_FOLDER_PATH: &'static str = "Assets/shaders";

    /// Creates a new shader compiler.
    /// Note that each instance of this needs to have your imports imported!
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

    /// Registers a custom import directive for shader processing.
    ///
    /// When a shader contains the specified `directive`, it will be automatically replaced with
    /// the provided `content` during compilation. This enables modular shader development through
    /// pseudo-imports of common code snippets.
    ///
    /// # Arguments
    /// * `directive` - The unique identifier to search for in shader code (e.g., "#import utils")
    /// * `content` - The actual GLSL code to insert when the directive is found
    pub fn add_import<S0: Into<String>, S1: Into<String>>(&mut self, directive: S0, content: S1) {
        self.known_imports.insert(directive.into(), content.into());
    }

    /// Adds an import directive by reading the contents of a file.
    ///
    /// This method reads the contents of a specified file and registers it as a custom import
    /// directive. If a specific directive is provided, it will be used; otherwise, the file's base name
    /// (without extension) will be used as the directive.
    ///
    /// # Arguments
    /// * `directive` - An optional identifier to use for the import directive.
    ///   If not provided, the file's base name is used.
    /// * `path` - The path to the file containing the GLSL code to import.
    pub fn add_file_import<D: Into<String>, P: Into<PathBuf>>(
        &mut self,
        directive: Option<D>,
        path: P,
    ) -> Result<(), ShaderPreprocessorError> {
        let path: PathBuf = path.into();

        // Get the file name without the extension as the directive
        let directive: String = directive.map(|x| x.into()).unwrap_or(
            (&path.file_stem())
                .expect("A filename must be present")
                .to_str()
                .ok_or(ShaderPreprocessorError::NonUTF8FileName {
                    file_name: path.clone().into_os_string(),
                })?
                .to_string(),
        );

        // Read the file content
        let content = read_to_string(&path).map_err(|e| ShaderPreprocessorError::IOError(e))?;

        // Register the directive and content using add_known_import
        self.add_import(directive, content);

        Ok(())
    }

    /// Recursively imports all WGSL shaders from a specified folder into the known imports.
    ///
    /// This method scans through a given directory for all `.wgsl` files, reads their contents,
    /// and registers them as custom import directives. The import directive name is derived from
    /// the file's path relative to the specified directory, with the extension removed and all
    /// directory separators replaced by slashes. The registration allows these imported shaders
    /// to be included in other shaders using the `#import` directive.
    ///
    /// # Arguments
    /// * `path` - The path to the directory containing the WGSL files to import.
    pub fn import_folder<S: Into<String>>(
        &mut self,
        path: S,
    ) -> Result<(), ShaderPreprocessorError> {
        const PATTERN: &'static str = "**/*.wgsl";

        let path_into = path.into();

        let mut pattern_path = path_into.clone();
        if !pattern_path.ends_with("/") {
            pattern_path.push('/');
        }
        debug!(
            "Folder import path canonicalized: {:?}",
            canonicalize(&pattern_path)
        );

        pattern_path += PATTERN;
        debug!("Pattern: {:?}", pattern_path);
        for entry in glob(&pattern_path)
            .map_err(|e| ShaderPreprocessorError::PatternError(e))?
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

            let content =
                read_to_string(&entry).map_err(|e| ShaderPreprocessorError::IOError(e))?;
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

    /// Parses a shader from source code.
    /// Any supported preprocessor definitions will be added as they are imported.
    pub fn parse_shader<S: Into<String>>(
        &self,
        source: S,
    ) -> Result<String, ShaderPreprocessorError> {
        let source = source.into();
        let imported_directives = Vec::new();
        self.parse_shader_(source, imported_directives)
    }

    /// Part of [Self::parse_shader].  
    /// Does the work, but is designed for recursive calls.
    fn parse_shader_(
        &self,
        source: String,
        imported_directives: Vec<&str>,
    ) -> Result<String, ShaderPreprocessorError> {
        let mut output = String::new();
        let mut imported_directives = imported_directives;
        let mut import_found = false;

        for line in source.lines() {
            if let Some(start) = line.find(Self::IMPORT_EXPRESSION_START) {
                if let Some(end) = line.find(Self::IMPORT_EXPRESSION_END) {
                    let directive = &line[start + Self::IMPORT_EXPRESSION_START.len()..end];
                    if imported_directives.contains(&directive) {
                        // Already imported in this shader so SKIP!
                        continue;
                    } else {
                        // Otherwise we need to add this directive
                        imported_directives.push(directive);

                        // Flag import found to true, this indicates that we need to run the shader preprocessor _again_ until there are no more imports found.
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

                    // Skip this line from being included as we replaced it with the import
                    continue;
                }
            }

            // No match, so just add the line to the shader
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
