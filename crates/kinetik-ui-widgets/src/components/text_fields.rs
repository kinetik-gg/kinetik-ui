use super::{
    ComponentState, CursorShape, OrderedTextInputResult, Primitive, Rect, RectPrimitive, Response,
    TextEditMode, TextEditState, TextLayoutStore, TextSelection, Theme, UiInput, UiMemory,
    WidgetId, WidgetOutput, display_text_with_composition, focusable, multi_line_hit_offset,
    multi_line_text_primitives, single_line_hit_offset, single_line_text_primitives,
    text_field_layout, text_field_semantics, text_input_platform_requests, text_line_fragments,
    with_hover_cursor, with_response_state,
};
use kinetik_ui_core::{RepaintRequest, TextInputOwnerMode, Ui as CoreUi};
use kinetik_ui_text::TextViewport;

use super::semantics::text_field_semantics_with_access;
use super::text_geometry::{TextFieldGeometry, TextFieldKind};
use super::text_interaction::{
    ResolvedTextPointerAction, TextPointerPhase, TextReplayResult, replay_text_field_events,
    text_wheel_delta,
};

/// Access policy for a canonical text field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextFieldAccess {
    /// Focusable text that accepts selection, editing, clipboard, and IME input.
    Editable,
    /// Focusable text that permits navigation, selection, and copy without mutation or IME.
    ReadOnly,
    /// Non-interactive text that cannot focus, select, scroll, copy, edit, or own IME.
    Disabled,
}

impl TextFieldAccess {
    const fn is_disabled(self) -> bool {
        matches!(self, Self::Disabled)
    }

    const fn owner_mode(self) -> Option<TextInputOwnerMode> {
        match self {
            Self::Editable => Some(TextInputOwnerMode::Editable),
            Self::ReadOnly => Some(TextInputOwnerMode::ReadOnly),
            Self::Disabled => None,
        }
    }
}

/// Output emitted by editable text widgets.
#[derive(Debug, Clone, PartialEq)]
pub struct TextFieldOutput {
    /// Base widget output.
    pub widget: WidgetOutput,
    /// Whether the text changed this frame.
    pub changed: bool,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn text_field_with_access_runtime(
    runtime: &mut CoreUi<'_>,
    id: WidgetId,
    rect: Rect,
    state: &mut TextEditState,
    theme: &Theme,
    access: TextFieldAccess,
    text_layouts: Option<&mut TextLayoutStore>,
    caret_visible: bool,
) -> (TextFieldOutput, OrderedTextInputResult) {
    let result = canonical_text_field_runtime(
        runtime,
        id,
        rect,
        state,
        theme,
        access,
        text_layouts,
        caret_visible,
        TextFieldKind::SingleLine,
        TextEditMode::SingleLine,
    );
    (
        TextFieldOutput {
            widget: result.widget,
            changed: result.changed,
        },
        result.ordered,
    )
}

/// Emits a single-line text field and applies text input while focused.
pub fn text_field(
    id: WidgetId,
    rect: Rect,
    state: &mut TextEditState,
    input: &UiInput,
    memory: &mut UiMemory,
    theme: &Theme,
    disabled: bool,
) -> TextFieldOutput {
    text_field_with_text_layouts(id, rect, state, input, memory, theme, disabled, None)
}

/// Emits a single-line text field using shaped text layout cache when available.
#[allow(clippy::too_many_arguments)]
pub fn text_field_with_text_layouts(
    id: WidgetId,
    rect: Rect,
    state: &mut TextEditState,
    input: &UiInput,
    memory: &mut UiMemory,
    theme: &Theme,
    disabled: bool,
    text_layouts: Option<&mut TextLayoutStore>,
) -> TextFieldOutput {
    text_field_with_text_layouts_and_caret_visibility(
        id,
        rect,
        state,
        input,
        memory,
        theme,
        disabled,
        text_layouts,
        true,
    )
}

/// Emits a single-line text field with explicit caret visibility.
#[allow(clippy::too_many_arguments)]
pub(crate) fn text_field_with_text_layouts_and_caret_visibility(
    id: WidgetId,
    rect: Rect,
    state: &mut TextEditState,
    input: &UiInput,
    memory: &mut UiMemory,
    theme: &Theme,
    disabled: bool,
    text_layouts: Option<&mut TextLayoutStore>,
    caret_visible: bool,
) -> TextFieldOutput {
    text_field_with_text_layouts_and_caret_visibility_and_ordered_result(
        id,
        rect,
        state,
        input,
        memory,
        theme,
        disabled,
        text_layouts,
        caret_visible,
    )
    .0
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn text_field_with_text_layouts_and_caret_visibility_and_ordered_result(
    id: WidgetId,
    rect: Rect,
    state: &mut TextEditState,
    input: &UiInput,
    memory: &mut UiMemory,
    theme: &Theme,
    disabled: bool,
    text_layouts: Option<&mut TextLayoutStore>,
    caret_visible: bool,
) -> (TextFieldOutput, OrderedTextInputResult) {
    let response = focusable(id, rect, input, memory, disabled);
    text_field_with_resolved_response_and_ordered_result(
        id,
        rect,
        state,
        input,
        memory,
        theme,
        disabled,
        text_layouts,
        caret_visible,
        response,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn text_field_with_resolved_response_and_ordered_result(
    id: WidgetId,
    rect: Rect,
    state: &mut TextEditState,
    input: &UiInput,
    memory: &mut UiMemory,
    theme: &Theme,
    disabled: bool,
    mut text_layouts: Option<&mut TextLayoutStore>,
    caret_visible: bool,
    mut response: Response,
) -> (TextFieldOutput, OrderedTextInputResult) {
    let before = state.text.clone();
    if response.clicked {
        memory.focus(id);
        response.state.focused = true;
    }
    let hit_recipe = theme.text_field(ComponentState {
        hovered: response.state.hovered,
        pressed: response.state.pressed,
        focused: response.state.focused,
        disabled,
        selected: false,
    });
    let (hit_text, _, _) = display_text_with_composition(state);
    if !disabled
        && response.state.hovered
        && input.pointer.primary.pressed
        && let Some(position) = input.pointer.position
    {
        let hit_layout = text_field_layout(
            text_layouts.as_deref_mut(),
            &hit_text,
            rect,
            &hit_recipe,
            false,
        );
        state.set_caret(single_line_hit_offset(
            position,
            rect,
            &hit_text,
            &hit_recipe,
            hit_layout,
        ));
        memory.focus(id);
        response.state.focused = true;
    }
    let mut platform_requests = text_input_platform_requests(id, rect, &response, memory);
    let mut ordered_result = OrderedTextInputResult::default();
    if response.state.focused
        && !disabled
        && memory.claim_text_input_events(id)
        && let Ok(events) = memory.effective_text_input_events(input)
    {
        ordered_result =
            state.apply_ordered_input_with_result(&events, id, TextEditMode::SingleLine);
        platform_requests.extend(ordered_result.platform_requests.iter().cloned());
    }
    let recipe = theme.text_field(ComponentState {
        hovered: response.state.hovered,
        pressed: response.state.pressed,
        focused: response.state.focused,
        disabled,
        selected: false,
    });
    let (display_text, _, _) = display_text_with_composition(state);
    let layout = text_field_layout(text_layouts, &display_text, rect, &recipe, false);
    let mut primitives = vec![Primitive::Rect(RectPrimitive {
        rect,
        fill: Some(recipe.background),
        stroke: Some(recipe.border),
        radius: recipe.radius,
    })];
    primitives.extend(single_line_text_primitives(
        id,
        rect,
        state,
        response.state.focused && !disabled,
        caret_visible,
        &recipe,
        layout,
    ));

    (
        TextFieldOutput {
            widget: with_hover_cursor(
                WidgetOutput::new(Some(response), primitives)
                    .with_semantic(with_response_state(
                        text_field_semantics(id, rect, "Text field", state.text.clone(), disabled),
                        &response,
                    ))
                    .with_platform_requests(platform_requests),
                &response,
                CursorShape::Text,
            ),
            changed: before != state.text,
        },
        ordered_result,
    )
}

/// Output emitted by multi-line text fields.
#[derive(Debug, Clone, PartialEq)]
pub struct MultiLineTextFieldOutput {
    /// Base widget output.
    pub widget: WidgetOutput,
    /// Whether the text changed this frame.
    pub changed: bool,
    /// Visible line count emitted by the widget.
    pub visible_lines: usize,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn multi_line_text_field_with_access_runtime(
    runtime: &mut CoreUi<'_>,
    id: WidgetId,
    rect: Rect,
    state: &mut TextEditState,
    theme: &Theme,
    access: TextFieldAccess,
    text_layouts: Option<&mut TextLayoutStore>,
    caret_visible: bool,
) -> MultiLineTextFieldOutput {
    let result = canonical_text_field_runtime(
        runtime,
        id,
        rect,
        state,
        theme,
        access,
        text_layouts,
        caret_visible,
        TextFieldKind::WrappedMultiLine,
        TextEditMode::MultiLine,
    );
    MultiLineTextFieldOutput {
        widget: result.widget,
        changed: result.changed,
        visible_lines: text_line_fragments(&state.text).len(),
    }
}

/// Parsed state of a numeric input draft.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NumericInputDraft {
    /// The draft contains only whitespace.
    Empty,
    /// The draft parses as a numeric value.
    Valid(f32),
    /// The draft is non-empty and does not parse as a numeric value.
    Invalid,
}

impl NumericInputDraft {
    /// Returns the parsed value when the draft is valid and non-empty.
    #[must_use]
    pub const fn value(self) -> Option<f32> {
        match self {
            Self::Valid(value) => Some(value),
            Self::Empty | Self::Invalid => None,
        }
    }

    /// Returns true when the draft is empty.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Returns true when the draft is empty or valid.
    #[must_use]
    pub const fn is_acceptable(self) -> bool {
        matches!(self, Self::Empty | Self::Valid(_))
    }
}

/// Generic commit/revert policy emitted by numeric inputs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NumericInputPolicy {
    /// Current draft classification.
    pub draft: NumericInputDraft,
    /// Whether the current frame requested committing a valid non-empty draft.
    pub commit_requested: bool,
    /// Whether the current frame requested reverting the draft to a caller-owned baseline.
    pub revert_requested: bool,
}

impl NumericInputPolicy {
    /// Creates a policy with no keyboard requests.
    #[must_use]
    pub const fn idle(draft: NumericInputDraft) -> Self {
        Self {
            draft,
            commit_requested: false,
            revert_requested: false,
        }
    }
}

/// Classifies numeric input draft text without mutating widget or application state.
#[must_use]
pub fn classify_numeric_input_draft(text: &str) -> NumericInputDraft {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        NumericInputDraft::Empty
    } else if let Ok(value) = trimmed.parse::<f32>() {
        NumericInputDraft::Valid(value)
    } else {
        NumericInputDraft::Invalid
    }
}

/// Restores a text-edit draft to a caller-owned baseline.
///
/// This helper is generic text state plumbing for commit/revert flows. It does
/// not parse, validate, or apply application-owned numeric values.
pub fn restore_text_draft(state: &mut TextEditState, draft: impl Into<String>) -> bool {
    let draft = draft.into();
    let caret = draft.len();
    let changed = state.text != draft
        || state.composition.is_some()
        || state.selection != TextSelection::new(caret, caret);

    state.text = draft;
    state.composition = None;
    state.set_selection(TextSelection::new(caret, caret));

    changed
}

/// Emits a multi-line text field and applies text input while focused.
pub fn multi_line_text_field(
    id: WidgetId,
    rect: Rect,
    state: &mut TextEditState,
    input: &UiInput,
    memory: &mut UiMemory,
    theme: &Theme,
    disabled: bool,
) -> MultiLineTextFieldOutput {
    multi_line_text_field_with_text_layouts(id, rect, state, input, memory, theme, disabled, None)
}

/// Emits a multi-line text field using shaped text layout cache when available.
#[allow(clippy::too_many_arguments)]
pub fn multi_line_text_field_with_text_layouts(
    id: WidgetId,
    rect: Rect,
    state: &mut TextEditState,
    input: &UiInput,
    memory: &mut UiMemory,
    theme: &Theme,
    disabled: bool,
    text_layouts: Option<&mut TextLayoutStore>,
) -> MultiLineTextFieldOutput {
    multi_line_text_field_with_text_layouts_and_caret_visibility(
        id,
        rect,
        state,
        input,
        memory,
        theme,
        disabled,
        text_layouts,
        true,
    )
}

/// Emits a multi-line text field with explicit caret visibility.
#[allow(clippy::too_many_arguments)]
pub(crate) fn multi_line_text_field_with_text_layouts_and_caret_visibility(
    id: WidgetId,
    rect: Rect,
    state: &mut TextEditState,
    input: &UiInput,
    memory: &mut UiMemory,
    theme: &Theme,
    disabled: bool,
    mut text_layouts: Option<&mut TextLayoutStore>,
    caret_visible: bool,
) -> MultiLineTextFieldOutput {
    let before = state.text.clone();
    let mut response = focusable(id, rect, input, memory, disabled);
    let hit_recipe = theme.text_field(ComponentState {
        hovered: response.state.hovered,
        pressed: response.state.pressed,
        focused: response.state.focused,
        disabled,
        selected: false,
    });
    let (hit_text, _, _) = display_text_with_composition(state);
    if !disabled
        && response.state.hovered
        && input.pointer.primary.pressed
        && let Some(position) = input.pointer.position
    {
        let hit_layout = text_field_layout(
            text_layouts.as_deref_mut(),
            &hit_text,
            rect,
            &hit_recipe,
            true,
        );
        state.set_caret(multi_line_hit_offset(
            position,
            rect,
            &hit_text,
            &hit_recipe,
            hit_layout,
        ));
        memory.focus(id);
        response.state.focused = true;
    }
    let mut platform_requests = text_input_platform_requests(id, rect, &response, memory);
    if response.state.focused
        && !disabled
        && memory.claim_text_input_events(id)
        && let Ok(events) = memory.effective_text_input_events(input)
    {
        platform_requests.extend(state.apply_ordered_input(&events, id, TextEditMode::MultiLine));
    }
    let recipe = theme.text_field(ComponentState {
        hovered: response.state.hovered,
        pressed: response.state.pressed,
        focused: response.state.focused,
        disabled,
        selected: false,
    });
    let (display_text, _, _) = display_text_with_composition(state);
    let layout = text_field_layout(text_layouts, &display_text, rect, &recipe, true);
    let mut primitives = vec![Primitive::Rect(RectPrimitive {
        rect,
        fill: Some(recipe.background),
        stroke: Some(recipe.border),
        radius: recipe.radius,
    })];
    primitives.extend(multi_line_text_primitives(
        id,
        rect,
        state,
        response.state.focused && !disabled,
        caret_visible,
        &recipe,
        layout,
    ));

    MultiLineTextFieldOutput {
        widget: with_hover_cursor(
            WidgetOutput::new(Some(response), primitives)
                .with_semantic(with_response_state(
                    text_field_semantics(id, rect, "Text field", state.text.clone(), disabled),
                    &response,
                ))
                .with_platform_requests(platform_requests),
            &response,
            CursorShape::Text,
        ),
        changed: before != state.text,
        visible_lines: text_line_fragments(&state.text).len(),
    }
}

struct CanonicalTextFieldResult {
    widget: WidgetOutput,
    changed: bool,
    ordered: OrderedTextInputResult,
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn canonical_text_field_runtime(
    runtime: &mut CoreUi<'_>,
    id: WidgetId,
    rect: Rect,
    state: &mut TextEditState,
    theme: &Theme,
    access: TextFieldAccess,
    mut text_layouts: Option<&mut TextLayoutStore>,
    caret_visible: bool,
    kind: TextFieldKind,
    edit_mode: TextEditMode,
) -> CanonicalTextFieldResult {
    if access == TextFieldAccess::ReadOnly && state.composition.is_some() {
        let _ = state.apply_read_only_ordered_input(&[], edit_mode);
    }
    let before = state.text.clone();
    let entry_focused = runtime.memory().is_focused(id);
    let entry_selection_anchor = state.selection.anchor;
    let retained_gesture_anchor = runtime.memory().selection_gesture_anchor(id);
    let retained_offset = runtime.memory().scroll_offset(id);
    let gesture = runtime.captured_selection_gesture(id, rect, access.is_disabled());
    let mut response = gesture.response;
    let entry_recipe = theme.text_field(ComponentState {
        hovered: response.state.hovered,
        pressed: response.state.pressed,
        focused: entry_focused,
        disabled: access.is_disabled(),
        selected: false,
    });
    let entry_geometry = TextFieldGeometry::build(
        rect,
        state,
        &entry_recipe,
        kind,
        retained_offset,
        text_layouts.as_deref_mut(),
    );
    let mut pointer_actions = gesture
        .actions
        .into_iter()
        .map(|action| ResolvedTextPointerAction {
            ordinal: action.ordinal,
            phase: TextPointerPhase::from(action.phase),
            model_offset: action.position.map(|position| {
                entry_geometry.model_offset_at_with_layout(position, text_layouts.as_deref())
            }),
            click_count: action.click_count,
            modifiers: action.modifiers,
        })
        .collect::<Vec<_>>();

    let final_root_press = runtime.last_root_primary_press_ordinal();
    let legacy_snapshot_press =
        runtime.input().events.is_empty() && runtime.input().pointer.primary.pressed;
    let root_press_present = final_root_press.is_some() || legacy_snapshot_press;
    let owns_press = if let Some(final_ordinal) = final_root_press {
        final_primary_press_is_unambiguous(&pointer_actions, final_ordinal)
            && pointer_actions
                .iter()
                .filter(|action| {
                    action.phase == TextPointerPhase::Press
                        && action.ordinal == Some(final_ordinal)
                        && action.model_offset.is_some()
                })
                .count()
                == 1
    } else if legacy_snapshot_press {
        pointer_actions
            .iter()
            .filter(|action| {
                action.phase == TextPointerPhase::Press
                    && action.ordinal.is_none()
                    && action.model_offset.is_some()
            })
            .count()
            == 1
    } else {
        false
    };

    if let Some(final_ordinal) = final_root_press {
        if owns_press {
            pointer_actions.retain(|action| {
                action
                    .ordinal
                    .is_some_and(|ordinal| ordinal >= final_ordinal)
            });
        } else {
            pointer_actions.clear();
        }
    } else if legacy_snapshot_press && !owns_press {
        pointer_actions.clear();
    }

    if access.is_disabled() || (root_press_present && !owns_press) {
        if runtime.memory().is_focused(id) {
            runtime.memory_mut().clear_focus();
        }
    } else if owns_press {
        runtime.memory_mut().focus(id);
    }

    let entry_accepts_input = entry_focused && (!root_press_present || owns_press);
    let prepared = access.owner_mode().is_some_and(|mode| {
        runtime.memory().is_focused(id) && runtime.prepare_text_input_owner(id, mode)
    });
    let ordered_events = if prepared {
        runtime
            .claim_ordered_text_input_events(id)
            .ok()
            .flatten()
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let replay = if access.is_disabled() {
        TextReplayResult::default()
    } else {
        replay_text_field_events(
            state,
            access,
            edit_mode,
            id,
            entry_accepts_input,
            entry_selection_anchor,
            retained_gesture_anchor,
            pointer_actions,
            ordered_events,
        )
    };
    if let Some(anchor) = replay.accepted_gesture_anchor {
        let _ = runtime
            .memory_mut()
            .set_selection_gesture_anchor(id, anchor);
    }
    if replay.focus_lost && runtime.memory().is_focused(id) {
        runtime.memory_mut().clear_focus();
    }

    response.state.focused = runtime.memory().is_focused(id) && !access.is_disabled();
    response.state.disabled = access.is_disabled();
    let recipe = theme.text_field(ComponentState {
        hovered: response.state.hovered,
        pressed: response.state.pressed,
        focused: response.state.focused,
        disabled: access.is_disabled(),
        selected: false,
    });
    let geometry =
        TextFieldGeometry::build(rect, state, &recipe, kind, retained_offset, text_layouts);

    if !access.is_disabled() {
        let wheel = text_wheel_delta(runtime.input(), runtime.memory(), id, rect, kind, false);
        let viewport = geometry.viewport();
        let mut candidate = viewport.scroll_by(wheel);
        let candidate_viewport = TextViewport::new(
            kind.viewport_mode(),
            viewport.viewport_size(),
            viewport.content_size(),
            candidate,
        );
        if response.state.focused {
            candidate = candidate_viewport.reveal(geometry.caret_content_rect());
        }
        if retained_offset != candidate {
            runtime.stage_scroll_offset(id, candidate);
            runtime.request_repaint(RepaintRequest::NextFrame);
        }
    }

    if access == TextFieldAccess::Editable
        && response.state.focused
        && !replay.focus_lost
        && let Some(caret) = geometry.visible_caret_rect()
    {
        let _ = runtime.publish_text_input_rect(id, caret);
    }

    let primitives = geometry.primitives(
        id,
        response.state.focused,
        !access.is_disabled(),
        caret_visible,
    );
    let widget = with_hover_cursor(
        WidgetOutput::new(Some(response), primitives)
            .with_semantic(with_response_state(
                text_field_semantics_with_access(
                    id,
                    rect,
                    "Text field",
                    state.text.clone(),
                    access,
                ),
                &response,
            ))
            .with_platform_requests(replay.ordered.platform_requests.iter().cloned()),
        &response,
        CursorShape::Text,
    );

    CanonicalTextFieldResult {
        widget,
        changed: before != state.text,
        ordered: replay.ordered,
    }
}

fn final_primary_press_is_unambiguous(
    actions: &[ResolvedTextPointerAction],
    final_ordinal: usize,
) -> bool {
    let mut primary_open = false;
    for action in actions.iter().filter(|action| {
        action
            .ordinal
            .is_some_and(|ordinal| ordinal < final_ordinal)
    }) {
        match action.phase {
            TextPointerPhase::Press => primary_open = true,
            TextPointerPhase::Release | TextPointerPhase::Cancel => primary_open = false,
            TextPointerPhase::Move => {}
        }
    }
    !primary_open
}
