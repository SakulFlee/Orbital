#[derive(Debug)]
pub enum Error {
    ShaderPreprocessor(shader_preprocessor::Error),
    Texture(texture::Error),
    IO(std::io::Error),
}
