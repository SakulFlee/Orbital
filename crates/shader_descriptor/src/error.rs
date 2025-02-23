#[derive(Debug)]
pub enum Error {
    ShaderPreprocessor(shader_preprocessor::Error),
    Texture(texture_realization::Error),
    IO(std::io::Error),
}
