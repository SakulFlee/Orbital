#[derive(Debug)]
pub enum ShaderError {
    ShaderPreprocessor(shader_preprocessor::Error),
    Texture(texture::Error),
    IO(std::io::Error),
}
