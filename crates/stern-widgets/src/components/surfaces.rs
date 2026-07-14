use super::{
    ImageId, ImagePrimitive, Insets, LinePrimitive, Point, Primitive, Rect, RectPrimitive, Theme,
    WidgetOutput, pad_rect,
};

/// Emits a passive panel surface.
#[must_use]
pub fn panel(rect: Rect, theme: &Theme) -> WidgetOutput {
    let recipe = theme.panel();
    let mut primitives = Vec::new();
    if let Some(shadow) = recipe.shadow {
        primitives.push(Primitive::Shadow(shadow.primitive(rect)));
    }
    primitives.push(Primitive::Rect(RectPrimitive {
        rect,
        fill: Some(recipe.background),
        stroke: Some(recipe.border),
        radius: recipe.radius,
    }));
    WidgetOutput::new(None, primitives)
}

/// Resolved panel surface and content body rectangles.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PanelFrame {
    /// Full panel surface rectangle.
    pub outer: Rect,
    /// Inner content body after insets.
    pub body: Rect,
}

impl PanelFrame {
    /// Resolves a panel body from an outer rectangle and content insets.
    #[must_use]
    pub fn new(outer: Rect, body_insets: Insets) -> Self {
        Self {
            outer,
            body: pad_rect(outer, body_insets),
        }
    }
}

/// Emits a simple horizontal separator line.
#[must_use]
pub fn separator(rect: Rect, theme: &Theme) -> Primitive {
    let recipe = theme.separator();
    Primitive::Line(LinePrimitive {
        from: Point::new(rect.x, rect.center().y),
        to: Point::new(rect.max_x(), rect.center().y),
        stroke: recipe.stroke,
    })
}

/// Emits an image primitive for a static icon-like resource.
#[must_use]
pub fn image(rect: Rect, image: ImageId) -> WidgetOutput {
    WidgetOutput::new(
        None,
        vec![Primitive::Image(ImagePrimitive {
            image,
            rect,
            tint: None,
        })],
    )
}
