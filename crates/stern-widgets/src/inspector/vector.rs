use stern_core::Rect;

use super::util::finite_non_negative;

/// Layout tuning for compact vector property fields.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VectorComponentLayout {
    /// Gap between vector component groups.
    pub component_gap: f32,
    /// Width reserved for compact component labels such as X/Y/Z/W.
    pub label_width: f32,
    /// Gap between a compact component label and its value field.
    pub label_gap: f32,
    /// Preferred minimum value-field width before labels are compressed.
    pub min_value_width: f32,
}

impl VectorComponentLayout {
    /// Creates a vector component layout.
    #[must_use]
    pub const fn new(
        component_gap: f32,
        label_width: f32,
        label_gap: f32,
        min_value_width: f32,
    ) -> Self {
        Self {
            component_gap,
            label_width,
            label_gap,
            min_value_width,
        }
    }
}

impl Default for VectorComponentLayout {
    fn default() -> Self {
        Self::new(6.0, 10.0, 3.0, 24.0)
    }
}

/// Rectangles assigned to one vector component.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VectorComponentRect {
    /// Component index.
    pub index: usize,
    /// Compact component label.
    pub label: &'static str,
    /// Full component group rectangle.
    pub rect: Rect,
    /// Compact label rectangle.
    pub label_rect: Rect,
    /// Numeric value field rectangle.
    pub value_rect: Rect,
}

/// Computes deterministic Vec2 component rectangles.
#[must_use]
pub fn vector2_component_rects(
    rect: Rect,
    layout: VectorComponentLayout,
) -> [VectorComponentRect; 2] {
    vector_component_rects(rect, ["X", "Y"], layout)
}

/// Computes deterministic Vec3 component rectangles.
#[must_use]
pub fn vector3_component_rects(
    rect: Rect,
    layout: VectorComponentLayout,
) -> [VectorComponentRect; 3] {
    vector_component_rects(rect, ["X", "Y", "Z"], layout)
}

/// Computes deterministic Vec4 component rectangles.
#[must_use]
pub fn vector4_component_rects(
    rect: Rect,
    layout: VectorComponentLayout,
) -> [VectorComponentRect; 4] {
    vector_component_rects(rect, ["X", "Y", "Z", "W"], layout)
}

#[allow(clippy::cast_precision_loss)]
fn vector_component_rects<const N: usize>(
    rect: Rect,
    labels: [&'static str; N],
    layout: VectorComponentLayout,
) -> [VectorComponentRect; N] {
    let count = N.max(1) as f32;
    let width = finite_non_negative(rect.width);
    let height = finite_non_negative(rect.height);
    let sanitized_component_gap = finite_non_negative(layout.component_gap);
    let total_gap = (sanitized_component_gap * (count - 1.0)).min(width);
    let component_width = (width - total_gap).max(0.0) / count;
    let preferred_label_width = finite_non_negative(layout.label_width);
    let preferred_label_gap = finite_non_negative(layout.label_gap);
    let min_value_width = finite_non_negative(layout.min_value_width);

    std::array::from_fn(|index| {
        let x = rect.x + index as f32 * (component_width + sanitized_component_gap);
        let component_rect = Rect::new(x, rect.y, component_width, height);
        let label_fits =
            component_width >= preferred_label_width + preferred_label_gap + min_value_width;
        let label_width = if label_fits {
            preferred_label_width.min(component_width)
        } else {
            (component_width * 0.35).min(preferred_label_width).max(0.0)
        };
        let label_gap = if component_width > label_width {
            preferred_label_gap.min(component_width - label_width)
        } else {
            0.0
        };
        let value_x = x + label_width + label_gap;
        let value_width = (component_rect.max_x() - value_x).max(0.0);

        VectorComponentRect {
            index,
            label: labels[index],
            rect: component_rect,
            label_rect: Rect::new(x, rect.y, label_width, height),
            value_rect: Rect::new(value_x, rect.y, value_width, height),
        }
    })
}
