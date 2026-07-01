use super::{
    Brush, Color, ComponentState, CornerRadius, CursorShape, NumericScrubInputConfig,
    NumericScrubInputOutput, Point, Primitive, Rect, RectPrimitive, Response, SemanticAction,
    SemanticActionKind, SemanticNode, SemanticRole, SemanticValue, TextEditState, TextLayoutStore,
    TextPrimitive, Theme, UiInput, UiMemory, VectorComponentLayout, VectorComponentRect, WidgetId,
    WidgetOutput, control_text_origin, finite_widget_extent,
    numeric_scrub_input_with_text_layouts_and_caret_visibility, pressable,
    suppress_disabled_interaction_reporting, vector2_component_rects, vector3_component_rects,
    vector4_component_rects, with_hover_cursor,
};

/// Configuration for vector numeric scrub fields.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VectorScrubInputConfig {
    /// Numeric scrub behavior applied to each component.
    pub numeric: NumericScrubInputConfig,
    /// Component rectangle layout.
    pub layout: VectorComponentLayout,
    /// Whether every component is disabled.
    pub disabled: bool,
    /// Whether every component is displayed but not editable.
    pub read_only: bool,
}

impl VectorScrubInputConfig {
    /// Creates a vector scrub configuration from a numeric component config.
    #[must_use]
    pub const fn new(numeric: NumericScrubInputConfig) -> Self {
        Self {
            numeric,
            layout: VectorComponentLayout::new(6.0, 10.0, 3.0, 24.0),
            disabled: false,
            read_only: false,
        }
    }

    /// Sets the component layout.
    #[must_use]
    pub const fn with_layout(mut self, layout: VectorComponentLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Sets whether all components are disabled.
    #[must_use]
    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets whether all components are read-only.
    #[must_use]
    pub const fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }
}

impl Default for VectorScrubInputConfig {
    fn default() -> Self {
        Self::new(NumericScrubInputConfig::default())
    }
}

/// Output emitted by vector numeric scrub fields.
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::struct_excessive_bools)]
pub struct VectorScrubInputOutput<const N: usize> {
    /// Aggregated widget output for component labels and numeric subfields.
    pub widget: WidgetOutput,
    /// Rectangles assigned to each vector component.
    pub component_rects: [VectorComponentRect; N],
    /// Numeric scrub outputs for each component in order.
    pub components: Vec<NumericScrubInputOutput>,
    /// Whether any component scrubbed this frame.
    pub scrubbed: bool,
    /// Whether any component value changed this frame.
    pub value_changed: bool,
    /// Whether all component interactions are disabled.
    pub disabled: bool,
    /// Whether all components are read-only.
    pub read_only: bool,
}

/// Configuration for a backend-independent color swatch/picker entry field.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorFieldConfig {
    /// Width of the leading color swatch.
    pub swatch_width: f32,
    /// Gap between swatch and text metadata.
    pub gap: f32,
    /// Whether the field is disabled.
    pub disabled: bool,
    /// Whether the field is displayed but cannot open a picker.
    pub read_only: bool,
}

impl ColorFieldConfig {
    /// Creates a color field configuration.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            swatch_width: 24.0,
            gap: 6.0,
            disabled: false,
            read_only: false,
        }
    }

    /// Sets the swatch width.
    #[must_use]
    pub const fn with_swatch_width(mut self, swatch_width: f32) -> Self {
        self.swatch_width = swatch_width;
        self
    }

    /// Sets the swatch-to-text gap.
    #[must_use]
    pub const fn with_gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Sets whether the field is disabled.
    #[must_use]
    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets whether the field is read-only.
    #[must_use]
    pub const fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }
}

impl Default for ColorFieldConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Output emitted by a color swatch/picker entry field.
#[derive(Debug, Clone, PartialEq)]
pub struct ColorFieldOutput {
    /// Base widget output.
    pub widget: WidgetOutput,
    /// Press/open response for the picker entry.
    pub response: Response,
    /// Sanitized RGBA value displayed by the swatch.
    pub color: Color,
    /// Whether the field requested that the application open a picker.
    pub open_requested: bool,
    /// Whether the field is read-only.
    pub read_only: bool,
}

/// Emits a Vec2 numeric scrub field.
#[allow(clippy::too_many_arguments)]
pub fn vector2_scrub_input(
    id: WidgetId,
    rect: Rect,
    label: impl AsRef<str>,
    values: &mut [f32; 2],
    states: &mut [TextEditState; 2],
    config: VectorScrubInputConfig,
    input: &UiInput,
    memory: &mut UiMemory,
    theme: &Theme,
) -> VectorScrubInputOutput<2> {
    vector_scrub_input_with_text_layouts_and_caret_visibility(
        id,
        rect,
        label.as_ref(),
        values,
        states,
        config,
        input,
        memory,
        theme,
        None,
        true,
        vector2_component_rects(rect, config.layout),
    )
}

/// Emits a Vec3 numeric scrub field.
#[allow(clippy::too_many_arguments)]
pub fn vector3_scrub_input(
    id: WidgetId,
    rect: Rect,
    label: impl AsRef<str>,
    values: &mut [f32; 3],
    states: &mut [TextEditState; 3],
    config: VectorScrubInputConfig,
    input: &UiInput,
    memory: &mut UiMemory,
    theme: &Theme,
) -> VectorScrubInputOutput<3> {
    vector_scrub_input_with_text_layouts_and_caret_visibility(
        id,
        rect,
        label.as_ref(),
        values,
        states,
        config,
        input,
        memory,
        theme,
        None,
        true,
        vector3_component_rects(rect, config.layout),
    )
}

/// Emits a Vec4 numeric scrub field.
#[allow(clippy::too_many_arguments)]
pub fn vector4_scrub_input(
    id: WidgetId,
    rect: Rect,
    label: impl AsRef<str>,
    values: &mut [f32; 4],
    states: &mut [TextEditState; 4],
    config: VectorScrubInputConfig,
    input: &UiInput,
    memory: &mut UiMemory,
    theme: &Theme,
) -> VectorScrubInputOutput<4> {
    vector_scrub_input_with_text_layouts_and_caret_visibility(
        id,
        rect,
        label.as_ref(),
        values,
        states,
        config,
        input,
        memory,
        theme,
        None,
        true,
        vector4_component_rects(rect, config.layout),
    )
}

/// Emits a backend-independent color swatch/picker entry field.
#[allow(clippy::too_many_arguments)]
pub fn color_field(
    id: WidgetId,
    rect: Rect,
    label: impl Into<String>,
    color: Color,
    config: ColorFieldConfig,
    input: &UiInput,
    memory: &mut UiMemory,
    theme: &Theme,
) -> ColorFieldOutput {
    let label = label.into();
    let interactions_disabled = config.disabled || config.read_only;
    let color = sanitize_color(color);
    let mut response = pressable(id, rect, input, memory, interactions_disabled);
    suppress_disabled_interaction_reporting(&mut response);
    let recipe = theme.text_field(ComponentState {
        hovered: response.state.hovered,
        pressed: response.state.pressed,
        focused: response.state.focused,
        disabled: interactions_disabled,
        selected: false,
    });
    let swatch_width = finite_widget_extent(config.swatch_width).min(rect.width.max(0.0));
    let gap = finite_widget_extent(config.gap).min((rect.width - swatch_width).max(0.0));
    let swatch_rect = Rect::new(
        rect.x + recipe.padding_x.min(rect.width.max(0.0)),
        rect.y + 4.0,
        (swatch_width - recipe.padding_x.min(swatch_width)).max(0.0),
        (rect.height - 8.0).max(0.0),
    );
    let text_x = swatch_rect.max_x() + gap;
    let text_rect = Rect::new(
        text_x,
        rect.y,
        (rect.max_x() - text_x).max(0.0),
        rect.height,
    );
    let value_text = format_color_value(color);
    let mut primitives = vec![
        Primitive::Rect(RectPrimitive {
            rect,
            fill: Some(recipe.background),
            stroke: Some(recipe.border),
            radius: CornerRadius::all(3.0),
        }),
        Primitive::Rect(RectPrimitive {
            rect: swatch_rect,
            fill: Some(Brush::Solid(color)),
            stroke: Some(recipe.border),
            radius: CornerRadius::all(2.0),
        }),
    ];
    if text_rect.width > 0.0 {
        primitives.push(Primitive::Text(TextPrimitive {
            layout: None,
            origin: Point::new(text_rect.x, control_text_origin(text_rect, theme).y),
            text: value_text.clone(),
            family: recipe.font.family.to_owned(),
            size: recipe.font.size,
            line_height: recipe.font.line_height,
            brush: Brush::Solid(recipe.foreground),
        }));
    }

    let mut node = SemanticNode::new(id, SemanticRole::Button, rect)
        .with_label(label)
        .focusable(!interactions_disabled);
    node.description = Some(value_text.clone());
    node.state.disabled = interactions_disabled;
    node.state.value = Some(SemanticValue::Text(value_text));
    if !interactions_disabled {
        node.actions.push(SemanticAction::new(
            SemanticActionKind::Open,
            "Open color picker",
        ));
    }

    let output = WidgetOutput::new(Some(response), primitives).with_semantic(node);
    let output = with_hover_cursor(output, &response, CursorShape::PointingHand);

    ColorFieldOutput {
        widget: output,
        response,
        color,
        open_requested: !interactions_disabled && (response.clicked || response.keyboard_activated),
        read_only: config.read_only,
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn vector_scrub_input_with_text_layouts_and_caret_visibility<const N: usize>(
    id: WidgetId,
    _rect: Rect,
    label: &str,
    values: &mut [f32; N],
    states: &mut [TextEditState; N],
    config: VectorScrubInputConfig,
    input: &UiInput,
    memory: &mut UiMemory,
    theme: &Theme,
    mut text_layouts: Option<&mut TextLayoutStore>,
    caret_visible: bool,
    component_rects: [VectorComponentRect; N],
) -> VectorScrubInputOutput<N> {
    let mut widget = WidgetOutput::new(None, Vec::new());
    let mut components = Vec::with_capacity(N);
    let mut scrubbed = false;
    let mut value_changed = false;
    let mut component_config = config.numeric;
    component_config.disabled = component_config.disabled || config.disabled;
    component_config.read_only = component_config.read_only || config.read_only;

    for component in component_rects {
        let component_id = id.child(component.label);
        let semantic_label = format!("{label} {}", component.label);
        let component_text_layouts = text_layouts.as_deref_mut();
        let mut output = numeric_scrub_input_with_text_layouts_and_caret_visibility(
            component_id,
            component.value_rect,
            &mut values[component.index],
            &mut states[component.index],
            component_config,
            input,
            memory,
            theme,
            component_text_layouts,
            caret_visible,
        );
        if let Some(node) = output.input.field.widget.semantics.first_mut() {
            node.label = Some(semantic_label);
        }
        widget.primitives.push(vector_component_label_primitive(
            component.label_rect,
            component.label,
            theme,
            component_config.disabled || component_config.read_only,
        ));
        widget
            .primitives
            .extend(output.input.field.widget.primitives.iter().cloned());
        widget
            .semantics
            .extend(output.input.field.widget.semantics.iter().cloned());
        widget
            .platform_requests
            .extend(output.input.field.widget.platform_requests.iter().cloned());
        scrubbed |= output.scrubbed;
        value_changed |= output.value_changed;
        components.push(output);
    }

    VectorScrubInputOutput {
        widget,
        component_rects,
        components,
        scrubbed,
        value_changed,
        disabled: component_config.disabled,
        read_only: component_config.read_only,
    }
}

fn vector_component_label_primitive(
    rect: Rect,
    label: impl Into<String>,
    theme: &Theme,
    disabled: bool,
) -> Primitive {
    let recipe = theme.text_field(ComponentState {
        hovered: false,
        pressed: false,
        focused: false,
        disabled,
        selected: false,
    });
    Primitive::Text(TextPrimitive {
        layout: None,
        origin: Point::new(rect.x, control_text_origin(rect, theme).y),
        text: label.into(),
        family: recipe.font.family.to_owned(),
        size: recipe.font.size,
        line_height: recipe.font.line_height,
        brush: Brush::Solid(recipe.foreground),
    })
}

fn sanitize_color(color: Color) -> Color {
    Color::rgba(
        sanitize_color_channel(color.r),
        sanitize_color_channel(color.g),
        sanitize_color_channel(color.b),
        sanitize_color_channel(color.a),
    )
}

fn sanitize_color_channel(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn format_color_value(color: Color) -> String {
    format!(
        "rgba({:.3}, {:.3}, {:.3}, {:.3})",
        color.r, color.g, color.b, color.a
    )
}
