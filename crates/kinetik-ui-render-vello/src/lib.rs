//! Vello renderer boundary for Kinetik UI render primitives.

use std::collections::HashSet;

use kinetik_ui_core::{
    Brush, Color, ImageId, LayerId, Primitive, Rect, Size, TextureId, Transform, ViewportInfo,
};
use vello::Scene;

/// Static image resource known by the renderer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImageResource {
    /// Image handle from core primitives.
    pub id: ImageId,
    /// Image size in physical pixels.
    pub size: Size,
}

/// Dynamic texture resource known by the renderer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextureResource {
    /// Texture handle from core primitives.
    pub id: TextureId,
    /// Texture size in physical pixels.
    pub size: Size,
}

/// Resource registry used during primitive translation.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RenderResources {
    images: HashSet<ImageId>,
    textures: HashSet<TextureId>,
}

impl RenderResources {
    /// Creates an empty resource registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers an image resource.
    pub fn register_image(&mut self, image: ImageResource) {
        self.images.insert(image.id);
    }

    /// Registers a texture resource.
    pub fn register_texture(&mut self, texture: TextureResource) {
        self.textures.insert(texture.id);
    }

    /// Returns true when an image is registered.
    #[must_use]
    pub fn has_image(&self, image: ImageId) -> bool {
        self.images.contains(&image)
    }

    /// Returns true when a texture is registered.
    #[must_use]
    pub fn has_texture(&self, texture: TextureId) -> bool {
        self.textures.contains(&texture)
    }
}

/// Input submitted to the renderer for one frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderFrameInput<'a> {
    /// Viewport for the frame.
    pub viewport: ViewportInfo,
    /// Primitive sequence to draw in order.
    pub primitives: &'a [Primitive],
    /// Image and texture resources available to this frame.
    pub resources: &'a RenderResources,
}

/// Output produced by renderer translation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderFrameOutput {
    /// Number of primitives submitted.
    pub primitive_count: usize,
    /// Translation diagnostics.
    pub diagnostics: Vec<RenderDiagnostic>,
}

/// Renderer diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderDiagnostic {
    /// Image resource was referenced but not registered.
    MissingImage(ImageId),
    /// Texture resource was referenced but not registered.
    MissingTexture(TextureId),
    /// Primitive kind is intentionally represented but not yet translated into Vello drawing.
    UnsupportedPrimitive(&'static str),
}

/// Deterministic command produced before backend drawing.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderCommand {
    /// Layer used by the command.
    pub layer: LayerId,
    /// Clip rectangle, when active.
    pub clip: Option<Rect>,
    /// Transform used by the command.
    pub transform: Transform,
    /// Command kind.
    pub kind: RenderCommandKind,
}

/// Command kind produced by primitive translation.
#[derive(Debug, Clone, PartialEq)]
pub enum RenderCommandKind {
    /// Filled and/or stroked rectangle.
    Rect {
        /// Rectangle bounds.
        rect: Rect,
        /// Fill brush.
        fill: Option<Brush>,
        /// Stroke width.
        stroke_width: Option<f32>,
    },
    /// Stroked line.
    Line {
        /// Start x.
        x0: f32,
        /// Start y.
        y0: f32,
        /// End x.
        x1: f32,
        /// End y.
        y1: f32,
    },
    /// Text placeholder command until glyph drawing is wired.
    Text {
        /// Text content.
        text: String,
        /// Text color.
        color: Color,
    },
    /// Image resource draw command.
    Image {
        /// Image resource.
        image: ImageId,
        /// Destination rectangle.
        rect: Rect,
    },
    /// Texture resource draw command.
    Texture {
        /// Texture resource.
        texture: TextureId,
        /// Destination rectangle.
        rect: Rect,
    },
}

/// Vello renderer boundary.
pub struct VelloRenderer {
    scene: Scene,
}

impl VelloRenderer {
    /// Creates a renderer boundary with an empty Vello scene.
    #[must_use]
    pub fn new() -> Self {
        Self {
            scene: Scene::new(),
        }
    }

    /// Returns the current Vello scene.
    #[must_use]
    pub const fn scene(&self) -> &Scene {
        &self.scene
    }

    /// Submits a frame for translation.
    pub fn submit_frame(&mut self, input: RenderFrameInput<'_>) -> RenderFrameOutput {
        let translated = translate_primitives(input.primitives, input.resources);
        RenderFrameOutput {
            primitive_count: input.primitives.len(),
            diagnostics: translated.diagnostics,
        }
    }
}

impl Default for VelloRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Translation result used by tests and renderer internals.
#[derive(Debug, Clone, PartialEq)]
pub struct Translation {
    /// Deterministic commands.
    pub commands: Vec<RenderCommand>,
    /// Translation diagnostics.
    pub diagnostics: Vec<RenderDiagnostic>,
}

/// Translates primitives into deterministic renderer commands.
#[must_use]
pub fn translate_primitives(primitives: &[Primitive], resources: &RenderResources) -> Translation {
    let mut commands = Vec::new();
    let mut diagnostics = Vec::new();
    let mut layer = LayerId::from_raw(0);
    let mut clip = None;
    let mut transform = Transform::IDENTITY;

    for primitive in primitives {
        match primitive {
            Primitive::Rect(rect) => commands.push(RenderCommand {
                layer,
                clip,
                transform,
                kind: RenderCommandKind::Rect {
                    rect: rect.rect,
                    fill: rect.fill,
                    stroke_width: rect.stroke.as_ref().map(|stroke| stroke.width),
                },
            }),
            Primitive::Line(line) => commands.push(RenderCommand {
                layer,
                clip,
                transform,
                kind: RenderCommandKind::Line {
                    x0: line.from.x,
                    y0: line.from.y,
                    x1: line.to.x,
                    y1: line.to.y,
                },
            }),
            Primitive::Text(text) => {
                diagnostics.push(RenderDiagnostic::UnsupportedPrimitive("text_glyphs"));
                commands.push(RenderCommand {
                    layer,
                    clip,
                    transform,
                    kind: RenderCommandKind::Text {
                        text: text.text.clone(),
                        color: brush_color(&text.brush),
                    },
                });
            }
            Primitive::Image(image) => {
                if !resources.has_image(image.image) {
                    diagnostics.push(RenderDiagnostic::MissingImage(image.image));
                }
                commands.push(RenderCommand {
                    layer,
                    clip,
                    transform,
                    kind: RenderCommandKind::Image {
                        image: image.image,
                        rect: image.rect,
                    },
                });
            }
            Primitive::Texture(texture) => {
                if !resources.has_texture(texture.texture) {
                    diagnostics.push(RenderDiagnostic::MissingTexture(texture.texture));
                }
                commands.push(RenderCommand {
                    layer,
                    clip,
                    transform,
                    kind: RenderCommandKind::Texture {
                        texture: texture.texture,
                        rect: texture.rect,
                    },
                });
            }
            Primitive::ClipBegin { rect, .. } => {
                clip = Some(*rect);
            }
            Primitive::ClipEnd { .. } => {
                clip = None;
            }
            Primitive::LayerBegin { id } => {
                layer = *id;
            }
            Primitive::LayerEnd { .. } => {
                layer = LayerId::from_raw(0);
            }
            Primitive::TransformBegin(next_transform) => {
                transform = *next_transform;
            }
            Primitive::TransformEnd => {
                transform = Transform::IDENTITY;
            }
        }
    }

    Translation {
        commands,
        diagnostics,
    }
}

fn brush_color(brush: &Brush) -> Color {
    match brush {
        Brush::Solid(color) => *color,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ImageResource, RenderCommandKind, RenderDiagnostic, RenderFrameInput, RenderResources,
        TextureResource, VelloRenderer, translate_primitives,
    };
    use kinetik_ui_core::render::TexturePrimitive;
    use kinetik_ui_core::{
        Brush, ClipId, Color, CornerRadius, ImageId, ImagePrimitive, LayerId, LinePrimitive, Point,
        Primitive, Rect, RectPrimitive, ScaleFactor, Size, Stroke, TextPrimitive, TextureId,
        Transform, Vec2, ViewportInfo,
    };

    fn resources() -> RenderResources {
        let mut resources = RenderResources::new();
        resources.register_image(ImageResource {
            id: ImageId::from_raw(1),
            size: Size::new(64.0, 64.0),
        });
        resources.register_texture(TextureResource {
            id: TextureId::from_raw(2),
            size: Size::new(128.0, 128.0),
        });
        resources
    }

    #[test]
    fn translates_rectangles_and_lines_in_order() {
        let primitives = vec![
            Primitive::Rect(RectPrimitive {
                rect: Rect::new(0.0, 0.0, 10.0, 10.0),
                fill: Some(Brush::Solid(Color::WHITE)),
                stroke: Some(Stroke::new(1.0, Brush::Solid(Color::BLACK))),
                radius: CornerRadius::all(0.0),
            }),
            Primitive::Line(LinePrimitive {
                from: Point::new(0.0, 0.0),
                to: Point::new(10.0, 10.0),
                stroke: Stroke::new(1.0, Brush::Solid(Color::WHITE)),
            }),
        ];

        let translation = translate_primitives(&primitives, &RenderResources::new());

        assert!(matches!(
            translation.commands[0].kind,
            RenderCommandKind::Rect { .. }
        ));
        assert!(matches!(
            translation.commands[1].kind,
            RenderCommandKind::Line { .. }
        ));
    }

    #[test]
    fn applies_layer_clip_and_transform_to_following_commands() {
        let primitives = vec![
            Primitive::LayerBegin {
                id: LayerId::from_raw(3),
            },
            Primitive::ClipBegin {
                id: ClipId::from_raw(4),
                rect: Rect::new(0.0, 0.0, 20.0, 20.0),
            },
            Primitive::TransformBegin(Transform::translation(Vec2::new(2.0, 3.0))),
            Primitive::Rect(RectPrimitive {
                rect: Rect::new(1.0, 1.0, 4.0, 4.0),
                fill: None,
                stroke: None,
                radius: CornerRadius::all(0.0),
            }),
        ];

        let translation = translate_primitives(&primitives, &RenderResources::new());
        let command = &translation.commands[0];

        assert_eq!(command.layer, LayerId::from_raw(3));
        assert_eq!(command.clip, Some(Rect::new(0.0, 0.0, 20.0, 20.0)));
        assert_eq!(
            command.transform,
            Transform::translation(Vec2::new(2.0, 3.0))
        );
    }

    #[test]
    fn reports_missing_image_and_texture_resources() {
        let primitives = vec![
            Primitive::Image(ImagePrimitive {
                image: ImageId::from_raw(9),
                rect: Rect::new(0.0, 0.0, 10.0, 10.0),
            }),
            Primitive::Texture(TexturePrimitive {
                texture: TextureId::from_raw(8),
                rect: Rect::new(0.0, 0.0, 10.0, 10.0),
                source_size: Size::new(10.0, 10.0),
            }),
        ];

        let translation = translate_primitives(&primitives, &resources());

        assert_eq!(
            translation.diagnostics,
            vec![
                RenderDiagnostic::MissingImage(ImageId::from_raw(9)),
                RenderDiagnostic::MissingTexture(TextureId::from_raw(8)),
            ]
        );
    }

    #[test]
    fn registered_resources_do_not_emit_missing_diagnostics() {
        let primitives = vec![
            Primitive::Image(ImagePrimitive {
                image: ImageId::from_raw(1),
                rect: Rect::new(0.0, 0.0, 10.0, 10.0),
            }),
            Primitive::Texture(TexturePrimitive {
                texture: TextureId::from_raw(2),
                rect: Rect::new(0.0, 0.0, 10.0, 10.0),
                source_size: Size::new(10.0, 10.0),
            }),
        ];

        let translation = translate_primitives(&primitives, &resources());

        assert!(translation.diagnostics.is_empty());
    }

    #[test]
    fn text_translation_reports_glyph_path_as_unsupported() {
        let primitives = vec![Primitive::Text(TextPrimitive {
            origin: Point::new(0.0, 0.0),
            text: "Label".to_owned(),
            size: 12.0,
            brush: Brush::Solid(Color::WHITE),
        })];

        let translation = translate_primitives(&primitives, &RenderResources::new());

        assert_eq!(
            translation.diagnostics,
            vec![RenderDiagnostic::UnsupportedPrimitive("text_glyphs")]
        );
    }

    #[test]
    fn frame_submission_reports_primitive_count_and_diagnostics() {
        let mut renderer = VelloRenderer::new();
        let primitives = vec![Primitive::Image(ImagePrimitive {
            image: ImageId::from_raw(9),
            rect: Rect::new(0.0, 0.0, 10.0, 10.0),
        })];
        let resources = RenderResources::new();
        let output = renderer.submit_frame(RenderFrameInput {
            viewport: ViewportInfo::new(
                Size::new(100.0, 100.0),
                kinetik_ui_core::PhysicalSize::new(100, 100),
                ScaleFactor::ONE,
            ),
            primitives: &primitives,
            resources: &resources,
        });

        assert_eq!(output.primitive_count, 1);
        assert_eq!(
            output.diagnostics,
            vec![RenderDiagnostic::MissingImage(ImageId::from_raw(9))]
        );
    }
}
