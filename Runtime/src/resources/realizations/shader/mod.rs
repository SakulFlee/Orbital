use std::fs;

use phf_macros::phf_map;
use wgpu::{ShaderModule, ShaderModuleDescriptor};

use crate::{
    error::Error,
    resources::{ShaderDescriptor, ShaderSource},
    runtime::Context,
};

/// Defines the import statement a line in the shader has to start with
/// to be understood as an import. This expression is followed by a
/// shader library entry (see below), or, a file that has to be read.
const IMPORT_STATEMENT: &'static str = "#import ";

/// Compile-Time Lookup table using PHF (Perfect Hash Function).
/// This acts as our shader library. Any file added to the shader library
/// needs to be listed here with an identifier. Said identifier should be
/// lowercase for this to work.
///
/// A given shader then can import one of those library functions by
/// the identifier. For example, for the "pbr" entry:
/// ```wgsl
/// #import pbr
/// ```
/// > Assuming `IMPORT_STATEMENT` (above) wasn't changed.
///
/// The full syntax is:
/// ```wgsl
/// // Shader lib import:
/// <IMPORT_STATEMENT> <SHADER_LIB_IMPORT>
///
/// File import:
/// <IMPORT_STATEMENT> <FILE_IMPORT>
///
/// // --- Assuming IMPORT_STATEMENT is "#import "
///
/// // Import the PBR shader library:
/// #import pbr
///
/// // Import the Vertex Data structure from shader lib:
/// #import vertex_data
///
/// // Import another shader file you wrote:
/// #import my_other_shader.wgsl
///
/// // Importing another shader file in another location:
/// #import ../SomeOtherFolder/some_other_shader.wgsl
/// ```
static SHADER_LIB: phf::Map<&'static str, &'static str> = phf_map! {
    "pbr" => include_str!("pbr.wgsl"),
    "vertex_data" => include_str!("vertex_data.wgsl"),
    "fragment_data" => include_str!("fragment_data.wgsl"),
};

pub struct Shader {
    identifier: String,
    shader_module: ShaderModule,
}

impl Shader {
    pub fn from_descriptor(
        shader_descriptor: &ShaderDescriptor,
        context: &Context,
    ) -> Result<Self, Error> {
        let source = match shader_descriptor.source {
            // If our source is a file, we can simply let the resolver function
            // figure everything out. Both methods call each other anyways.
            ShaderSource::FromFile(file) => Self::resolve_shader_import(file),
            // If our source is a string already (e.g. included with
            // include_str!), we just parse it normally.
            ShaderSource::FromSourceString(source) => Self::parse_shader_source_string(source),
        }?;

        let shader_module = context
            .device()
            .create_shader_module(ShaderModuleDescriptor {
                label: Some(&shader_descriptor.identifier),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            });

        Ok(Self {
            identifier: shader_descriptor.identifier.clone(),
            shader_module,
        })
    }

    pub fn parse_shader_source_string(shader_source: &str) -> Result<String, Error> {
        let mut result = String::new();
        let mut to_be_imported = Vec::<String>::new();

        // Read through the provided source code and look for IMPORT_STATEMENT's
        for line in shader_source.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with(IMPORT_STATEMENT) {
                // If an IMPORT_STATEMENT was found, remove the import part
                // and make it lower case as the PHF map (see above) is
                // expecting shader library imports to be always lowercase!
                let import = trimmed.replace(IMPORT_STATEMENT, "").to_lowercase();
                // ... and add the to be imported file to our list:
                to_be_imported.push(import);
            } else {
                // Non-matches simply get added to our result
                result.push_str(&format!("{line}\n"));
            }
        }

        // Now, try resolving every import file.
        // This _may_ call this function in a loop and **CAN** lead to
        // import cycles (circular dependency calls)!
        for import in to_be_imported {
            let resolved_import = Self::resolve_shader_import(&import)?;
            result = format!("{resolved_import}\n{result}");
        }

        Ok(result)
    }

    pub fn resolve_shader_import(import_statement: &str) -> Result<String, Error> {
        // Check if the import source is in our PHF map (see above)
        let import_source = if SHADER_LIB.contains_key(import_statement) {
            // We found a shader library import!
            // Simply reference the PHF map entry ...
            SHADER_LIB[import_statement].to_string()
        } else {
            // Otherwise, we need to try and read a file
            // This only works if the file exists and can be found of course!
            fs::read_to_string(import_statement).map_err(|e| Error::IOError(e))?
        };

        // Now parse the source code to check for further imports and return
        Ok(Self::parse_shader_source_string(&import_source)?)
    }

    pub fn from_existing<S: Into<String>>(identifier: S, shader_module: ShaderModule) -> Self {
        Self {
            identifier: identifier.into(),
            shader_module,
        }
    }

    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub fn shader_module(&self) -> &ShaderModule {
        &self.shader_module
    }
}
