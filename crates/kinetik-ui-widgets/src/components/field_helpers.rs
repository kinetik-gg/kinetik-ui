use super::{
    Brush, Point, Primitive, Rect, TextFieldRecipe, TextPrimitive, Theme, control_text_origin,
};

pub(super) fn field_text_primitive(
    rect: Rect,
    text: impl Into<String>,
    recipe: &TextFieldRecipe,
    theme: &Theme,
) -> Primitive {
    Primitive::Text(TextPrimitive {
        layout: None,
        origin: Point::new(rect.x, control_text_origin(rect, theme).y),
        text: text.into(),
        family: recipe.font.family.to_owned(),
        size: recipe.font.size,
        line_height: recipe.font.line_height,
        brush: Brush::Solid(recipe.foreground),
    })
}

pub(super) fn finite_widget_extent(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}
