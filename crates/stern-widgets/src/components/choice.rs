use super::{
    ComponentState, CornerRadius, CursorShape, Primitive, Rect, RectPrimitive, SemanticRole, Theme,
    UiInput, UiMemory, WidgetId, WidgetOutput, checkbox_semantics, clicked_select_state,
    clicked_toggle_state, response_reported_focus, response_reported_pressed, selectable,
    suppress_disabled_interaction_reporting, toggle_semantics, with_hover_cursor,
    with_response_state,
};

/// Returns the deterministic activation target for a choice control and its label.
#[must_use]
pub fn choice_label_target_rect(control_rect: Rect, label_rect: Rect) -> Rect {
    control_rect.union(label_rect)
}

/// Emits a checkbox.
pub fn checkbox(
    id: WidgetId,
    rect: Rect,
    checked: bool,
    input: &UiInput,
    memory: &mut UiMemory,
    theme: &Theme,
    disabled: bool,
) -> WidgetOutput {
    checkbox_with_label(
        id, rect, "Checkbox", checked, input, memory, theme, disabled,
    )
}

/// Emits a checkbox with an accessible label.
#[allow(clippy::too_many_arguments)]
pub fn checkbox_with_label(
    id: WidgetId,
    rect: Rect,
    label: impl Into<String>,
    checked: bool,
    input: &UiInput,
    memory: &mut UiMemory,
    theme: &Theme,
    disabled: bool,
) -> WidgetOutput {
    checkbox_with_label_target(
        id,
        rect,
        Rect::ZERO,
        label,
        checked,
        input,
        memory,
        theme,
        disabled,
    )
}

/// Emits a checkbox with a deterministic label activation target.
#[allow(clippy::too_many_arguments)]
pub fn checkbox_with_label_target(
    id: WidgetId,
    rect: Rect,
    label_rect: Rect,
    label: impl Into<String>,
    checked: bool,
    input: &UiInput,
    memory: &mut UiMemory,
    theme: &Theme,
    disabled: bool,
) -> WidgetOutput {
    let target_rect = choice_label_target_rect(rect, label_rect);
    let mut response = selectable(id, target_rect, input, memory, checked, disabled);
    suppress_disabled_interaction_reporting(&mut response);
    let selected = clicked_toggle_state(checked, response.clicked);
    response.state.selected = selected;
    let recipe = theme.checkbox(ComponentState {
        hovered: response.state.hovered,
        pressed: response_reported_pressed(&response),
        focused: response_reported_focus(&response),
        disabled,
        selected,
    });
    let box_rect = Rect::new(rect.x, rect.y, recipe.size, recipe.size);

    with_hover_cursor(
        WidgetOutput::new(
            Some(response),
            vec![Primitive::Rect(RectPrimitive {
                rect: box_rect,
                fill: Some(recipe.fill),
                stroke: Some(recipe.border),
                radius: recipe.radius,
            })],
        )
        .with_semantic(with_response_state(
            checkbox_semantics(id, target_rect, label, selected, disabled),
            &response,
        )),
        &response,
        CursorShape::PointingHand,
    )
}

/// Emits a radio button.
pub fn radio_button(
    id: WidgetId,
    rect: Rect,
    selected: bool,
    input: &UiInput,
    memory: &mut UiMemory,
    theme: &Theme,
    disabled: bool,
) -> WidgetOutput {
    radio_button_with_label(
        id,
        rect,
        "Radio button",
        selected,
        input,
        memory,
        theme,
        disabled,
    )
}

/// Emits a radio button with an accessible label.
#[allow(clippy::too_many_arguments)]
pub fn radio_button_with_label(
    id: WidgetId,
    rect: Rect,
    label: impl Into<String>,
    selected: bool,
    input: &UiInput,
    memory: &mut UiMemory,
    theme: &Theme,
    disabled: bool,
) -> WidgetOutput {
    radio_button_with_label_target(
        id,
        rect,
        Rect::ZERO,
        label,
        selected,
        input,
        memory,
        theme,
        disabled,
    )
}

/// Emits a radio button with a deterministic label activation target.
#[allow(clippy::too_many_arguments)]
pub fn radio_button_with_label_target(
    id: WidgetId,
    rect: Rect,
    label_rect: Rect,
    label: impl Into<String>,
    selected: bool,
    input: &UiInput,
    memory: &mut UiMemory,
    theme: &Theme,
    disabled: bool,
) -> WidgetOutput {
    let mut output = checkbox_with_label_target(
        id, rect, label_rect, label, selected, input, memory, theme, disabled,
    );
    let display_selected = clicked_select_state(
        selected,
        output
            .response
            .as_ref()
            .is_some_and(|response| response.clicked),
    );
    if let Some(response) = output.response.as_mut() {
        response.state.selected = display_selected;
    }
    let recipe = theme.radio_button(ComponentState {
        hovered: output
            .response
            .as_ref()
            .is_some_and(|response| response.state.hovered),
        pressed: output
            .response
            .as_ref()
            .is_some_and(response_reported_pressed),
        focused: output
            .response
            .as_ref()
            .is_some_and(response_reported_focus),
        disabled,
        selected: display_selected,
    });
    if let Some(Primitive::Rect(primitive)) = output.primitives.first_mut() {
        primitive.radius = recipe.radius;
    }
    for node in &mut output.semantics {
        node.role = SemanticRole::RadioButton;
        node.state.selected = display_selected;
        node.state.checked = Some(display_selected);
    }
    output
}

/// Emits a toggle control.
pub fn toggle(
    id: WidgetId,
    rect: Rect,
    on: bool,
    input: &UiInput,
    memory: &mut UiMemory,
    theme: &Theme,
    disabled: bool,
) -> WidgetOutput {
    toggle_with_label(id, rect, "Toggle", on, input, memory, theme, disabled)
}

/// Emits a toggle control with an accessible label.
#[allow(clippy::too_many_arguments)]
pub fn toggle_with_label(
    id: WidgetId,
    rect: Rect,
    label: impl Into<String>,
    on: bool,
    input: &UiInput,
    memory: &mut UiMemory,
    theme: &Theme,
    disabled: bool,
) -> WidgetOutput {
    toggle_with_label_target(
        id,
        rect,
        Rect::ZERO,
        label,
        on,
        input,
        memory,
        theme,
        disabled,
    )
}

/// Emits a toggle control with a deterministic label activation target.
#[allow(clippy::too_many_arguments)]
pub fn toggle_with_label_target(
    id: WidgetId,
    rect: Rect,
    label_rect: Rect,
    label: impl Into<String>,
    on: bool,
    input: &UiInput,
    memory: &mut UiMemory,
    theme: &Theme,
    disabled: bool,
) -> WidgetOutput {
    let target_rect = choice_label_target_rect(rect, label_rect);
    let mut response = selectable(id, target_rect, input, memory, on, disabled);
    suppress_disabled_interaction_reporting(&mut response);
    let selected = clicked_toggle_state(on, response.clicked);
    response.state.selected = selected;
    let recipe = theme.toggle(ComponentState {
        hovered: response.state.hovered,
        pressed: response_reported_pressed(&response),
        focused: response_reported_focus(&response),
        disabled,
        selected,
    });
    let knob_x = if selected {
        rect.max_x() - rect.height
    } else {
        rect.x
    };

    with_hover_cursor(
        WidgetOutput::new(
            Some(response),
            vec![
                Primitive::Rect(RectPrimitive {
                    rect,
                    fill: Some(recipe.track),
                    stroke: Some(recipe.border),
                    radius: CornerRadius::all(rect.height * 0.5),
                }),
                Primitive::Rect(RectPrimitive {
                    rect: Rect::new(
                        knob_x + recipe.padding,
                        rect.y + recipe.padding,
                        rect.height - recipe.padding * 2.0,
                        rect.height - recipe.padding * 2.0,
                    ),
                    fill: Some(recipe.thumb),
                    stroke: None,
                    radius: CornerRadius::all((rect.height - recipe.padding * 2.0) * 0.5),
                }),
            ],
        )
        .with_semantic(with_response_state(
            toggle_semantics(id, target_rect, label, selected, disabled),
            &response,
        )),
        &response,
        CursorShape::PointingHand,
    )
}
