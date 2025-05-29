use crate::resources::Texture;

pub struct ModelRenderer {
    depth_texture: Texture,
}

impl ModelRenderer {
    pub fn new(depth_texture: Texture) -> Self {
        Self { depth_texture }
    }
}
