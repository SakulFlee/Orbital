// NOTE: DISABLED DUE TO rewrite necessary!
// See: https://github.com/SakulFlee/Orbital/issues/477

// use super::ShaderPreprocessor;
//
// pub const SHADER_PATH: &str = "../../Assets/Shaders/";
//
// /// Tests if a shader without any imports parses without any changes being done to it. The input must equal the output!
// #[cfg(test)]
// #[test]
// fn test_parse_shader_default_imports() {
//     let shader_preprocessor =
//         ShaderPreprocessor::new_with_defaults().expect("Failed constructing instance of Shaders!");
//     assert!(!shader_preprocessor.known_imports.is_empty());
// }
//
// /// Tests if a shader without any imports parses without any changes being done to it. The input must equal the output!
// #[cfg(test)]
// #[test]
// fn test_parse_shader_no_imports() {
//     const SHADER: &str = "fn main() {
//     let i: i32 = 0;
// }";
//
//     let shader_preprocessor = ShaderPreprocessor::new_empty();
//     assert!(shader_preprocessor.known_imports.is_empty());
//
//     let parsed_shader = shader_preprocessor
//         .parse_shader(SHADER)
//         .expect("Shader parsing failed!");
//     // Make sure that nothing changed
//     assert_eq!(parsed_shader, SHADER);
// }
//
// /// Tests if a very simple shader with an import is processed correctly.
// /// The processed shader has to have the content, "TEST PASSED!", included into it.
// #[cfg(test)]
// #[test]
// fn test_parse_shader() {
//     const IMPORT_DIRECTIVE: &str = "this/is/a/test";
//     const IMPORT_CONTENT: &str = "TEST PASSED!";
//
//     let mut shader_preprocessor = ShaderPreprocessor::new_empty();
//     shader_preprocessor.add_import(IMPORT_DIRECTIVE, IMPORT_CONTENT);
//
//     assert!(!shader_preprocessor.known_imports.is_empty());
//     assert_eq!(shader_preprocessor.known_imports.len(), 1);
//     assert!(shader_preprocessor
//         .known_imports
//         .contains_key(IMPORT_DIRECTIVE));
//     assert_eq!(
//         shader_preprocessor
//             .known_imports
//             .get(IMPORT_DIRECTIVE)
//             .expect("Missing import!"),
//         IMPORT_CONTENT
//     );
//
//     let shader_source = format!("#import <{}>", IMPORT_DIRECTIVE);
//     let parsed_shader = shader_preprocessor
//         .parse_shader(shader_source)
//         .expect("Shader parsing failed!");
//     assert_eq!(parsed_shader, IMPORT_CONTENT);
// }
//
// /// Tests multiple things:
// /// - If multiple directives can be defined
// /// - If multiple contents can be defined
// /// - If a shader can have multiple imports
// ///
// /// The end result must include every import.
// #[cfg(test)]
// #[test]
// fn test_parse_shader_multi_import() {
//     const DIRECTIVE_0: &str = "test0";
//     const DIRECTIVE_1: &str = "test1";
//     const DIRECTIVE_2: &str = "test2";
//     const DIRECTIVE_3: &str = "test3";
//     const CONTENT_0: &str = "Just some example content!";
//     const CONTENT_1: &str = "ABCD";
//     const CONTENT_2: &str = "Even More testing content!";
//     const CONTENT_3: &str =
//         "You might be surprised but there is actually EVEN MORE test content here!";
//
//     let mut shader_preprocessor = ShaderPreprocessor::new_empty();
//     shader_preprocessor.add_import(DIRECTIVE_0, CONTENT_0);
//     shader_preprocessor.add_import(DIRECTIVE_1, CONTENT_1);
//     shader_preprocessor.add_import(DIRECTIVE_2, CONTENT_2);
//     shader_preprocessor.add_import(DIRECTIVE_3, CONTENT_3);
//
//     assert!(!shader_preprocessor.known_imports.is_empty());
//     assert_eq!(shader_preprocessor.known_imports.len(), 4);
//     assert!(shader_preprocessor.known_imports.contains_key(DIRECTIVE_0));
//     assert!(shader_preprocessor.known_imports.contains_key(DIRECTIVE_1));
//     assert!(shader_preprocessor.known_imports.contains_key(DIRECTIVE_2));
//     assert!(shader_preprocessor.known_imports.contains_key(DIRECTIVE_3));
//     assert_eq!(
//         shader_preprocessor
//             .known_imports
//             .get(DIRECTIVE_0)
//             .expect("Missing import!"),
//         CONTENT_0
//     );
//     assert_eq!(
//         shader_preprocessor
//             .known_imports
//             .get(DIRECTIVE_1)
//             .expect("Missing import!"),
//         CONTENT_1
//     );
//     assert_eq!(
//         shader_preprocessor
//             .known_imports
//             .get(DIRECTIVE_2)
//             .expect("Missing import!"),
//         CONTENT_2
//     );
//     assert_eq!(
//         shader_preprocessor
//             .known_imports
//             .get(DIRECTIVE_3)
//             .expect("Missing import!"),
//         CONTENT_3
//     );
//
//     let shader_source = format!(
//         "#import <{DIRECTIVE_0}>
// #import <{DIRECTIVE_1}>
// #import <{DIRECTIVE_2}>
// #import <{DIRECTIVE_3}>"
//     );
//     let parsed_shader = shader_preprocessor
//         .parse_shader(shader_source)
//         .expect("Shader parsing failed!");
//     assert_eq!(
//         parsed_shader,
//         format!(
//             "{CONTENT_0}
// {CONTENT_1}
// {CONTENT_2}
// {CONTENT_3}"
//         )
//     );
// }
//
// /// Similar to [test_parse_shader_multi_import], but additionally checks if the order of imports is following the order of import statements.
// /// This _should_ never be a problem as WGSL itself is preprocessed, but I like to enforce this rule.
// /// Also, if in the future we add additional shading languages, this might become important.
// #[cfg(test)]
// #[test]
// fn test_parse_shader_multi_import_order() {
//     const DIRECTIVE_0: &str = "test0";
//     const DIRECTIVE_1: &str = "test1";
//     const DIRECTIVE_2: &str = "test2";
//     const DIRECTIVE_3: &str = "test3";
//     const CONTENT_0: &str = "Just some example content!";
//     const CONTENT_1: &str = "ABCD";
//     const CONTENT_2: &str = "Even More testing content!";
//     const CONTENT_3: &str =
//         "You might be surprised but there is actually EVEN MORE test content here!";
//
//     let mut shader_preprocessor = ShaderPreprocessor::new_empty();
//     shader_preprocessor.add_import(DIRECTIVE_0, CONTENT_0);
//     shader_preprocessor.add_import(DIRECTIVE_1, CONTENT_1);
//     shader_preprocessor.add_import(DIRECTIVE_2, CONTENT_2);
//     shader_preprocessor.add_import(DIRECTIVE_3, CONTENT_3);
//
//     assert!(!shader_preprocessor.known_imports.is_empty());
//     assert_eq!(shader_preprocessor.known_imports.len(), 4);
//     assert!(shader_preprocessor.known_imports.contains_key(DIRECTIVE_0));
//     assert!(shader_preprocessor.known_imports.contains_key(DIRECTIVE_1));
//     assert!(shader_preprocessor.known_imports.contains_key(DIRECTIVE_2));
//     assert!(shader_preprocessor.known_imports.contains_key(DIRECTIVE_3));
//     assert_eq!(
//         shader_preprocessor
//             .known_imports
//             .get(DIRECTIVE_0)
//             .expect("Missing import!"),
//         CONTENT_0
//     );
//     assert_eq!(
//         shader_preprocessor
//             .known_imports
//             .get(DIRECTIVE_1)
//             .expect("Missing import!"),
//         CONTENT_1
//     );
//     assert_eq!(
//         shader_preprocessor
//             .known_imports
//             .get(DIRECTIVE_2)
//             .expect("Missing import!"),
//         CONTENT_2
//     );
//     assert_eq!(
//         shader_preprocessor
//             .known_imports
//             .get(DIRECTIVE_3)
//             .expect("Missing import!"),
//         CONTENT_3
//     );
//
//     let shader_source = format!(
//         "#import <{DIRECTIVE_2}>
// #import <{DIRECTIVE_1}>
// fn some_function() {{ ... }}
// #import <{DIRECTIVE_3}>
// const i: i32 = 123;
// #import <{DIRECTIVE_0}>"
//     );
//     let parsed_shader = shader_preprocessor
//         .parse_shader(shader_source)
//         .expect("Shader parsing failed!");
//     assert_eq!(
//         parsed_shader,
//         format!(
//             "{CONTENT_2}
// {CONTENT_1}
// fn some_function() {{ ... }}
// {CONTENT_3}
// const i: i32 = 123;
// {CONTENT_0}"
//         )
//     );
// }
//
// /// Tests if a duplicated import will be included or not.
// /// The result should only have one instance of the content, "TEST PASSED!".
// /// Anymore than one will cause problems in WGSL.
// #[cfg(test)]
// #[test]
// fn test_parse_shader_duplicate_import() {
//     const IMPORT_DIRECTIVE: &str = "this/is/a/test";
//     const IMPORT_CONTENT: &str = "TEST PASSED!";
//
//     let mut shader_preprocessor = ShaderPreprocessor::new_empty();
//     shader_preprocessor.add_import(IMPORT_DIRECTIVE, IMPORT_CONTENT);
//
//     assert!(!shader_preprocessor.known_imports.is_empty());
//     assert_eq!(shader_preprocessor.known_imports.len(), 1);
//     assert!(shader_preprocessor
//         .known_imports
//         .contains_key(IMPORT_DIRECTIVE));
//     assert_eq!(
//         shader_preprocessor
//             .known_imports
//             .get(IMPORT_DIRECTIVE)
//             .expect("Missing import!"),
//         IMPORT_CONTENT
//     );
//
//     let shader_source = format!(
//         "#import <{IMPORT_DIRECTIVE}>
// #import <{IMPORT_DIRECTIVE}>"
//     );
//     let parsed_shader = shader_preprocessor
//         .parse_shader(shader_source)
//         .expect("Shader parsing failed!");
//
//     let location = parsed_shader
//         .find(IMPORT_CONTENT)
//         .expect("Must be found as assert passed already!?");
//     let remaining = &parsed_shader[location + IMPORT_CONTENT.len()..];
//
//     // Fail if another iteration of content is found!
//     assert!(!remaining.contains(IMPORT_CONTENT));
// }
//
// /// Tests what happens if an import, imports itself.
// /// This should not end in an infinite loop.
// /// Nor, should it include the import multiple times.
// #[cfg(test)]
// #[test]
// fn test_parse_shader_recursive_import_itself() {
//     const IMPORT_DIRECTIVE: &str = "this/is/a/test";
//     let import_content = format!("#import <{IMPORT_DIRECTIVE}>");
//
//     let mut shader_preprocessor = ShaderPreprocessor::new_empty();
//     shader_preprocessor.add_import(IMPORT_DIRECTIVE, &import_content);
//
//     assert!(!shader_preprocessor.known_imports.is_empty());
//     assert_eq!(shader_preprocessor.known_imports.len(), 1);
//     assert!(shader_preprocessor
//         .known_imports
//         .contains_key(IMPORT_DIRECTIVE));
//     assert_eq!(
//         shader_preprocessor
//             .known_imports
//             .get(IMPORT_DIRECTIVE)
//             .expect("Missing import!"),
//         &import_content
//     );
//
//     let shader_source = format!("#import <{IMPORT_DIRECTIVE}>");
//     let parsed_shader = shader_preprocessor
//         .parse_shader(shader_source)
//         .expect("Shader parsing failed!");
//
//     // Recursive including should yield to nothing in this case.
//     assert_eq!(parsed_shader, "");
//     assert_eq!(parsed_shader.len(), 0);
//     assert!(parsed_shader.is_empty());
// }
//
// /// The shader is importing directive 0.
// /// Directive 0 imports directive 1.
// /// I.e. the import flow is:
// /// Shader -> 0 -> 1
// ///
// /// The end result should be:
// /// test1 <- from directive 1, which is imported by 0
// /// test0 <- from directive 0, which is imported by shader
// #[cfg(test)]
// #[test]
// fn test_parse_shader_recursive_import() {
//     const DIRECTIVE_0: &str = "test0";
//     const DIRECTIVE_1: &str = "test1";
//     let content_0 = format!(
//         "#import <{DIRECTIVE_1}>
// {DIRECTIVE_0}"
//     );
//     let content_1 = DIRECTIVE_1.to_string();
//
//     let mut shader_preprocessor = ShaderPreprocessor::new_empty();
//     shader_preprocessor.add_import(DIRECTIVE_0, &content_0);
//     shader_preprocessor.add_import(DIRECTIVE_1, &content_1);
//
//     assert!(!shader_preprocessor.known_imports.is_empty());
//     assert_eq!(shader_preprocessor.known_imports.len(), 2);
//     assert!(shader_preprocessor.known_imports.contains_key(DIRECTIVE_0));
//     assert!(shader_preprocessor.known_imports.contains_key(DIRECTIVE_1));
//     assert_eq!(
//         shader_preprocessor
//             .known_imports
//             .get(DIRECTIVE_0)
//             .expect("Missing import!"),
//         &content_0
//     );
//     assert_eq!(
//         shader_preprocessor
//             .known_imports
//             .get(DIRECTIVE_1)
//             .expect("Missing import!"),
//         &content_1
//     );
//
//     let shader_source = format!("#import <{DIRECTIVE_0}>");
//     let parsed_shader = shader_preprocessor
//         .parse_shader(shader_source)
//         .expect("Shader parsing failed!");
//
//     assert_eq!(
//         parsed_shader,
//         format!(
//             "{DIRECTIVE_1}
// {DIRECTIVE_0}"
//         )
//     );
// }
//
// /// Tests if a directive defined via `add_import` actually gets added to the knowledge database.
// #[cfg(test)]
// #[test]
// fn test_add_import() {
//     const DIRECTIVE: &str = "this/is/a/test";
//     const CONTENT: &str = "Just some example content!";
//
//     let mut shader_preprocessor = ShaderPreprocessor::new_empty();
//     shader_preprocessor.add_import(DIRECTIVE, CONTENT);
//
//     assert!(!shader_preprocessor.known_imports.is_empty());
//     assert_eq!(shader_preprocessor.known_imports.len(), 1);
//     assert!(shader_preprocessor.known_imports.contains_key(DIRECTIVE));
//     assert_eq!(
//         shader_preprocessor
//             .known_imports
//             .get(DIRECTIVE)
//             .expect("Missing import!"),
//         CONTENT
//     );
// }
//
// /// Tests if a file can be imported into the knowledge database.
// #[cfg(test)]
// #[test]
// fn test_add_file_import() {
//     const PATH: &str = "pbr/pbr.wgsl";
//     const CONTENT: &str = include_str!("../../../Assets/Shaders/pbr/pbr.wgsl");
//     const EXPECTED_DIRECTIVE: &str = "pbr";
//
//     let mut shader_preprocessor = ShaderPreprocessor::new_empty();
//     shader_preprocessor
//         .add_file_import(None::<String>, format!("{}/{}", SHADER_PATH, PATH))
//         .expect("Adding file import failed!");
//
//     assert!(!shader_preprocessor.known_imports.is_empty());
//     assert_eq!(shader_preprocessor.known_imports.len(), 1);
//     assert!(shader_preprocessor
//         .known_imports
//         .contains_key(EXPECTED_DIRECTIVE));
//     assert_eq!(
//         shader_preprocessor
//             .known_imports
//             .get(EXPECTED_DIRECTIVE)
//             .expect("Missing import!"),
//         CONTENT
//     );
// }
//
// /// Same as [test_add_file_import], but attempts adding the file with a custom directive.
// #[cfg(test)]
// #[test]
// fn test_add_file_import_custom_directive() {
//     const PATH: &str = "pbr/pbr.wgsl";
//     const CONTENT: &str = include_str!("../../../Assets/Shaders/pbr/pbr.wgsl");
//     const EXPECTED_DIRECTIVE: &str = "pbr/pbr";
//
//     let mut shader_preprocessor = ShaderPreprocessor::new_empty();
//     shader_preprocessor
//         .add_file_import(
//             Some(EXPECTED_DIRECTIVE.to_string()),
//             format!("{}/{}", SHADER_PATH, PATH),
//         )
//         .expect("Adding file import failed!");
//
//     assert!(!shader_preprocessor.known_imports.is_empty());
//     assert_eq!(shader_preprocessor.known_imports.len(), 1);
//     assert!(shader_preprocessor
//         .known_imports
//         .contains_key(EXPECTED_DIRECTIVE));
//     assert_eq!(
//         shader_preprocessor
//             .known_imports
//             .get(EXPECTED_DIRECTIVE)
//             .expect("Missing import!"),
//         CONTENT
//     );
// }
//
// /// Tests if a folder can be imported.
// #[cfg(test)]
// #[test]
// fn test_folder_import() {
//     let mut shader_preprocessor = ShaderPreprocessor::new_empty();
//     shader_preprocessor
//         .import_folder(SHADER_PATH)
//         .expect("Failure to import shader lib!");
//
//     assert!(!shader_preprocessor.known_imports.is_empty());
// }
//
// /// Tests if a known directive, pbr/pbr in this case, can be found after folder import.
// #[cfg(test)]
// #[test]
// fn test_folder_import_contains_pbr() {
//     let mut shader_preprocessor = ShaderPreprocessor::new_empty();
//     shader_preprocessor
//         .import_folder(SHADER_PATH)
//         .expect("Failure to import shader lib!");
//
//     assert!(shader_preprocessor.known_imports.contains_key("pbr/pbr"));
// }
//
// /// Tests if there are any entries in the knowledge database after a folder got imported.
// #[cfg(test)]
// #[test]
// fn test_folder_import_not_empty() {
//     let mut shader_preprocessor = ShaderPreprocessor::new_empty();
//     shader_preprocessor
//         .import_folder(SHADER_PATH)
//         .expect("Failure to import shader lib!");
//
//     assert!(!shader_preprocessor.known_imports.is_empty());
// }
