//! Render layer system for configuring rendering order.
//!
//! Provides a layered rendering approach where different types of content
//! (3D scene, 2D world, UI, debug) can be rendered in a configurable order.

use std::fmt;

/// Defines the rendering order for different types of content.
///
/// Layers are rendered in the order they are defined (lower index = rendered first).
/// The default order is:
/// 1. Skybox
/// 2. 3D Scene (opaque)
/// 3. 3D Scene (transparent) - reserved for future use
/// 4. 2D World (game sprites, tiled maps)
/// 5. 3D Overlays (name tags, health bars in world-space)
/// 6. UI (screen-space HUD, menus)
/// 7. Debug (debug visualization)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RenderLayer {
    /// Background skybox rendering (always first).
    Skybox = 0,
    /// Main 3D scene rendering (opaque models).
    Scene3D = 1,
    /// 3D scene transparent objects (reserved for future use).
    Scene3DTransparent = 2,
    /// 2D world rendering (sprites, tile maps, game graphics).
    World2D = 3,
    /// 3D overlay rendering (name tags, health bars in world-space).
    Overlay3D = 4,
    /// Screen-space UI rendering (HUD, menus).
    UI = 5,
    /// Debug visualization rendering (always last).
    Debug = 6,
}

impl RenderLayer {
    /// Returns all layers in their default order.
    pub fn all() -> &'static [RenderLayer] {
        &[
            Self::Skybox,
            Self::Scene3D,
            Self::Scene3DTransparent,
            Self::World2D,
            Self::Overlay3D,
            Self::UI,
            Self::Debug,
        ]
    }

    /// Returns the index of this layer (lower = rendered first).
    pub fn index(&self) -> usize {
        *self as usize
    }

    /// Returns the name of this layer.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Skybox => "Skybox",
            Self::Scene3D => "Scene3D",
            Self::Scene3DTransparent => "Scene3DTransparent",
            Self::World2D => "World2D",
            Self::Overlay3D => "Overlay3D",
            Self::UI => "UI",
            Self::Debug => "Debug",
        }
    }

    /// Returns true if this layer should use depth testing.
    pub fn uses_depth_test(&self) -> bool {
        matches!(
            self,
            Self::Scene3D | Self::Scene3DTransparent | Self::Overlay3D
        )
    }

    /// Returns true if this layer should use alpha blending.
    pub fn uses_alpha_blend(&self) -> bool {
        matches!(
            self,
            Self::World2D | Self::Overlay3D | Self::UI | Self::Debug
        )
    }

    /// Returns true if this layer should clear the render target.
    pub fn clears_target(&self) -> bool {
        matches!(self, Self::Skybox)
    }
}

impl fmt::Display for RenderLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Configuration for which layers are enabled and their rendering order.
#[derive(Debug, Clone)]
pub struct LayerConfig {
    /// Ordered list of enabled layers.
    layers: Vec<RenderLayer>,
}

impl Default for LayerConfig {
    fn default() -> Self {
        Self {
            layers: RenderLayer::all().to_vec(),
        }
    }
}

impl LayerConfig {
    /// Creates a new layer configuration with all layers enabled.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a configuration with only the specified layers enabled.
    pub fn with_layers(layers: Vec<RenderLayer>) -> Self {
        let mut sorted = layers;
        sorted.sort_by_key(|l| l.index());
        Self { layers: sorted }
    }

    /// Enables a layer.
    pub fn enable(&mut self, layer: RenderLayer) {
        if !self.layers.contains(&layer) {
            self.layers.push(layer);
            self.layers.sort_by_key(|l| l.index());
        }
    }

    /// Disables a layer.
    pub fn disable(&mut self, layer: RenderLayer) {
        self.layers.retain(|l| *l != layer);
    }

    /// Returns true if a layer is enabled.
    pub fn is_enabled(&self, layer: RenderLayer) -> bool {
        self.layers.contains(&layer)
    }

    /// Returns an iterator over enabled layers in rendering order.
    pub fn iter(&self) -> impl Iterator<Item = &RenderLayer> {
        self.layers.iter()
    }

    /// Returns the number of enabled layers.
    pub fn len(&self) -> usize {
        self.layers.len()
    }

    /// Returns true if no layers are enabled.
    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_order() {
        assert!(RenderLayer::Skybox < RenderLayer::Scene3D);
        assert!(RenderLayer::Scene3D < RenderLayer::World2D);
        assert!(RenderLayer::World2D < RenderLayer::UI);
        assert!(RenderLayer::UI < RenderLayer::Debug);
    }

    #[test]
    fn layer_index() {
        assert_eq!(RenderLayer::Skybox.index(), 0);
        assert_eq!(RenderLayer::Scene3D.index(), 1);
        assert_eq!(RenderLayer::Debug.index(), 6);
    }

    #[test]
    fn layer_depth_test() {
        assert!(RenderLayer::Scene3D.uses_depth_test());
        assert!(!RenderLayer::World2D.uses_depth_test());
        assert!(!RenderLayer::UI.uses_depth_test());
    }

    #[test]
    fn layer_alpha_blend() {
        assert!(!RenderLayer::Scene3D.uses_alpha_blend());
        assert!(RenderLayer::World2D.uses_alpha_blend());
        assert!(RenderLayer::UI.uses_alpha_blend());
    }

    #[test]
    fn layer_clear() {
        assert!(RenderLayer::Skybox.clears_target());
        assert!(!RenderLayer::Scene3D.clears_target());
    }

    #[test]
    fn default_config_has_all_layers() {
        let config = LayerConfig::default();
        assert_eq!(config.len(), 7);
        assert!(config.is_enabled(RenderLayer::Skybox));
        assert!(config.is_enabled(RenderLayer::Debug));
    }

    #[test]
    fn config_enable_disable() {
        let mut config = LayerConfig::default();

        config.disable(RenderLayer::Debug);
        assert!(!config.is_enabled(RenderLayer::Debug));
        assert_eq!(config.len(), 6);

        config.enable(RenderLayer::Debug);
        assert!(config.is_enabled(RenderLayer::Debug));
        assert_eq!(config.len(), 7);
    }

    #[test]
    fn config_with_layers() {
        let config = LayerConfig::with_layers(vec![RenderLayer::UI, RenderLayer::Scene3D]);
        assert_eq!(config.len(), 2);
        assert!(config.is_enabled(RenderLayer::Scene3D));
        assert!(config.is_enabled(RenderLayer::UI));
        assert!(!config.is_enabled(RenderLayer::Debug));
    }

    #[test]
    fn config_iter_order() {
        let config = LayerConfig::with_layers(vec![RenderLayer::UI, RenderLayer::Skybox]);
        let layers: Vec<_> = config.iter().collect();
        assert_eq!(layers[0], &RenderLayer::Skybox);
        assert_eq!(layers[1], &RenderLayer::UI);
    }
}
