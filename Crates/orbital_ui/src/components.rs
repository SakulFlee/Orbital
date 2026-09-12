

/// Marker component for UI entities.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UiEntity;

/// Base component for all UI elements.
#[derive(Debug, Clone)]
pub struct UiElement {
    /// Unique identifier for this UI element.
    pub id: String,
    /// Whether this element is visible.
    pub visible: bool,
    /// Z-index for rendering order (higher = rendered on top).
    pub z_index: i32,
}

impl Default for UiElement {
    fn default() -> Self {
        Self {
            id: String::new(),
            visible: true,
            z_index: 0,
        }
    }
}

impl UiElement {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            visible: true,
            z_index: 0,
        }
    }

    pub fn with_z_index(mut self, z_index: i32) -> Self {
        self.z_index = z_index;
        self
    }
}

/// State of a button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    /// Normal state.
    Normal,
    /// Button is being hovered.
    Hovered,
    /// Button is being pressed.
    Pressed,
}

impl Default for ButtonState {
    fn default() -> Self {
        Self::Normal
    }
}

/// Button widget component.
#[derive(Debug, Clone)]
pub struct UiButton {
    /// Button label text.
    pub label: String,
    /// Current button state.
    pub state: ButtonState,
}

impl UiButton {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            state: ButtonState::Normal,
        }
    }
}

/// Text widget component.
#[derive(Debug, Clone)]
pub struct UiText {
    /// Text content.
    pub content: String,
    /// Font size in pixels.
    pub font_size: f32,
    /// Text color (RGBA).
    pub color: [f32; 4],
}

impl Default for UiText {
    fn default() -> Self {
        Self {
            content: String::new(),
            font_size: 24.0,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

impl UiText {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            ..Default::default()
        }
    }

    pub fn with_font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self
    }

    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }
}

/// TextBox widget component for text input.
#[derive(Debug, Clone)]
pub struct UiTextBox {
    /// Current text value.
    pub value: String,
    /// Placeholder text when empty.
    pub placeholder: String,
    /// Cursor position in characters.
    pub cursor_pos: usize,
    /// Whether this text box has focus.
    pub focused: bool,
}

impl Default for UiTextBox {
    fn default() -> Self {
        Self {
            value: String::new(),
            placeholder: String::new(),
            cursor_pos: 0,
            focused: false,
        }
    }
}

impl UiTextBox {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }
}

/// Checkbox widget component.
#[derive(Debug, Clone)]
pub struct UiCheckbox {
    /// Whether the checkbox is checked.
    pub checked: bool,
    /// Checkbox label.
    pub label: String,
}

impl Default for UiCheckbox {
    fn default() -> Self {
        Self {
            checked: false,
            label: String::new(),
        }
    }
}

impl UiCheckbox {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            checked: false,
            label: label.into(),
        }
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }
}

/// Image widget component.
#[derive(Debug, Clone)]
pub struct UiImage {
    /// Tint color (RGBA).
    pub tint: [f32; 4],
}

impl Default for UiImage {
    fn default() -> Self {
        Self {
            tint: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

impl UiImage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_tint(mut self, tint: [f32; 4]) -> Self {
        self.tint = tint;
        self
    }
}

/// Background component for UI elements.
#[derive(Debug, Clone)]
pub struct UiBackground {
    /// Background color (RGBA).
    pub color: [f32; 4],
    /// Corner radius in pixels.
    pub corner_radius: f32,
}

impl Default for UiBackground {
    fn default() -> Self {
        Self {
            color: [0.2, 0.2, 0.2, 0.8],
            corner_radius: 4.0,
        }
    }
}

impl UiBackground {
    pub fn solid(color: [f32; 4]) -> Self {
        Self {
            color,
            corner_radius: 0.0,
        }
    }

    pub fn rounded(color: [f32; 4], corner_radius: f32) -> Self {
        Self {
            color,
            corner_radius,
        }
    }
}

/// Focus state component for interactive elements.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UiFocus {
    /// Whether this element currently has focus.
    pub focused: bool,
}

/// Resolved layout for a UI element (computed position and size).
#[derive(Debug, Clone, Copy)]
pub struct ResolvedLayout {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl ResolvedLayout {
    /// Returns true if a point is inside this layout.
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_element_default() {
        let elem = UiElement::default();
        assert!(elem.visible);
        assert_eq!(elem.z_index, 0);
    }

    #[test]
    fn ui_button_state() {
        let btn = UiButton::new("Click me");
        assert_eq!(btn.label, "Click me");
        assert_eq!(btn.state, ButtonState::Normal);
    }

    #[test]
    fn ui_text_builder() {
        let text = UiText::new("Hello").with_font_size(32.0).with_color([1.0, 0.0, 0.0, 1.0]);
        assert_eq!(text.content, "Hello");
        assert_eq!(text.font_size, 32.0);
        assert_eq!(text.color, [1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn ui_checkbox_toggle() {
        let cb = UiCheckbox::new("Accept").checked(true);
        assert!(cb.checked);
        assert_eq!(cb.label, "Accept");
    }

    #[test]
    fn resolved_layout_contains() {
        let layout = ResolvedLayout {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 50.0,
        };
        assert!(layout.contains(50.0, 40.0));
        assert!(!layout.contains(5.0, 5.0));
        assert!(!layout.contains(120.0, 40.0));
    }

    #[test]
    fn ui_background() {
        let bg = UiBackground::rounded([0.5, 0.5, 0.5, 1.0], 8.0);
        assert_eq!(bg.corner_radius, 8.0);
    }
}
