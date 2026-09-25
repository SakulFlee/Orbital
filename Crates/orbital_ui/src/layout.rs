use crate::components::ResolvedLayout;

/// Layout mode for UI elements.
#[derive(Debug, Clone, PartialEq)]
pub enum UiLayout {
    /// Absolute positioning with fixed position and size.
    Absolute {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
    /// Horizontal layout - children are laid out left to right.
    Horizontal {
        /// Spacing between children in pixels.
        spacing: f32,
        /// Vertical alignment within the container.
        alignment: Alignment,
    },
    /// Vertical layout - children are laid out top to bottom.
    Vertical {
        /// Spacing between children in pixels.
        spacing: f32,
        /// Horizontal alignment within the container.
        alignment: Alignment,
    },
}

impl Default for UiLayout {
    fn default() -> Self {
        Self::Absolute {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 30.0,
        }
    }
}

/// Alignment within a layout container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Alignment {
    /// Align to the start (left for horizontal, top for vertical).
    #[default]
    Start,
    /// Align to the center.
    Center,
    /// Align to the end (right for horizontal, bottom for vertical).
    End,
}

/// A container for child layouts.
#[derive(Debug, Clone)]
pub struct UiContainer {
    /// The layout of this container.
    pub layout: UiLayout,
    /// Resolved position and size (computed during layout pass).
    pub resolved: ResolvedLayout,
}

impl UiContainer {
    pub fn new(layout: UiLayout) -> Self {
        Self {
            layout,
            resolved: ResolvedLayout {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            },
        }
    }
}

/// Resolves a horizontal layout for children.
pub fn resolve_horizontal(
    container_x: f32,
    container_y: f32,
    container_height: f32,
    children: &[(f32, f32)], // (width, height) of each child
    spacing: f32,
    alignment: Alignment,
) -> Vec<ResolvedLayout> {
    let mut layouts = Vec::with_capacity(children.len());
    let mut current_x = container_x;

    for &(child_width, child_height) in children {
        let y = match alignment {
            Alignment::Start => container_y,
            Alignment::Center => container_y + (container_height - child_height) / 2.0,
            Alignment::End => container_y + container_height - child_height,
        };

        layouts.push(ResolvedLayout {
            x: current_x,
            y,
            width: child_width,
            height: child_height,
        });

        current_x += child_width + spacing;
    }

    layouts
}

/// Resolves a vertical layout for children.
pub fn resolve_vertical(
    container_x: f32,
    container_y: f32,
    container_width: f32,
    children: &[(f32, f32)], // (width, height) of each child
    spacing: f32,
    alignment: Alignment,
) -> Vec<ResolvedLayout> {
    let mut layouts = Vec::with_capacity(children.len());
    let mut current_y = container_y;

    for &(child_width, child_height) in children {
        let x = match alignment {
            Alignment::Start => container_x,
            Alignment::Center => container_x + (container_width - child_width) / 2.0,
            Alignment::End => container_x + container_width - child_width,
        };

        layouts.push(ResolvedLayout {
            x,
            y: current_y,
            width: child_width,
            height: child_height,
        });

        current_y += child_height + spacing;
    }

    layouts
}

/// Computes the total size needed for a horizontal layout.
pub fn horizontal_size(children: &[(f32, f32)], spacing: f32) -> (f32, f32) {
    let mut total_width: f32 = 0.0;
    let mut max_height: f32 = 0.0;

    for &(w, h) in children {
        total_width += w;
        max_height = max_height.max(h);
    }

    if !children.is_empty() {
        total_width += spacing * (children.len() - 1) as f32;
    }

    (total_width, max_height)
}

/// Computes the total size needed for a vertical layout.
pub fn vertical_size(children: &[(f32, f32)], spacing: f32) -> (f32, f32) {
    let mut max_width: f32 = 0.0;
    let mut total_height: f32 = 0.0;

    for &(w, h) in children {
        max_width = max_width.max(w);
        total_height += h;
    }

    if !children.is_empty() {
        total_height += spacing * (children.len() - 1) as f32;
    }

    (max_width, total_height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn horizontal_layout_basic() {
        let children = [(100.0, 30.0), (80.0, 30.0), (60.0, 30.0)];
        let layouts = resolve_horizontal(0.0, 0.0, 50.0, &children, 10.0, Alignment::Start);

        assert_eq!(layouts.len(), 3);
        assert_eq!(layouts[0].x, 0.0);
        assert_eq!(layouts[1].x, 110.0); // 100 + 10 spacing
        assert_eq!(layouts[2].x, 200.0); // 100 + 10 + 80 + 10
    }

    #[test]
    fn horizontal_layout_centered() {
        let children = [(100.0, 20.0)];
        let layouts = resolve_horizontal(0.0, 0.0, 40.0, &children, 10.0, Alignment::Center);

        assert_eq!(layouts[0].y, 10.0); // (40 - 20) / 2
    }

    #[test]
    fn vertical_layout_basic() {
        let children = [(100.0, 30.0), (100.0, 20.0), (100.0, 40.0)];
        let layouts = resolve_vertical(0.0, 0.0, 150.0, &children, 5.0, Alignment::Start);

        assert_eq!(layouts.len(), 3);
        assert_eq!(layouts[0].y, 0.0);
        assert_eq!(layouts[1].y, 35.0); // 30 + 5 spacing
        assert_eq!(layouts[2].y, 60.0); // 30 + 5 + 20 + 5
    }

    #[test]
    fn horizontal_size_calc() {
        let children = [(100.0, 30.0), (80.0, 40.0)];
        let (w, h) = horizontal_size(&children, 10.0);
        assert_eq!(w, 190.0); // 100 + 10 + 80
        assert_eq!(h, 40.0);
    }

    #[test]
    fn vertical_size_calc() {
        let children = [(100.0, 30.0), (80.0, 40.0)];
        let (w, h) = vertical_size(&children, 10.0);
        assert_eq!(w, 100.0);
        assert_eq!(h, 80.0); // 30 + 10 + 40
    }
}
