//! Backend-independent render primitives.

use crate::{Point, Rect, Size, Vec2};

/// RGBA color in linear toolkit space.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Color {
    /// Red channel.
    pub r: f32,
    /// Green channel.
    pub g: f32,
    /// Blue channel.
    pub b: f32,
    /// Alpha channel.
    pub a: f32,
}

impl Color {
    /// Transparent black.
    pub const TRANSPARENT: Self = Self::rgba(0.0, 0.0, 0.0, 0.0);
    /// Opaque black.
    pub const BLACK: Self = Self::rgba(0.0, 0.0, 0.0, 1.0);
    /// Opaque white.
    pub const WHITE: Self = Self::rgba(1.0, 1.0, 1.0, 1.0);

    /// Creates a color from RGBA channels.
    #[must_use]
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Creates an opaque color from RGB channels.
    #[must_use]
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::rgba(r, g, b, 1.0)
    }

    /// Returns this color with a replaced alpha channel.
    #[must_use]
    pub const fn with_alpha(self, a: f32) -> Self {
        Self { a, ..self }
    }
}

/// Fill/stroke brush.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Brush {
    /// Solid color brush.
    Solid(Color),
}

/// Stroke style.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stroke {
    /// Stroke width in logical units.
    pub width: f32,
    /// Stroke brush.
    pub brush: Brush,
}

impl Stroke {
    /// Creates a stroke.
    #[must_use]
    pub const fn new(width: f32, brush: Brush) -> Self {
        Self { width, brush }
    }
}

/// Corner radii for rounded rectangles.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CornerRadius {
    /// Top-left radius.
    pub top_left: f32,
    /// Top-right radius.
    pub top_right: f32,
    /// Bottom-right radius.
    pub bottom_right: f32,
    /// Bottom-left radius.
    pub bottom_left: f32,
}

impl CornerRadius {
    /// Creates equal corner radii.
    #[must_use]
    pub const fn all(value: f32) -> Self {
        Self {
            top_left: value,
            top_right: value,
            bottom_right: value,
            bottom_left: value,
        }
    }
}

/// Static image resource handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ImageId(u64);

impl ImageId {
    /// Creates an image ID from raw bits.
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns raw ID bits.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// GPU-resident texture surface handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TextureId(u64);

impl TextureId {
    /// Creates a texture ID from raw bits.
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns raw ID bits.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// Clip command identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ClipId(u64);

impl ClipId {
    /// Creates a clip ID from raw bits.
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }
}

/// Layer command identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LayerId(u64);

impl LayerId {
    /// Creates a layer ID from raw bits.
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }
}

/// 2D affine transform matrix.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    /// Scale/skew x component.
    pub m11: f32,
    /// Skew y component.
    pub m12: f32,
    /// Skew x component.
    pub m21: f32,
    /// Scale/skew y component.
    pub m22: f32,
    /// Translation x.
    pub dx: f32,
    /// Translation y.
    pub dy: f32,
}

impl Transform {
    /// Identity transform.
    pub const IDENTITY: Self = Self {
        m11: 1.0,
        m12: 0.0,
        m21: 0.0,
        m22: 1.0,
        dx: 0.0,
        dy: 0.0,
    };

    /// Creates a translation transform.
    #[must_use]
    pub const fn translation(offset: Vec2) -> Self {
        Self {
            dx: offset.x,
            dy: offset.y,
            ..Self::IDENTITY
        }
    }
}

/// Rectangle draw command.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RectPrimitive {
    /// Rectangle bounds.
    pub rect: Rect,
    /// Fill brush.
    pub fill: Option<Brush>,
    /// Stroke style.
    pub stroke: Option<Stroke>,
    /// Corner radii.
    pub radius: CornerRadius,
}

/// Line draw command.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinePrimitive {
    /// Start point.
    pub from: Point,
    /// End point.
    pub to: Point,
    /// Stroke style.
    pub stroke: Stroke,
}

/// Text draw command.
#[derive(Debug, Clone, PartialEq)]
pub struct TextPrimitive {
    /// Text baseline origin.
    pub origin: Point,
    /// Text content.
    pub text: String,
    /// Font size in logical units.
    pub size: f32,
    /// Fill brush.
    pub brush: Brush,
}

/// Static image draw command.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImagePrimitive {
    /// Image handle.
    pub image: ImageId,
    /// Destination rectangle.
    pub rect: Rect,
}

/// Texture draw command.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TexturePrimitive {
    /// Texture handle.
    pub texture: TextureId,
    /// Destination rectangle.
    pub rect: Rect,
    /// Source size in texture pixels.
    pub source_size: Size,
}

/// Backend-independent draw command.
#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
    /// Rectangle or rounded rectangle.
    Rect(RectPrimitive),
    /// Straight line.
    Line(LinePrimitive),
    /// Text.
    Text(TextPrimitive),
    /// Static image.
    Image(ImagePrimitive),
    /// GPU texture surface.
    Texture(TexturePrimitive),
    /// Begin rectangular clipping.
    ClipBegin {
        /// Clip command identity.
        id: ClipId,
        /// Clip rectangle.
        rect: Rect,
    },
    /// End clipping.
    ClipEnd {
        /// Clip command identity.
        id: ClipId,
    },
    /// Begin layer.
    LayerBegin {
        /// Layer command identity.
        id: LayerId,
    },
    /// End layer.
    LayerEnd {
        /// Layer command identity.
        id: LayerId,
    },
    /// Begin transform.
    TransformBegin(Transform),
    /// End transform.
    TransformEnd,
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::{
        Brush, ClipId, Color, CornerRadius, ImageId, ImagePrimitive, LayerId, LinePrimitive,
        Primitive, RectPrimitive, Stroke, TextPrimitive, TextureId, TexturePrimitive, Transform,
    };
    use crate::{Point, Rect, Size, Vec2};

    #[test]
    fn constructs_color_and_brush_values() {
        let color = Color::rgb(0.1, 0.2, 0.3).with_alpha(0.4);

        assert_eq!(color, Color::rgba(0.1, 0.2, 0.3, 0.4));
        assert_eq!(Brush::Solid(color), Brush::Solid(color));
    }

    #[test]
    fn constructs_stroke_and_radius_values() {
        let stroke = Stroke::new(1.5, Brush::Solid(Color::WHITE));

        assert_eq!(stroke.width, 1.5);
        assert_eq!(CornerRadius::all(4.0).top_left, 4.0);
    }

    #[test]
    fn resource_handles_are_stable() {
        assert_eq!(ImageId::from_raw(7).raw(), 7);
        assert_eq!(TextureId::from_raw(9).raw(), 9);
        assert_ne!(ImageId::from_raw(7).raw(), TextureId::from_raw(9).raw());
    }

    #[test]
    fn creates_translation_transform() {
        let transform = Transform::translation(Vec2::new(3.0, 4.0));

        assert_eq!(transform.dx, 3.0);
        assert_eq!(transform.dy, 4.0);
        assert_eq!(transform.m11, 1.0);
    }

    #[test]
    fn primitive_sequence_preserves_order() {
        let primitives = [
            Primitive::LayerBegin {
                id: LayerId::from_raw(1),
            },
            Primitive::ClipBegin {
                id: ClipId::from_raw(2),
                rect: Rect::new(0.0, 0.0, 10.0, 10.0),
            },
            Primitive::Rect(RectPrimitive {
                rect: Rect::new(1.0, 1.0, 8.0, 8.0),
                fill: Some(Brush::Solid(Color::BLACK)),
                stroke: None,
                radius: CornerRadius::all(2.0),
            }),
            Primitive::ClipEnd {
                id: ClipId::from_raw(2),
            },
            Primitive::LayerEnd {
                id: LayerId::from_raw(1),
            },
        ];

        assert!(matches!(primitives[0], Primitive::LayerBegin { .. }));
        assert!(matches!(primitives[1], Primitive::ClipBegin { .. }));
        assert!(matches!(primitives[2], Primitive::Rect(_)));
        assert!(matches!(primitives[3], Primitive::ClipEnd { .. }));
        assert!(matches!(primitives[4], Primitive::LayerEnd { .. }));
    }

    #[test]
    fn creates_text_image_texture_and_line_primitives() {
        let stroke = Stroke::new(1.0, Brush::Solid(Color::WHITE));

        let line = Primitive::Line(LinePrimitive {
            from: Point::new(0.0, 0.0),
            to: Point::new(1.0, 1.0),
            stroke,
        });
        let text = Primitive::Text(TextPrimitive {
            origin: Point::new(1.0, 2.0),
            text: "Label".to_owned(),
            size: 12.0,
            brush: Brush::Solid(Color::WHITE),
        });
        let image = Primitive::Image(ImagePrimitive {
            image: ImageId::from_raw(1),
            rect: Rect::new(0.0, 0.0, 10.0, 10.0),
        });
        let texture = Primitive::Texture(TexturePrimitive {
            texture: TextureId::from_raw(2),
            rect: Rect::new(0.0, 0.0, 20.0, 10.0),
            source_size: Size::new(1920.0, 1080.0),
        });

        assert!(matches!(line, Primitive::Line(_)));
        assert!(matches!(text, Primitive::Text(_)));
        assert!(matches!(image, Primitive::Image(_)));
        assert!(matches!(texture, Primitive::Texture(_)));
    }
}
