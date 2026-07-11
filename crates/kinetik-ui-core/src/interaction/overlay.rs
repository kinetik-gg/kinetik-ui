use super::hit::HitTarget;
use super::{DropTargetResponse, InteractionState, Response, pressable, pressable_transformed};
use crate::{
    Key, KeyState, MouseButton, Rect, Transform, UiInput, UiInputEvent, UiMemory, WidgetId,
};

/// Resolves neutral context-menu trigger behavior.
pub fn context_menu_trigger(
    id: WidgetId,
    rect: Rect,
    input: &UiInput,
    memory: &mut UiMemory,
    disabled: bool,
) -> Response {
    let mut response = pressable(id, rect, input, memory, disabled);
    response.context_requested =
        !disabled && (response.secondary_clicked || keyboard_context_requested(id, input, memory));
    response
}

/// Resolves neutral context-menu trigger behavior with transformed local-space hit testing.
pub fn context_menu_trigger_transformed(
    id: WidgetId,
    rect: Rect,
    local_to_screen: Transform,
    input: &UiInput,
    memory: &mut UiMemory,
    disabled: bool,
) -> Response {
    let mut response = pressable_transformed(id, rect, local_to_screen, input, memory, disabled);
    response.context_requested =
        !disabled && (response.secondary_clicked || keyboard_context_requested(id, input, memory));
    response
}

/// Resolves neutral tooltip trigger behavior.
pub fn tooltip_trigger(
    id: WidgetId,
    rect: Rect,
    input: &UiInput,
    memory: &mut UiMemory,
    disabled: bool,
) -> Response {
    tooltip_trigger_with_hit_target(id, rect, HitTarget::Rect, input, memory, disabled)
}

/// Resolves neutral tooltip trigger behavior with transformed local-space hit testing.
pub fn tooltip_trigger_transformed(
    id: WidgetId,
    rect: Rect,
    local_to_screen: Transform,
    input: &UiInput,
    memory: &mut UiMemory,
    disabled: bool,
) -> Response {
    tooltip_trigger_with_hit_target(
        id,
        rect,
        HitTarget::Transformed(local_to_screen),
        input,
        memory,
        disabled,
    )
}

fn tooltip_trigger_with_hit_target(
    id: WidgetId,
    rect: Rect,
    hit_target: HitTarget,
    input: &UiInput,
    memory: &mut UiMemory,
    disabled: bool,
) -> Response {
    let hovered = !disabled
        && !memory.pointer_input_conflicted(input)
        && hit_target.routed_hit_test(id, rect, input, memory);
    if hovered {
        memory.set_hovered(id);
    }

    let mut response = Response::new(
        id,
        rect,
        InteractionState {
            hovered,
            focused: memory.is_focused(id),
            active: false,
            pressed: false,
            disabled,
            selected: false,
        },
    );
    response.tooltip_requested = hovered
        && !input.pointer.primary.down
        && !input.pointer.secondary.down
        && !input.pointer.middle.down;
    response
}

/// Resolves neutral drop-target behavior for active drags.
pub fn drop_target(
    id: WidgetId,
    rect: Rect,
    input: &UiInput,
    memory: &mut UiMemory,
    disabled: bool,
) -> DropTargetResponse {
    drop_target_with_hit_target(id, rect, HitTarget::Rect, input, memory, disabled)
}

/// Resolves neutral drop-target behavior for active drags with transformed local-space hit testing.
pub fn drop_target_transformed(
    id: WidgetId,
    rect: Rect,
    local_to_screen: Transform,
    input: &UiInput,
    memory: &mut UiMemory,
    disabled: bool,
) -> DropTargetResponse {
    drop_target_with_hit_target(
        id,
        rect,
        HitTarget::Transformed(local_to_screen),
        input,
        memory,
        disabled,
    )
}

fn drop_target_with_hit_target(
    id: WidgetId,
    rect: Rect,
    hit_target: HitTarget,
    input: &UiInput,
    memory: &mut UiMemory,
    disabled: bool,
) -> DropTargetResponse {
    let pointer_cancelled = memory.pointer_interaction_cancelled()
        || memory.pointer_input_conflicted(input)
        || canonical_pointer_cancelled(input);
    let source_candidate = memory
        .released_drag_source()
        .or_else(|| memory.drag_source())
        .filter(|source| *source != id);
    let target_hit = hit_target.hit_test(rect, input);
    let (release_seen, release_hit) = primary_release_hit(hit_target, rect, input, target_hit);
    let source_hit = if release_seen {
        release_hit
    } else {
        target_hit
    };
    let hovered = !pointer_cancelled
        && !disabled
        && if source_candidate.is_some() {
            source_hit && memory.pointer_drop_route_allows(id)
        } else {
            hit_target.routed_hit_test(id, rect, input, memory)
        };
    let source = if !pointer_cancelled
        && !disabled
        && source_candidate.is_some()
        && source_hit
        && memory.pointer_drop_route_allows(id)
    {
        source_candidate
    } else {
        None
    };
    if hovered {
        memory.set_hovered(id);
    }
    let dropped = !pointer_cancelled && !disabled && hovered && release_seen && source.is_some();
    let response = Response::new(
        id,
        rect,
        InteractionState {
            hovered,
            focused: memory.is_focused(id),
            active: false,
            pressed: false,
            disabled,
            selected: false,
        },
    );

    DropTargetResponse {
        response,
        source,
        dropped,
    }
}

fn primary_release_hit(
    hit_target: HitTarget,
    rect: Rect,
    input: &UiInput,
    legacy_target_hit: bool,
) -> (bool, bool) {
    if input.events.is_empty() {
        return (input.pointer.primary.released, legacy_target_hit);
    }

    let mut release_seen = false;
    let mut release_hit = false;
    for event in &input.events {
        if let UiInputEvent::PointerButton {
            button: MouseButton::Primary,
            down: false,
            position,
            ..
        } = event
        {
            release_seen = true;
            release_hit |= hit_target.hit_test_position(rect, *position);
        }
    }
    (release_seen, release_hit)
}

fn canonical_pointer_cancelled(input: &UiInput) -> bool {
    !input.events.is_empty()
        && input.events.iter().any(|event| {
            matches!(
                event,
                UiInputEvent::PointerReleaseAll { .. } | UiInputEvent::WindowFocusChanged(false)
            )
        })
}

fn keyboard_context_requested(id: WidgetId, input: &UiInput, memory: &UiMemory) -> bool {
    memory.is_focused(id)
        && input.keyboard.events.iter().any(|event| {
            event.state == KeyState::Pressed
                && event.modifiers.shift
                && matches!(event.key, Key::Function(10))
        })
}
