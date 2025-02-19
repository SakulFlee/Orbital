use std::{
    collections::HashMap,
    fs::{canonicalize, read_to_string},
    path::PathBuf,
};

use glob::glob;
use log::debug;

pub struct Shaders {
    known_imports: HashMap<String, String>,
}

impl Shaders {
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
    pub const SHADER_LIB_IMPORT_FOLDER_PATH_DEBUG_BUILD: &'static str =
        "../../crates/shaders/shaders";

    /// Path to the expected shader lib to be used for default importing.
    #[cfg(not(debug_assertions))]
    pub const SHADER_LIB_IMPORT_FOLDER_PATH: &'static str = "Assets/shaders";

    /// Creates a new shader compiler.
    /// Note that each instance of this needs to have your imports imported!
    pub fn new(import_default: bool) -> Result<Self, String> {
        let mut s = Self {
            known_imports: HashMap::new(),
        };

        if import_default {
            #[cfg(debug_assertions)]
            s.import_folder(Self::SHADER_LIB_IMPORT_FOLDER_PATH_DEBUG_BUILD)?;

            #[cfg(not(debug_assertions))]
            s.import_folder(Self::SHADER_LIB_IMPORT_FOLDER_PATH)?;
        }

        Ok(s)
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
    ) {
        let path = path.into();

        // Get the file name without the extension as the directive
        let directive = directive.map(|x| x.into()).unwrap_or(
            path.file_stem()
                .and_then(|stem| stem.to_str())
                .map(|s| s.to_string())
                .expect("Failed to get file name"),
        );

        // Read the file content
        let content = read_to_string(&path).expect(&format!("Failed to read file: {:?}", path));

        // Register the directive and content using add_known_import
        self.add_import(directive, content);
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
    pub fn import_folder<S: Into<String>>(&mut self, path: S) -> Result<(), String> {
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
        let glob = glob(&pattern_path).expect("Failure building glob ...");
        for entry in glob {
            let relative_entry_path = entry.map_err(|e| e.to_string())?;
            let directive = &relative_entry_path
                .strip_prefix(&path_into)
                .map_err(|e| e.to_string())?
                .to_str()
                .ok_or(String::from("Non UTF-8 name detected in import directive!"))?
                .replace("\\", "/")
                .replace(".wgsl", "")
                .to_lowercase();

            let content = read_to_string(&relative_entry_path).map_err(|e| e.to_string())?;
            debug!(
                "Imported content for directive '{}' ({:?}):\n{}\n",
                directive,
                canonicalize(&relative_entry_path)
                    .expect("Debug print for canonicalized relative path failed ...?"),
                content
            );

            self.add_import(directive, content);
        }

        Ok(())
    }

    /// Parses a shader from source code.
    /// Any supported preprocessor definitions will be added as they are imported.
    pub fn parse_shader<S: Into<String>>(&self, source: S) -> Result<String, String> {
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
    ) -> Result<String, String> {
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

                    let import = self
                        .known_imports
                        .get(directive)
                        .ok_or(format!("Import directive '{directive}' is unknown!"))?;

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

/// Tests if a shader without any imports parses without any changes being done to it. The input must equal the output!
#[cfg(test)]
#[test]
fn test_parse_shader_default_imports() {
    let shaders = Shaders::new(true).expect("Failed constructing instance of Shaders!");
    assert!(!shaders.known_imports.is_empty());
}

/// Tests if a shader without any imports parses without any changes being done to it. The input must equal the output!
#[cfg(test)]
#[test]
fn test_parse_shader_no_imports() {
    const SHADER: &'static str = "fn main() {
    let i: i32 = 0;
}";

    let shaders = Shaders::new(false).expect("Failed constructing instance of Shaders!");
    assert!(shaders.known_imports.is_empty());

    let parsed_shader = shaders
        .parse_shader(SHADER)
        .expect("Shader parsing failed!");
    // Make sure that nothing changed
    assert_eq!(parsed_shader, SHADER);
}

/// Tests if a very simple shader with an import is processed correctly.
/// The processed shader has to have the content, "TEST PASSED!", included into it.
#[cfg(test)]
#[test]
fn test_parse_shader() {
    const IMPORT_DIRECTIVE: &'static str = "this/is/a/test";
    const IMPORT_CONTENT: &'static str = "TEST PASSED!";

    let mut shaders = Shaders::new(false).expect("Failed to construct Shaders struct!");
    shaders.add_import(IMPORT_DIRECTIVE, IMPORT_CONTENT);

    assert!(!shaders.known_imports.is_empty());
    assert_eq!(shaders.known_imports.len(), 1);
    assert!(shaders.known_imports.contains_key(IMPORT_DIRECTIVE));
    assert_eq!(
        shaders
            .known_imports
            .get(IMPORT_DIRECTIVE)
            .expect("Missing import!"),
        IMPORT_CONTENT
    );

    let shader_source = format!("#import <{}>", IMPORT_DIRECTIVE);
    let parsed_shader = shaders
        .parse_shader(shader_source)
        .expect("Shader parsing failed!");
    assert_eq!(parsed_shader, IMPORT_CONTENT);
}

/// Tests multiple things:
/// - If multiple directives can be defined
/// - If multiple contents can be defined
/// - If a shader can have multiple imports
///
/// The end result must include every import.
#[cfg(test)]
#[test]
fn test_parse_shader_multi_import() {
    const DIRECTIVE_0: &'static str = "test0";
    const DIRECTIVE_1: &'static str = "test1";
    const DIRECTIVE_2: &'static str = "test2";
    const DIRECTIVE_3: &'static str = "test3";
    const CONTENT_0: &'static str = "Just some example content!";
    const CONTENT_1: &'static str = "ABCD";
    const CONTENT_2: &'static str = "Even More testing content!";
    const CONTENT_3: &'static str =
        "You might be surprised but there is actually EVEN MORE test content here!";

    let mut shaders = Shaders::new(false).expect("Failed to construct Shaders struct!");
    shaders.add_import(DIRECTIVE_0, CONTENT_0);
    shaders.add_import(DIRECTIVE_1, CONTENT_1);
    shaders.add_import(DIRECTIVE_2, CONTENT_2);
    shaders.add_import(DIRECTIVE_3, CONTENT_3);

    assert!(!shaders.known_imports.is_empty());
    assert_eq!(shaders.known_imports.len(), 4);
    assert!(shaders.known_imports.contains_key(DIRECTIVE_0));
    assert!(shaders.known_imports.contains_key(DIRECTIVE_1));
    assert!(shaders.known_imports.contains_key(DIRECTIVE_2));
    assert!(shaders.known_imports.contains_key(DIRECTIVE_3));
    assert_eq!(
        shaders
            .known_imports
            .get(DIRECTIVE_0)
            .expect("Missing import!"),
        CONTENT_0
    );
    assert_eq!(
        shaders
            .known_imports
            .get(DIRECTIVE_1)
            .expect("Missing import!"),
        CONTENT_1
    );
    assert_eq!(
        shaders
            .known_imports
            .get(DIRECTIVE_2)
            .expect("Missing import!"),
        CONTENT_2
    );
    assert_eq!(
        shaders
            .known_imports
            .get(DIRECTIVE_3)
            .expect("Missing import!"),
        CONTENT_3
    );

    let shader_source = format!(
        "#import <{DIRECTIVE_0}>
#import <{DIRECTIVE_1}>
#import <{DIRECTIVE_2}>
#import <{DIRECTIVE_3}>"
    );
    let parsed_shader = shaders
        .parse_shader(shader_source)
        .expect("Shader parsing failed!");
    assert_eq!(
        parsed_shader,
        format!(
            "{CONTENT_0}
{CONTENT_1}
{CONTENT_2}
{CONTENT_3}"
        )
    );
}

/// Similar to [test_parse_shader_multi_import], but additionally checks if the order of imports is following the order of import statements.
/// This _should_ never be a problem as WGSL itself is preprocessed, but I like to enforce this rule.
/// Also, if in the future we add additional shading languages, this might become important.
#[cfg(test)]
#[test]
fn test_parse_shader_multi_import_order() {
    const DIRECTIVE_0: &'static str = "test0";
    const DIRECTIVE_1: &'static str = "test1";
    const DIRECTIVE_2: &'static str = "test2";
    const DIRECTIVE_3: &'static str = "test3";
    const CONTENT_0: &'static str = "Just some example content!";
    const CONTENT_1: &'static str = "ABCD";
    const CONTENT_2: &'static str = "Even More testing content!";
    const CONTENT_3: &'static str =
        "You might be surprised but there is actually EVEN MORE test content here!";

    let mut shaders = Shaders::new(false).expect("Failed to construct Shaders struct!");
    shaders.add_import(DIRECTIVE_0, CONTENT_0);
    shaders.add_import(DIRECTIVE_1, CONTENT_1);
    shaders.add_import(DIRECTIVE_2, CONTENT_2);
    shaders.add_import(DIRECTIVE_3, CONTENT_3);

    assert!(!shaders.known_imports.is_empty());
    assert_eq!(shaders.known_imports.len(), 4);
    assert!(shaders.known_imports.contains_key(DIRECTIVE_0));
    assert!(shaders.known_imports.contains_key(DIRECTIVE_1));
    assert!(shaders.known_imports.contains_key(DIRECTIVE_2));
    assert!(shaders.known_imports.contains_key(DIRECTIVE_3));
    assert_eq!(
        shaders
            .known_imports
            .get(DIRECTIVE_0)
            .expect("Missing import!"),
        CONTENT_0
    );
    assert_eq!(
        shaders
            .known_imports
            .get(DIRECTIVE_1)
            .expect("Missing import!"),
        CONTENT_1
    );
    assert_eq!(
        shaders
            .known_imports
            .get(DIRECTIVE_2)
            .expect("Missing import!"),
        CONTENT_2
    );
    assert_eq!(
        shaders
            .known_imports
            .get(DIRECTIVE_3)
            .expect("Missing import!"),
        CONTENT_3
    );

    let shader_source = format!(
        "#import <{DIRECTIVE_2}>
#import <{DIRECTIVE_1}>
fn some_function() {{ ... }}
#import <{DIRECTIVE_3}>
const i: i32 = 123;
#import <{DIRECTIVE_0}>"
    );
    let parsed_shader = shaders
        .parse_shader(shader_source)
        .expect("Shader parsing failed!");
    assert_eq!(
        parsed_shader,
        format!(
            "{CONTENT_2}
{CONTENT_1}
fn some_function() {{ ... }}
{CONTENT_3}
const i: i32 = 123;
{CONTENT_0}"
        )
    );
}

/// Tests if a duplicated import will be included or not.
/// The result should only have one instance of the content, "TEST PASSED!".
/// Anymore than one will cause problems in WGSL.
#[cfg(test)]
#[test]
fn test_parse_shader_duplicate_import() {
    const IMPORT_DIRECTIVE: &'static str = "this/is/a/test";
    const IMPORT_CONTENT: &'static str = "TEST PASSED!";

    let mut shaders = Shaders::new(false).expect("Failed to construct Shaders struct!");
    shaders.add_import(IMPORT_DIRECTIVE, IMPORT_CONTENT);

    assert!(!shaders.known_imports.is_empty());
    assert_eq!(shaders.known_imports.len(), 1);
    assert!(shaders.known_imports.contains_key(IMPORT_DIRECTIVE));
    assert_eq!(
        shaders
            .known_imports
            .get(IMPORT_DIRECTIVE)
            .expect("Missing import!"),
        IMPORT_CONTENT
    );

    let shader_source = format!(
        "#import <{IMPORT_DIRECTIVE}>
#import <{IMPORT_DIRECTIVE}>"
    );
    let parsed_shader = shaders
        .parse_shader(shader_source)
        .expect("Shader parsing failed!");

    let location = parsed_shader
        .find(IMPORT_CONTENT)
        .expect("Must be found as assert passed already!?");
    let remaining = &parsed_shader[location + IMPORT_CONTENT.len()..];

    // Fail if another iteration of content is found!
    assert!(!remaining.contains(IMPORT_CONTENT));
}

/// Tests what happens if an import, imports itself.
/// This should not end in an infinite loop.
/// Nor, should it include the import multiple times.
#[cfg(test)]
#[test]
fn test_parse_shader_recursive_import_itself() {
    const IMPORT_DIRECTIVE: &'static str = "this/is/a/test";
    let import_content = format!("#import <{IMPORT_DIRECTIVE}>");

    let mut shaders = Shaders::new(false).expect("Failed to construct Shaders struct!");
    shaders.add_import(IMPORT_DIRECTIVE, &import_content);

    assert!(!shaders.known_imports.is_empty());
    assert_eq!(shaders.known_imports.len(), 1);
    assert!(shaders.known_imports.contains_key(IMPORT_DIRECTIVE));
    assert_eq!(
        shaders
            .known_imports
            .get(IMPORT_DIRECTIVE)
            .expect("Missing import!"),
        &import_content
    );

    let shader_source = format!("#import <{IMPORT_DIRECTIVE}>");
    let parsed_shader = shaders
        .parse_shader(shader_source)
        .expect("Shader parsing failed!");

    // Recursive including should yield to nothing in this case.
    assert_eq!(parsed_shader, "");
    assert_eq!(parsed_shader.len(), 0);
    assert!(parsed_shader.is_empty());
}

/// The shader is importing directive 0.
/// Directive 0 imports directive 1.
/// I.e. the import flow is:
/// Shader -> 0 -> 1
///
/// The end result should be:
/// test1 <- from directive 1, which is imported by 0
/// test0 <- from directive 0, which is imported by shader
#[cfg(test)]
#[test]
fn test_parse_shader_recursive_import() {
    const DIRECTIVE_0: &'static str = "test0";
    const DIRECTIVE_1: &'static str = "test1";
    let content_0 = format!(
        "#import <{DIRECTIVE_1}>
{DIRECTIVE_0}"
    );
    let content_1 = format!("{DIRECTIVE_1}");

    let mut shaders = Shaders::new(false).expect("Failed to construct Shaders struct!");
    shaders.add_import(DIRECTIVE_0, &content_0);
    shaders.add_import(DIRECTIVE_1, &content_1);

    assert!(!shaders.known_imports.is_empty());
    assert_eq!(shaders.known_imports.len(), 2);
    assert!(shaders.known_imports.contains_key(DIRECTIVE_0));
    assert!(shaders.known_imports.contains_key(DIRECTIVE_1));
    assert_eq!(
        shaders
            .known_imports
            .get(DIRECTIVE_0)
            .expect("Missing import!"),
        &content_0
    );
    assert_eq!(
        shaders
            .known_imports
            .get(DIRECTIVE_1)
            .expect("Missing import!"),
        &content_1
    );

    let shader_source = format!("#import <{DIRECTIVE_0}>");
    let parsed_shader = shaders
        .parse_shader(shader_source)
        .expect("Shader parsing failed!");

    assert_eq!(
        parsed_shader,
        format!(
            "{DIRECTIVE_1}
{DIRECTIVE_0}"
        )
    );
}

/// Tests if a directive defined via `add_import` actually gets added to the knowledge database.
#[cfg(test)]
#[test]
fn test_add_import() {
    const DIRECTIVE: &'static str = "this/is/a/test";
    const CONTENT: &'static str = "Just some example content!";

    let mut shaders = Shaders::new(false).expect("Failed to construct Shaders struct!");
    shaders.add_import(DIRECTIVE, CONTENT);

    assert!(!shaders.known_imports.is_empty());
    assert_eq!(shaders.known_imports.len(), 1);
    assert!(shaders.known_imports.contains_key(DIRECTIVE));
    assert_eq!(
        shaders
            .known_imports
            .get(DIRECTIVE)
            .expect("Missing import!"),
        CONTENT
    );
}

/// Tests if a file can be imported into the knowledge database.
#[cfg(test)]
#[test]
fn test_add_file_import() {
    const PATH: &'static str = "../../crates/shaders/shaders/pbr/pbr.wgsl";
    const CONTENT: &'static str = include_str!("../shaders/pbr/pbr.wgsl");
    const EXPECTED_DIRECTIVE: &'static str = "pbr";

    let mut shaders = Shaders::new(false).expect("Failed to construct Shaders struct!");
    shaders.add_file_import(None::<String>, PATH);

    assert!(!shaders.known_imports.is_empty());
    assert_eq!(shaders.known_imports.len(), 1);
    assert!(shaders.known_imports.contains_key(EXPECTED_DIRECTIVE));
    assert_eq!(
        shaders
            .known_imports
            .get(EXPECTED_DIRECTIVE)
            .expect("Missing import!"),
        CONTENT
    );
}

/// Same as [test_add_file_import], but attempts adding the file with a custom directive.
#[cfg(test)]
#[test]
fn test_add_file_import_custom_directive() {
    const PATH: &'static str = "../../crates/shaders/shaders/pbr/pbr.wgsl";
    const CONTENT: &'static str = include_str!("../shaders/pbr/pbr.wgsl");
    const EXPECTED_DIRECTIVE: &'static str = "pbr/pbr";

    let mut shaders = Shaders::new(false).expect("Failed to construct Shaders struct!");
    shaders.add_file_import(Some(EXPECTED_DIRECTIVE.to_string()), PATH);

    assert!(!shaders.known_imports.is_empty());
    assert_eq!(shaders.known_imports.len(), 1);
    assert!(shaders.known_imports.contains_key(EXPECTED_DIRECTIVE));
    assert_eq!(
        shaders
            .known_imports
            .get(EXPECTED_DIRECTIVE)
            .expect("Missing import!"),
        CONTENT
    );
}

/// Tests if a folder can be imported.
#[cfg(test)]
#[test]
fn test_folder_import() {
    let mut shaders = Shaders::new(false).expect("Failed to construct Shaders struct!");
    shaders
        .import_folder("../../crates/shaders/shaders")
        .expect("Failure to import shader lib!");

    assert!(!shaders.known_imports.is_empty());
}

/// Tests if a known directive, pbr/pbr in this case, can be found after folder import.
#[cfg(test)]
#[test]
fn test_folder_import_contains_pbr() {
    let mut shaders = Shaders::new(false).expect("Failed to construct Shaders struct!");
    shaders
        .import_folder("../../crates/shaders/shaders")
        .expect("Failure to import shader lib!");

    assert!(shaders.known_imports.contains_key("pbr/pbr"));
}

/// Tests if there are any entries in the knowledge database after a folder got imported.
#[cfg(test)]
#[test]
fn test_folder_import_not_empty() {
    let mut shaders = Shaders::new(false).expect("Failed to construct Shaders struct!");
    shaders
        .import_folder("../../crates/shaders/shaders")
        .expect("Failure to import shader lib!");

    assert!(!shaders.known_imports.is_empty());
}
