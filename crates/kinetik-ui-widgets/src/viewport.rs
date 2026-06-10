//! Viewport texture surfaces and editor overlay primitives.

use kinetik_ui_core::{
    Brush, ClipId, Color, LinePrimitive, Point, Primitive, Rect, Size, Stroke, TextPrimitive,
    TextureId, TexturePrimitive, Vec2,
};

/// How viewport content should fit inside its bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewportFit {
    /// Preserve aspect ratio and fit entire content.
    Fit,
    /// Preserve source pixel size in logical units.
    ActualSize,
    /// Use a custom zoom factor.
    Zoom,
}

/// Pan and zoom state for viewport content.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PanZoom {
    /// Current fit mode.
    pub fit: ViewportFit,
    /// Custom zoom factor.
    pub zoom: f32,
    /// Pan offset in logical units.
    pub pan: Vec2,
}

impl Default for PanZoom {
    fn default() -> Self {
        Self {
            fit: ViewportFit::Fit,
            zoom: 1.0,
            pan: Vec2::ZERO,
        }
    }
}

impl PanZoom {
    /// Sets fit mode.
    pub fn fit(&mut self) {
        self.fit = ViewportFit::Fit;
    }

    /// Sets 100% mode.
    pub fn actual_size(&mut self) {
        self.fit = ViewportFit::ActualSize;
        self.zoom = 1.0;
    }

    /// Sets custom zoom.
    pub fn set_zoom(&mut self, zoom: f32) {
        self.fit = ViewportFit::Zoom;
        self.zoom = zoom.max(0.01);
    }

    /// Adds a pan delta.
    pub fn pan_by(&mut self, delta: Vec2) {
        self.pan = Vec2::new(self.pan.x + delta.x, self.pan.y + delta.y);
    }
}

/// UI-managed viewport surface backed by an application-owned texture.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportSurface {
    /// Texture to display.
    pub texture: TextureId,
    /// Source content size.
    pub source_size: Size,
    /// Viewport bounds.
    pub bounds: Rect,
    /// Pan and zoom state.
    pub pan_zoom: PanZoom,
}

impl ViewportSurface {
    /// Computes the destination rectangle for the texture.
    #[must_use]
    pub fn content_rect(self) -> Rect {
        let scale = match self.pan_zoom.fit {
            ViewportFit::Fit => fit_scale(self.source_size, self.bounds.size()),
            ViewportFit::ActualSize => 1.0,
            ViewportFit::Zoom => self.pan_zoom.zoom,
        };
        let width = self.source_size.width * scale;
        let height = self.source_size.height * scale;
        Rect::new(
            self.bounds.x + (self.bounds.width - width) * 0.5 + self.pan_zoom.pan.x,
            self.bounds.y + (self.bounds.height - height) * 0.5 + self.pan_zoom.pan.y,
            width,
            height,
        )
    }

    /// Emits the texture primitive.
    #[must_use]
    pub fn texture_primitive(self) -> Primitive {
        Primitive::Texture(TexturePrimitive {
            texture: self.texture,
            rect: self.content_rect(),
            source_size: self.source_size,
        })
    }
}

fn fit_scale(source: Size, bounds: Size) -> f32 {
    if source.width <= 0.0 || source.height <= 0.0 || bounds.width <= 0.0 || bounds.height <= 0.0 {
        return 0.0;
    }
    (bounds.width / source.width).min(bounds.height / source.height)
}

/// Viewport guide line.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Guide {
    /// Horizontal guide at y.
    Horizontal(f32),
    /// Vertical guide at x.
    Vertical(f32),
}

/// Computes ruler tick positions.
#[must_use]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
pub fn ruler_ticks(start: f32, end: f32, zoom: f32) -> Vec<f32> {
    let span = (end - start).abs();
    if span <= 0.0 {
        return Vec::new();
    }
    let step = if zoom >= 2.0 {
        10.0
    } else if zoom >= 1.0 {
        25.0
    } else {
        50.0
    };
    let first = (start / step).floor() as i32;
    let last = (end / step).ceil() as i32;
    (first..=last).map(|index| index as f32 * step).collect()
}

/// Emits guide line primitives.
#[must_use]
pub fn guide_primitives(bounds: Rect, guides: &[Guide], color: Color) -> Vec<Primitive> {
    guides
        .iter()
        .map(|guide| match *guide {
            Guide::Horizontal(y) => Primitive::Line(LinePrimitive {
                from: Point::new(bounds.x, y),
                to: Point::new(bounds.max_x(), y),
                stroke: Stroke::new(1.0, Brush::Solid(color)),
            }),
            Guide::Vertical(x) => Primitive::Line(LinePrimitive {
                from: Point::new(x, bounds.y),
                to: Point::new(x, bounds.max_y()),
                stroke: Stroke::new(1.0, Brush::Solid(color)),
            }),
        })
        .collect()
}

/// Crosshair overlay state.
#[derive(Debug, Clone, PartialEq)]
pub struct Crosshair {
    /// Whether the crosshair is visible.
    pub visible: bool,
    /// Cursor position.
    pub position: Point,
    /// Optional label.
    pub label: Option<String>,
    /// Crosshair color.
    pub color: Color,
}

impl Crosshair {
    /// Emits crosshair primitives.
    #[must_use]
    pub fn primitives(&self, bounds: Rect) -> Vec<Primitive> {
        if !self.visible || !bounds.contains_point(self.position) {
            return Vec::new();
        }
        let mut primitives = vec![
            Primitive::Line(LinePrimitive {
                from: Point::new(bounds.x, self.position.y),
                to: Point::new(bounds.max_x(), self.position.y),
                stroke: Stroke::new(1.0, Brush::Solid(self.color)),
            }),
            Primitive::Line(LinePrimitive {
                from: Point::new(self.position.x, bounds.y),
                to: Point::new(self.position.x, bounds.max_y()),
                stroke: Stroke::new(1.0, Brush::Solid(self.color)),
            }),
        ];
        if let Some(label) = &self.label {
            primitives.push(Primitive::Text(TextPrimitive {
                origin: Point::new(self.position.x + 6.0, self.position.y - 6.0),
                text: label.clone(),
                size: 11.0,
                brush: Brush::Solid(self.color),
            }));
        }
        primitives
    }
}

/// Viewport overlay composition request.
#[derive(Debug, Clone, PartialEq)]
pub struct ViewportComposition {
    /// Surface.
    pub surface: ViewportSurface,
    /// Guides.
    pub guides: Vec<Guide>,
    /// Crosshair.
    pub crosshair: Option<Crosshair>,
    /// Clip identity.
    pub clip: ClipId,
}

impl ViewportComposition {
    /// Emits primitives in deterministic viewport order.
    #[must_use]
    pub fn primitives(&self) -> Vec<Primitive> {
        let mut primitives = vec![
            Primitive::ClipBegin {
                id: self.clip,
                rect: self.surface.bounds,
            },
            self.surface.texture_primitive(),
        ];
        primitives.extend(guide_primitives(
            self.surface.bounds,
            &self.guides,
            Color::rgba(1.0, 1.0, 1.0, 0.35),
        ));
        if let Some(crosshair) = &self.crosshair {
            primitives.extend(crosshair.primitives(self.surface.bounds));
        }
        primitives.push(Primitive::ClipEnd { id: self.clip });
        primitives
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Crosshair, Guide, PanZoom, ViewportComposition, ViewportFit, ViewportSurface,
        guide_primitives, ruler_ticks,
    };
    use kinetik_ui_core::{ClipId, Color, Point, Primitive, Rect, Size, TextureId, Vec2};

    fn surface() -> ViewportSurface {
        ViewportSurface {
            texture: TextureId::from_raw(1),
            source_size: Size::new(400.0, 200.0),
            bounds: Rect::new(0.0, 0.0, 200.0, 200.0),
            pan_zoom: PanZoom::default(),
        }
    }

    #[test]
    fn fit_mode_preserves_aspect_ratio() {
        let rect = surface().content_rect();

        assert!((rect.width - 200.0).abs() < f32::EPSILON);
        assert!((rect.height - 100.0).abs() < f32::EPSILON);
        assert!((rect.y - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn pan_zoom_supports_actual_size_custom_zoom_and_pan() {
        let mut surface = surface();
        surface.pan_zoom.actual_size();
        assert!((surface.content_rect().width - 400.0).abs() < f32::EPSILON);

        surface.pan_zoom.set_zoom(0.5);
        surface.pan_zoom.pan_by(Vec2::new(10.0, 5.0));
        let rect = surface.content_rect();

        assert_eq!(surface.pan_zoom.fit, ViewportFit::Zoom);
        assert!((rect.x - 10.0).abs() < f32::EPSILON);
        assert!((rect.y - 55.0).abs() < f32::EPSILON);
    }

    #[test]
    fn texture_surface_emits_texture_primitive() {
        assert!(matches!(
            surface().texture_primitive(),
            Primitive::Texture(_)
        ));
    }

    #[test]
    fn ruler_ticks_change_with_zoom() {
        assert!(ruler_ticks(0.0, 100.0, 2.0).len() > ruler_ticks(0.0, 100.0, 0.5).len());
    }

    #[test]
    fn guide_primitives_emit_lines() {
        let primitives = guide_primitives(
            Rect::new(0.0, 0.0, 100.0, 100.0),
            &[Guide::Horizontal(50.0), Guide::Vertical(25.0)],
            Color::WHITE,
        );

        assert_eq!(primitives.len(), 2);
        assert!(matches!(primitives[0], Primitive::Line(_)));
    }

    #[test]
    fn crosshair_emits_lines_and_label_inside_bounds() {
        let crosshair = Crosshair {
            visible: true,
            position: Point::new(50.0, 50.0),
            label: Some("50,50".to_owned()),
            color: Color::WHITE,
        };

        let primitives = crosshair.primitives(Rect::new(0.0, 0.0, 100.0, 100.0));

        assert_eq!(primitives.len(), 3);
    }

    #[test]
    fn composition_orders_clip_texture_guides_crosshair() {
        let composition = ViewportComposition {
            surface: surface(),
            guides: vec![Guide::Horizontal(50.0)],
            crosshair: Some(Crosshair {
                visible: true,
                position: Point::new(50.0, 50.0),
                label: None,
                color: Color::WHITE,
            }),
            clip: ClipId::from_raw(1),
        };
        let primitives = composition.primitives();

        assert!(matches!(primitives[0], Primitive::ClipBegin { .. }));
        assert!(matches!(primitives[1], Primitive::Texture(_)));
        assert!(matches!(primitives.last(), Some(Primitive::ClipEnd { .. })));
    }
}
