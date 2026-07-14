//! Prepared public viewport widget contract.

use stern_core::{
    Point, PointerOrder, PointerTarget, PointerTargetPlan, Response, ScaleFactor, WidgetId,
};

use super::{PanZoom, ViewportActionDescriptor, ViewportActionRequest, ViewportSurface};

const DEFAULT_MIN_ZOOM: f32 = 0.05;
const DEFAULT_MAX_ZOOM: f32 = 64.0;
const DEFAULT_ZOOM_STEP: f32 = 0.2;

/// Caller-owned configuration for one prepared viewport widget.
#[derive(Debug, Clone, PartialEq)]
pub struct ViewportWidgetConfig {
    /// Stable widget identity.
    pub id: WidgetId,
    /// Frozen texture surface and current pan/zoom snapshot.
    pub surface: ViewportSurface,
    /// Accessible viewport label.
    pub label: String,
    /// Whether interaction is disabled.
    pub disabled: bool,
    /// Minimum custom zoom factor.
    pub min_zoom: f32,
    /// Maximum custom zoom factor.
    pub max_zoom: f32,
    /// Exponential wheel and action zoom step.
    pub zoom_step: f32,
    /// App-owned viewport actions exposed through semantics.
    pub actions: Vec<ViewportActionDescriptor>,
}

impl ViewportWidgetConfig {
    /// Creates an enabled viewport configuration with practical zoom defaults.
    #[must_use]
    pub fn new(id: WidgetId, surface: ViewportSurface) -> Self {
        Self {
            id,
            surface,
            label: "Viewport".to_owned(),
            disabled: false,
            min_zoom: DEFAULT_MIN_ZOOM,
            max_zoom: DEFAULT_MAX_ZOOM,
            zoom_step: DEFAULT_ZOOM_STEP,
            actions: Vec::new(),
        }
    }

    /// Sets the accessible label.
    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Sets whether interaction is disabled.
    #[must_use]
    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets the inclusive custom zoom range.
    #[must_use]
    pub const fn with_zoom_range(mut self, min: f32, max: f32) -> Self {
        self.min_zoom = min;
        self.max_zoom = max;
        self
    }

    /// Sets the exponential wheel and action zoom step.
    #[must_use]
    pub const fn with_zoom_step(mut self, step: f32) -> Self {
        self.zoom_step = step;
        self
    }

    /// Replaces the app-owned viewport action descriptors.
    #[must_use]
    pub fn with_actions(
        mut self,
        actions: impl IntoIterator<Item = ViewportActionDescriptor>,
    ) -> Self {
        self.actions = actions.into_iter().collect();
        self
    }
}

/// Immutable frame-local viewport widget.
#[derive(Debug, Clone, PartialEq)]
pub struct ViewportWidget {
    config: ViewportWidgetConfig,
    scale_factor: ScaleFactor,
}

impl ViewportWidget {
    /// Prepares a viewport widget and sanitizes its zoom policy.
    #[must_use]
    pub fn new(mut config: ViewportWidgetConfig, scale_factor: ScaleFactor) -> Self {
        let (min_zoom, max_zoom) = sanitize_zoom_range(config.min_zoom, config.max_zoom);
        config.min_zoom = min_zoom;
        config.max_zoom = max_zoom;
        config.zoom_step = sanitize_zoom_step(config.zoom_step);
        Self {
            config,
            scale_factor,
        }
    }

    /// Returns the prepared configuration.
    #[must_use]
    pub const fn config(&self) -> &ViewportWidgetConfig {
        &self.config
    }

    /// Returns the stable viewport widget identity.
    #[must_use]
    pub const fn widget_id(&self) -> WidgetId {
        self.config.id
    }

    /// Returns the frozen texture and pan/zoom snapshot.
    #[must_use]
    pub const fn surface(&self) -> ViewportSurface {
        self.config.surface
    }

    /// Returns the frame scale used by paint and coordinate conversion.
    #[must_use]
    pub const fn scale_factor(&self) -> ScaleFactor {
        self.scale_factor
    }

    /// Converts a screen point through the frozen painted snapshot.
    #[must_use]
    pub fn screen_to_content(&self, point: Point) -> Option<Point> {
        self.config
            .surface
            .screen_to_content_at(point, self.scale_factor)
    }

    /// Converts a content point through the frozen painted snapshot.
    #[must_use]
    pub fn content_to_screen(&self, point: Point) -> Option<Point> {
        self.config
            .surface
            .content_to_screen_at(point, self.scale_factor)
    }

    /// Adds the viewport blocker and routed interaction target to a pointer plan.
    pub fn declare_pointer_targets(
        &self,
        plan: &mut PointerTargetPlan,
        first_order: PointerOrder,
    ) -> PointerOrder {
        let bounds = self.config.surface.effective_bounds();
        if bounds.width <= 0.0 || bounds.height <= 0.0 {
            return first_order;
        }

        plan.blocker(bounds, first_order);
        let target_order = PointerOrder::new(first_order.raw().saturating_add(1));
        plan.target(
            PointerTarget::new(self.config.id, bounds, target_order)
                .wheel_owner(self.config.id)
                .domain_drag_source()
                .enabled(!self.config.disabled),
        );
        PointerOrder::new(target_order.raw().saturating_add(1))
    }
}

/// Output from one public viewport widget evaluation.
#[derive(Debug, Clone, PartialEq)]
pub struct ViewportWidgetOutput {
    /// Common interaction response for the viewport surface.
    pub response: Response,
    /// Exact frozen surface used for paint, hit testing, and conversions.
    pub surface: ViewportSurface,
    /// Pan/zoom state staged for the caller's next prepared frame.
    pub next_pan_zoom: PanZoom,
    /// Pointer position converted through the frozen surface, when inside it.
    pub content_pointer: Option<Point>,
    /// Whether pan changed this frame.
    pub pan_changed: bool,
    /// Whether custom zoom changed this frame.
    pub zoom_changed: bool,
    /// Whether fit/display mode changed this frame.
    pub fit_changed: bool,
    /// Targeted action requests not consumed by generic viewport navigation.
    pub action_requests: Vec<ViewportActionRequest>,
}

impl ViewportWidgetOutput {
    /// Returns true when the caller should prepare a new pan/zoom snapshot.
    #[must_use]
    pub const fn changed(&self) -> bool {
        self.pan_changed || self.zoom_changed || self.fit_changed
    }
}

fn sanitize_zoom_range(min: f32, max: f32) -> (f32, f32) {
    if min.is_finite() && max.is_finite() && min > 0.0 && max >= min {
        (min, max)
    } else {
        (DEFAULT_MIN_ZOOM, DEFAULT_MAX_ZOOM)
    }
}

fn sanitize_zoom_step(step: f32) -> f32 {
    if step.is_finite() && step > 0.0 {
        step.min(4.0)
    } else {
        DEFAULT_ZOOM_STEP
    }
}
