use super::{
    Rect, SemanticAction, SemanticActionKind, SemanticNode, SemanticRole, SemanticState,
    SemanticValue, WidgetId,
};

/// Returns semantics for a static label.
#[must_use]
pub fn label_semantics(id: WidgetId, rect: Rect, text: impl Into<String>) -> SemanticNode {
    SemanticNode::new(id, SemanticRole::Label, rect).with_label(text)
}

/// Returns semantics for a push button.
#[must_use]
pub fn button_semantics(
    id: WidgetId,
    rect: Rect,
    text: impl Into<String>,
    disabled: bool,
) -> SemanticNode {
    let mut node = SemanticNode::new(id, SemanticRole::Button, rect)
        .with_label(text)
        .focusable(!disabled)
        .with_action(SemanticAction::new(SemanticActionKind::Invoke, "Invoke"));
    node.state.disabled = disabled;
    node
}

/// Returns semantics for an icon button.
#[must_use]
pub fn icon_button_semantics(
    id: WidgetId,
    rect: Rect,
    label: impl Into<String>,
    disabled: bool,
) -> SemanticNode {
    let mut node = button_semantics(id, rect, label, disabled);
    node.role = SemanticRole::IconButton;
    node
}

/// Returns semantics for a checkbox.
#[must_use]
pub fn checkbox_semantics(
    id: WidgetId,
    rect: Rect,
    label: impl Into<String>,
    checked: bool,
    disabled: bool,
) -> SemanticNode {
    let mut node = SemanticNode::new(id, SemanticRole::CheckBox, rect)
        .with_label(label)
        .focusable(!disabled)
        .with_action(SemanticAction::new(SemanticActionKind::Invoke, "Toggle"));
    node.state = SemanticState {
        disabled,
        checked: Some(checked),
        ..SemanticState::default()
    };
    node
}

/// Returns semantics for a radio button.
#[must_use]
pub fn radio_button_semantics(
    id: WidgetId,
    rect: Rect,
    label: impl Into<String>,
    selected: bool,
    disabled: bool,
) -> SemanticNode {
    let mut node = checkbox_semantics(id, rect, label, selected, disabled);
    node.role = SemanticRole::RadioButton;
    node.state.selected = selected;
    node
}

/// Returns semantics for a toggle control.
#[must_use]
pub fn toggle_semantics(
    id: WidgetId,
    rect: Rect,
    label: impl Into<String>,
    on: bool,
    disabled: bool,
) -> SemanticNode {
    let mut node = checkbox_semantics(id, rect, label, on, disabled);
    node.role = SemanticRole::Toggle;
    node
}

/// Returns semantics for a slider.
#[must_use]
pub fn slider_semantics(
    id: WidgetId,
    rect: Rect,
    label: impl Into<String>,
    value: f32,
    range: core::ops::RangeInclusive<f32>,
    disabled: bool,
) -> SemanticNode {
    let mut node = SemanticNode::new(id, SemanticRole::Slider, rect)
        .with_label(label)
        .focusable(!disabled)
        .with_action(SemanticAction::new(
            SemanticActionKind::Increment,
            "Increase",
        ))
        .with_action(SemanticAction::new(
            SemanticActionKind::Decrement,
            "Decrease",
        ))
        .with_action(SemanticAction::new(
            SemanticActionKind::SetValue,
            "Set value",
        ));
    node.state = SemanticState {
        disabled,
        value: Some(SemanticValue::Number {
            current: value,
            min: *range.start(),
            max: *range.end(),
        }),
        ..SemanticState::default()
    };
    node
}

/// Returns semantics for a text field.
#[must_use]
pub fn text_field_semantics(
    id: WidgetId,
    rect: Rect,
    label: impl Into<String>,
    text: impl Into<String>,
    disabled: bool,
) -> SemanticNode {
    let mut node = SemanticNode::new(id, SemanticRole::TextField, rect)
        .with_label(label)
        .focusable(!disabled)
        .with_action(SemanticAction::new(SemanticActionKind::SetText, "Set text"));
    node.state = SemanticState {
        disabled,
        value: Some(SemanticValue::Text(text.into())),
        ..SemanticState::default()
    };
    node
}

pub(crate) fn text_field_semantics_with_access(
    id: WidgetId,
    rect: Rect,
    label: impl Into<String>,
    text: impl Into<String>,
    access: super::text_fields::TextFieldAccess,
) -> SemanticNode {
    let disabled = access == super::text_fields::TextFieldAccess::Disabled;
    let mut node = SemanticNode::new(id, SemanticRole::TextField, rect)
        .with_label(label)
        .focusable(!disabled);
    if access == super::text_fields::TextFieldAccess::Editable {
        node = node.with_action(SemanticAction::new(SemanticActionKind::SetText, "Set text"));
    }
    node.state = SemanticState {
        disabled,
        value: Some(SemanticValue::Text(text.into())),
        ..SemanticState::default()
    };
    node
}

/// Returns semantics for a search field.
#[must_use]
pub fn search_field_semantics(
    id: WidgetId,
    rect: Rect,
    label: impl Into<String>,
    query: impl Into<String>,
    disabled: bool,
) -> SemanticNode {
    let mut node = text_field_semantics(id, rect, label, query, disabled);
    node.role = SemanticRole::SearchField;
    node
}

/// Returns semantics for a passive panel.
#[must_use]
pub fn panel_semantics(id: WidgetId, rect: Rect, label: impl Into<String>) -> SemanticNode {
    SemanticNode::new(id, SemanticRole::Panel, rect).with_label(label)
}

/// Returns semantics for a static image.
#[must_use]
pub fn image_semantics(id: WidgetId, rect: Rect, label: impl Into<String>) -> SemanticNode {
    SemanticNode::new(id, SemanticRole::Image, rect).with_label(label)
}
