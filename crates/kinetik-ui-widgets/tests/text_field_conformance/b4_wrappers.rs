#![allow(clippy::float_cmp)]

use kinetik_ui_core::{
    FrameOutput, Key, KeyEvent, KeyState, KeyboardInput, Modifiers, MouseButton, PlatformRequest,
    Point, PointerButtonState, PointerInput, Rect, SemanticActionKind, SemanticValue,
    TextInputEvent, UiInput, UiInputEvent, UiMemory, Vec2,
};
use kinetik_ui_text::{TextEditState, TextSelection};
use kinetik_ui_widgets::{
    NumericScrubInputConfig, PathFieldConfig, Ui, VectorComponentLayout, VectorScrubInputConfig,
    vector3_component_rects,
};

use super::{default_dark_theme, root_child};

const FIELD_RECT: Rect = Rect::new(0.0, 0.0, 160.0, 24.0);

fn press(x: f32, y: f32, click_count: u8) -> UiInputEvent {
    UiInputEvent::PointerButton {
        button: MouseButton::Primary,
        down: true,
        click_count,
        position: Some(Point::new(x, y)),
    }
}

fn release(x: f32, y: f32, click_count: u8) -> UiInputEvent {
    UiInputEvent::PointerButton {
        button: MouseButton::Primary,
        down: false,
        click_count,
        position: Some(Point::new(x, y)),
    }
}

fn moved(x: f32, y: f32, delta_x: f32) -> UiInputEvent {
    UiInputEvent::PointerMoved {
        position: Point::new(x, y),
        delta: Vec2::new(delta_x, 0.0),
    }
}

fn canonical(events: impl IntoIterator<Item = UiInputEvent>) -> UiInput {
    let mut input = UiInput::default();
    for event in events {
        input.push_event(event);
    }
    input
}

fn ctrl() -> Modifiers {
    Modifiers::new(false, true, false, false)
}

fn shift() -> Modifiers {
    Modifiers::new(true, false, false, false)
}

fn copy_input() -> UiInput {
    UiInput {
        keyboard: KeyboardInput {
            modifiers: ctrl(),
            events: vec![KeyEvent::new(
                Key::Character("c".to_owned()),
                KeyState::Pressed,
                ctrl(),
                false,
            )],
        },
        ..UiInput::default()
    }
}

fn has_action(node: &kinetik_ui_core::SemanticNode, action: &SemanticActionKind) -> bool {
    node.actions
        .iter()
        .any(|candidate| candidate.kind == *action)
}

fn assert_caret_start(frame: &FrameOutput, field_rect: Rect) {
    let caret = frame
        .platform_requests
        .iter()
        .find_map(|request| match request {
            PlatformRequest::StartTextInput { rect: Some(rect) } => Some(*rect),
            _ => None,
        })
        .expect("wrapper starts text input with caret geometry");
    assert_eq!(caret.width, 1.0);
    assert!(caret.height > 0.0);
    assert_ne!(caret, field_rect);
    assert!(field_rect.intersection(caret).is_some());
}

#[test]
fn clicked_scrub_places_caret_at_release_and_only_replays_later_text() {
    let theme = default_dark_theme();
    let input = canonical([
        press(8.0, 8.0, 1),
        UiInputEvent::Text(TextInputEvent::Commit("X".to_owned())),
        release(9.0, 8.0, 1),
        UiInputEvent::Text(TextInputEvent::Commit("Y".to_owned())),
    ]);
    let mut memory = UiMemory::new();
    let mut state = TextEditState::new("12");
    let mut value = 12.0;
    let mut ui = Ui::new(&input, &mut memory, &theme);

    let output = ui.numeric_scrub_input(
        "number",
        FIELD_RECT,
        &mut value,
        &mut state,
        NumericScrubInputConfig::new(1.0),
    );
    let frame = ui.finish_output();

    assert!(!output.scrubbed);
    assert_eq!(value, 12.0);
    assert!(!state.text.contains('X'));
    assert!(state.text.contains('Y'));
    assert_eq!(memory.focused(), Some(root_child("number")));
    assert_eq!(memory.text_input_owner(), Some(root_child("number")));
    assert_caret_start(&frame, FIELD_RECT);
}

#[test]
fn clicked_scrub_rejects_preplacement_text_and_supports_multiframe_snapshots() {
    let theme = default_dark_theme();
    for events in [
        vec![
            UiInputEvent::Text(TextInputEvent::Commit("X".to_owned())),
            press(8.0, 8.0, 1),
            release(9.0, 8.0, 1),
        ],
        vec![
            press(8.0, 8.0, 1),
            release(9.0, 8.0, 1),
            UiInputEvent::Text(TextInputEvent::Commit("Y".to_owned())),
        ],
    ] {
        let input = canonical(events);
        let has_post_text = input.events.iter().any(
            |event| matches!(event, UiInputEvent::Text(TextInputEvent::Commit(text)) if text == "Y"),
        );
        let mut memory = UiMemory::new();
        let mut state = TextEditState::new("12");
        let mut value = 12.0;
        let mut ui = Ui::new(&input, &mut memory, &theme);
        ui.numeric_scrub_input(
            "number",
            FIELD_RECT,
            &mut value,
            &mut state,
            NumericScrubInputConfig::new(1.0),
        );
        let _ = ui.finish_output();
        assert!(!state.text.contains('X'));
        assert_eq!(state.text.contains('Y'), has_post_text);
    }

    let mut memory = UiMemory::new();
    let mut state = TextEditState::new("12");
    let mut value = 12.0;
    let pressed = UiInput {
        pointer: PointerInput {
            position: Some(Point::new(8.0, 8.0)),
            primary: PointerButtonState::new(true, true, false),
            ..PointerInput::default()
        },
        ..UiInput::default()
    };
    let mut ui = Ui::new(&pressed, &mut memory, &theme);
    ui.numeric_scrub_input(
        "number",
        FIELD_RECT,
        &mut value,
        &mut state,
        NumericScrubInputConfig::new(1.0),
    );
    let _ = ui.finish_output();
    assert_eq!(memory.focused(), None);

    let released = UiInput {
        pointer: PointerInput {
            position: Some(Point::new(9.0, 8.0)),
            primary: PointerButtonState::new(false, false, true),
            ..PointerInput::default()
        },
        ..UiInput::default()
    };
    let mut ui = Ui::new(&released, &mut memory, &theme);
    ui.numeric_scrub_input(
        "number",
        FIELD_RECT,
        &mut value,
        &mut state,
        NumericScrubInputConfig::new(1.0),
    );
    let frame = ui.finish_output();
    assert_eq!(memory.focused(), Some(root_child("number")));
    assert_caret_start(&frame, FIELD_RECT);
}

#[test]
fn real_scrub_uses_causal_move_and_never_activates_unfocused_text() {
    let theme = default_dark_theme();
    let input = canonical([
        press(8.0, 8.0, 1),
        UiInputEvent::ModifiersChanged(shift()),
        moved(16.0, 8.0, 8.0),
        UiInputEvent::ModifiersChanged(ctrl()),
        UiInputEvent::Text(TextInputEvent::Commit("X".to_owned())),
        release(16.0, 8.0, 1),
    ]);
    let mut memory = UiMemory::new();
    let mut state = TextEditState::new("2");
    let mut value = 2.0;
    let config = NumericScrubInputConfig::new(1.0)
        .with_fine_step(0.25)
        .with_coarse_step(5.0);
    let mut ui = Ui::new(&input, &mut memory, &theme);

    let output = ui.numeric_scrub_input("number", FIELD_RECT, &mut value, &mut state, config);
    let frame = ui.finish_output();

    assert!(output.scrub_response.dragged);
    assert!(output.scrubbed);
    assert_eq!(output.step, 0.25);
    assert_eq!(value, 4.0);
    assert_eq!(state.text, "4");
    assert_eq!(memory.focused(), None);
    assert!(
        !frame
            .platform_requests
            .iter()
            .any(|request| matches!(request, PlatformRequest::StartTextInput { .. }))
    );
}

#[test]
fn release_only_threshold_crossing_uses_release_modifiers_not_final_snapshot() {
    let theme = default_dark_theme();
    let input = canonical([
        press(8.0, 8.0, 1),
        UiInputEvent::ModifiersChanged(shift()),
        release(13.0, 8.0, 1),
        UiInputEvent::ModifiersChanged(ctrl()),
    ]);
    let mut memory = UiMemory::new();
    let mut state = TextEditState::new("10");
    let mut value = 10.0;
    let config = NumericScrubInputConfig::new(1.0)
        .with_fine_step(0.25)
        .with_coarse_step(5.0);
    let mut ui = Ui::new(&input, &mut memory, &theme);

    let output = ui.numeric_scrub_input("number", FIELD_RECT, &mut value, &mut state, config);
    let _ = ui.finish_output();

    assert!(output.scrubbed);
    assert_eq!(output.step, 0.25);
    assert_eq!(value, 11.25);
    assert_eq!(state.text, "11.25");
    assert_eq!(memory.focused(), None);
}

#[test]
fn multiple_domain_transactions_never_borrow_aggregate_drag_evidence() {
    let theme = default_dark_theme();
    for (events, expect_focus) in [
        (
            vec![
                press(8.0, 8.0, 1),
                moved(14.0, 8.0, 6.0),
                release(14.0, 8.0, 1),
                press(8.0, 8.0, 1),
                release(9.0, 8.0, 1),
            ],
            true,
        ),
        (
            vec![
                press(8.0, 8.0, 1),
                release(9.0, 8.0, 1),
                press(8.0, 8.0, 1),
                moved(14.0, 8.0, 6.0),
                release(14.0, 8.0, 1),
            ],
            false,
        ),
    ] {
        let input = canonical(events);
        let mut memory = UiMemory::new();
        let mut state = TextEditState::new("3");
        let mut value = 3.0;
        let mut ui = Ui::new(&input, &mut memory, &theme);
        let output = ui.numeric_scrub_input(
            "number",
            FIELD_RECT,
            &mut value,
            &mut state,
            NumericScrubInputConfig::new(1.0),
        );
        let _ = ui.finish_output();

        assert!(output.scrub_response.dragged);
        assert!(!output.scrubbed);
        assert_eq!(value, 3.0);
        assert_eq!(state.text, "3");
        assert_eq!(memory.focused() == Some(root_child("number")), expect_focus);
    }
}

#[test]
fn later_press_outside_scrub_revokes_earlier_drag_authority() {
    let theme = default_dark_theme();
    let input = canonical([
        press(8.0, 8.0, 1),
        moved(14.0, 8.0, 6.0),
        release(14.0, 8.0, 1),
        press(220.0, 8.0, 1),
    ]);
    let mut memory = UiMemory::new();
    let mut state = TextEditState::new("3");
    let mut value = 3.0;
    let mut ui = Ui::new(&input, &mut memory, &theme);
    let output = ui.numeric_scrub_input(
        "number",
        FIELD_RECT,
        &mut value,
        &mut state,
        NumericScrubInputConfig::new(1.0),
    );
    let _ = ui.finish_output();

    assert!(output.scrub_response.dragged);
    assert!(!output.scrubbed);
    assert_eq!(value, 3.0);
    assert_eq!(state.text, "3");
    assert_eq!(memory.focused(), None);
}

#[test]
fn read_only_and_disabled_scrubs_have_exact_access_semantics() {
    let theme = default_dark_theme();
    let id = root_child("number");

    let mut read_only_memory = UiMemory::new();
    read_only_memory.focus(id);
    let mut read_only_state = TextEditState::new("42");
    read_only_state.set_selection(TextSelection::new(0, 2));
    let mut read_only_value = 42.0;
    let input = copy_input();
    let mut ui = Ui::new(&input, &mut read_only_memory, &theme);
    let read_only = ui.numeric_scrub_input(
        "number",
        FIELD_RECT,
        &mut read_only_value,
        &mut read_only_state,
        NumericScrubInputConfig::new(1.0).read_only(true),
    );
    let frame = ui.finish_output();
    let node = frame.semantics.get(id).expect("read-only numeric node");

    assert!(read_only.read_only);
    assert_eq!(read_only_value, 42.0);
    assert_eq!(read_only_state.text, "42");
    assert!(node.focusable);
    assert!(!node.state.disabled);
    assert_eq!(
        node.state.value,
        Some(SemanticValue::Number {
            current: 42.0,
            min: 42.0,
            max: 42.0,
        })
    );
    assert!(!has_action(node, &SemanticActionKind::SetText));
    assert!(!has_action(node, &SemanticActionKind::SetValue));
    assert!(
        frame
            .platform_requests
            .contains(&PlatformRequest::CopyToClipboard("42".to_owned()))
    );
    assert!(
        !frame
            .platform_requests
            .iter()
            .any(|request| matches!(request, PlatformRequest::StartTextInput { .. }))
    );

    let mut disabled_memory = UiMemory::new();
    disabled_memory.focus(id);
    disabled_memory.set_text_input_owner(id);
    let mut disabled_state = TextEditState::new("7");
    let mut disabled_value = 7.0;
    let disabled_input = UiInput::default();
    let mut ui = Ui::new(&disabled_input, &mut disabled_memory, &theme);
    let disabled = ui.numeric_scrub_input(
        "number",
        FIELD_RECT,
        &mut disabled_value,
        &mut disabled_state,
        NumericScrubInputConfig::new(1.0)
            .disabled(true)
            .read_only(true),
    );
    let frame = ui.finish_output();
    let node = frame.semantics.get(id).expect("disabled numeric node");
    assert!(disabled.read_only);
    assert!(node.state.disabled);
    assert!(!node.focusable);
    assert!(!has_action(node, &SemanticActionKind::SetText));
    assert!(!has_action(node, &SemanticActionKind::SetValue));
    assert_eq!(disabled_memory.focused(), None);
    assert_eq!(disabled_memory.text_input_owner(), None);
}

#[test]
fn invalid_editable_scrub_keeps_text_semantics_without_set_value() {
    let theme = default_dark_theme();
    let input = UiInput::default();
    let mut memory = UiMemory::new();
    let mut state = TextEditState::new("bad draft");
    let mut value = 7.0;
    let mut ui = Ui::new(&input, &mut memory, &theme);
    ui.numeric_scrub_input(
        "number",
        FIELD_RECT,
        &mut value,
        &mut state,
        NumericScrubInputConfig::new(1.0),
    );
    let frame = ui.finish_output();
    let node = frame
        .semantics
        .get(root_child("number"))
        .expect("numeric semantics");
    assert_eq!(
        node.state.value,
        Some(SemanticValue::Text("bad draft".to_owned()))
    );
    assert!(has_action(node, &SemanticActionKind::SetText));
    assert!(!has_action(node, &SemanticActionKind::SetValue));
}

#[test]
fn ui_numeric_commit_and_revert_intents_remain_editable_only() {
    let theme = default_dark_theme();
    let id = root_child("number");

    let input = canonical([UiInputEvent::Key(KeyEvent::new(
        Key::Enter,
        KeyState::Pressed,
        Modifiers::default(),
        false,
    ))]);
    let mut memory = UiMemory::new();
    memory.focus(id);
    let mut state = TextEditState::new("42");
    let mut ui = Ui::new(&input, &mut memory, &theme);
    let output = ui.numeric_input("number", FIELD_RECT, &mut state, false);
    let _ = ui.finish_output();
    assert!(output.policy.commit_requested);
    assert!(!output.policy.revert_requested);

    let input = canonical([UiInputEvent::Key(KeyEvent::new(
        Key::Escape,
        KeyState::Pressed,
        Modifiers::default(),
        false,
    ))]);
    let mut memory = UiMemory::new();
    memory.focus(id);
    let mut state = TextEditState::new("bad draft");
    let mut ui = Ui::new(&input, &mut memory, &theme);
    let output = ui.numeric_input("number", FIELD_RECT, &mut state, false);
    let _ = ui.finish_output();
    assert!(!output.policy.commit_requested);
    assert!(output.policy.revert_requested);

    let mut memory = UiMemory::new();
    memory.focus(id);
    let mut state = TextEditState::new("42");
    let mut ui = Ui::new(&input, &mut memory, &theme);
    let output = ui.numeric_input("number", FIELD_RECT, &mut state, true);
    let _ = ui.finish_output();
    assert!(!output.policy.commit_requested);
    assert!(!output.policy.revert_requested);
}

#[test]
fn search_and_numeric_wrappers_share_one_ordered_owner_in_both_call_orders() {
    let theme = default_dark_theme();
    let search_rect = Rect::new(0.0, 0.0, 160.0, 24.0);
    let numeric_rect = Rect::new(0.0, 32.0, 160.0, 24.0);
    let input = canonical([
        press(8.0, 40.0, 1),
        UiInputEvent::Text(TextInputEvent::Commit("X".to_owned())),
    ]);

    for numeric_first in [false, true] {
        let mut memory = UiMemory::new();
        let mut search_state = TextEditState::new("find");
        let mut numeric_state = TextEditState::new("12");
        let mut ui = Ui::new(&input, &mut memory, &theme);
        if numeric_first {
            ui.numeric_input("number", numeric_rect, &mut numeric_state, false);
            ui.search_field("search", search_rect, &mut search_state, false);
        } else {
            ui.search_field("search", search_rect, &mut search_state, false);
            ui.numeric_input("number", numeric_rect, &mut numeric_state, false);
        }
        let frame = ui.finish_output();

        assert_eq!(search_state.text, "find");
        assert!(numeric_state.text.contains('X'));
        assert_eq!(memory.focused(), Some(root_child("number")));
        assert_eq!(memory.text_input_owner(), Some(root_child("number")));
        assert_eq!(
            frame
                .platform_requests
                .iter()
                .filter(|request| matches!(request, PlatformRequest::StartTextInput { .. }))
                .count(),
            1
        );
        assert!(frame.warnings.is_empty());
    }
}

#[test]
fn path_runtime_preserves_child_ids_copy_policy_and_open_intent() {
    let theme = default_dark_theme();
    let path_id = root_child("path");
    let text_id = path_id.child("text");
    let browse_id = path_id.child("browse");
    let mut memory = UiMemory::new();
    memory.focus(text_id);
    let mut state = TextEditState::new("src/main.rs");
    state.set_selection(TextSelection::new(0, 3));
    let input = copy_input();
    let mut ui = Ui::new(&input, &mut memory, &theme);
    let output = ui.path_field(
        "path",
        FIELD_RECT,
        "Source",
        &mut state,
        PathFieldConfig::new().read_only(true).open(true),
    );
    let frame = ui.finish_output();

    assert!(!output.browse_requested);
    assert!(!output.open_requested);
    assert!(frame.semantics.get(text_id).is_some());
    let browse = frame.semantics.get(browse_id).expect("browse semantic");
    assert!(browse.state.disabled);
    assert!(
        frame
            .platform_requests
            .contains(&PlatformRequest::CopyToClipboard("src".to_owned()))
    );
    assert!(frame.warnings.is_empty());

    let input = canonical([press(8.0, 8.0, 2), release(8.0, 8.0, 2)]);
    let mut memory = UiMemory::new();
    let mut state = TextEditState::new("src/main.rs");
    let mut ui = Ui::new(&input, &mut memory, &theme);
    let output = ui.path_field(
        "path",
        FIELD_RECT,
        "Source",
        &mut state,
        PathFieldConfig::new().open(true),
    );
    let frame = ui.finish_output();
    assert!(output.open_requested);
    assert!(!output.browse_requested);
    assert!(!state.selection.is_caret());
    assert!(frame.warnings.is_empty());
}

#[test]
fn vector_runtime_isolates_target_and_read_only_copy_by_exact_child_id() {
    let theme = default_dark_theme();
    let rect = Rect::new(0.0, 0.0, 240.0, 24.0);
    let component_rects = vector3_component_rects(rect, VectorComponentLayout::default());
    let target = component_rects[1].value_rect.center();
    let input = canonical([
        press(target.x, target.y, 1),
        moved(target.x + 6.0, target.y, 6.0),
        release(target.x + 6.0, target.y, 1),
    ]);
    let mut memory = UiMemory::new();
    let mut values = [1.0, 2.0, 3.0];
    let mut states = [
        TextEditState::new("1"),
        TextEditState::new("2"),
        TextEditState::new("3"),
    ];
    let mut ui = Ui::new(&input, &mut memory, &theme);
    let output = ui.vector3_scrub_input(
        "vector",
        rect,
        "Position",
        &mut values,
        &mut states,
        VectorScrubInputConfig::new(NumericScrubInputConfig::new(0.5)),
    );
    let frame = ui.finish_output();
    assert!(output.components[1].scrubbed);
    assert_eq!(values, [1.0, 5.0, 3.0]);
    assert_eq!(states[0].text, "1");
    assert_eq!(states[1].text, "5");
    assert_eq!(states[2].text, "3");
    assert!(
        frame
            .semantics
            .get(root_child("vector").child("X"))
            .is_some()
    );
    assert!(
        frame
            .semantics
            .get(root_child("vector").child("Y"))
            .is_some()
    );
    assert!(
        frame
            .semantics
            .get(root_child("vector").child("Z"))
            .is_some()
    );
    assert!(frame.warnings.is_empty());

    let y_id = root_child("vector").child("Y");
    let mut memory = UiMemory::new();
    memory.focus(y_id);
    let mut values = [1.0, 2.0, 3.0];
    let mut states = [
        TextEditState::new("1"),
        TextEditState::new("2"),
        TextEditState::new("3"),
    ];
    states[1].set_selection(TextSelection::new(0, 1));
    let input = copy_input();
    let mut ui = Ui::new(&input, &mut memory, &theme);
    let output = ui.vector3_scrub_input(
        "vector",
        rect,
        "Position",
        &mut values,
        &mut states,
        VectorScrubInputConfig::new(NumericScrubInputConfig::new(0.5)).read_only(true),
    );
    let frame = ui.finish_output();
    let node = frame.semantics.get(y_id).expect("Y semantic");
    assert!(output.read_only);
    assert_eq!(values, [1.0, 2.0, 3.0]);
    assert!(node.focusable);
    assert!(!node.state.disabled);
    assert!(!has_action(node, &SemanticActionKind::SetText));
    assert!(!has_action(node, &SemanticActionKind::SetValue));
    assert!(
        frame
            .platform_requests
            .contains(&PlatformRequest::CopyToClipboard("2".to_owned()))
    );
    assert!(frame.warnings.is_empty());
}

#[test]
fn narrow_canonical_wrappers_never_emit_zero_area_text_clips() {
    let theme = default_dark_theme();
    let input = UiInput::default();
    let mut memory = UiMemory::new();
    let mut values = [1.0, 2.0, 3.0];
    let mut states = [
        TextEditState::new("1"),
        TextEditState::new("2"),
        TextEditState::new("3"),
    ];
    let mut ui = Ui::new(&input, &mut memory, &theme);
    ui.vector3_scrub_input(
        "vector",
        Rect::new(0.0, 0.0, 30.0, 24.0),
        "Position",
        &mut values,
        &mut states,
        VectorScrubInputConfig::default(),
    );
    let frame = ui.finish_output();
    assert!(frame.primitives.iter().all(|primitive| {
        !matches!(
            primitive,
            kinetik_ui_core::Primitive::ClipBegin { rect, .. }
                if rect.width <= 0.0 || rect.height <= 0.0
        )
    }));
    assert!(frame.warnings.is_empty());
}

#[test]
fn every_migrated_wrapper_publishes_caret_geometry_not_field_geometry() {
    let theme = default_dark_theme();

    let input = canonical([press(8.0, 8.0, 1)]);
    let mut memory = UiMemory::new();
    let mut state = TextEditState::new("query");
    let mut ui = Ui::new(&input, &mut memory, &theme);
    ui.search_field("search", FIELD_RECT, &mut state, false);
    assert_caret_start(&ui.finish_output(), FIELD_RECT);

    let mut memory = UiMemory::new();
    let mut state = TextEditState::new("42");
    let mut ui = Ui::new(&input, &mut memory, &theme);
    ui.numeric_input("number", FIELD_RECT, &mut state, false);
    assert_caret_start(&ui.finish_output(), FIELD_RECT);

    let mut memory = UiMemory::new();
    let mut state = TextEditState::new("src/lib.rs");
    let mut ui = Ui::new(&input, &mut memory, &theme);
    let path = ui.path_field(
        "path",
        FIELD_RECT,
        "Source",
        &mut state,
        PathFieldConfig::new(),
    );
    assert_caret_start(
        &ui.finish_output(),
        path.field.widget.response.unwrap().rect,
    );

    let vector_rect = Rect::new(0.0, 0.0, 240.0, 24.0);
    let component_rects = vector3_component_rects(vector_rect, VectorComponentLayout::default());
    let target = component_rects[0].value_rect.center();
    let input = canonical([
        press(target.x, target.y, 1),
        release(target.x + 1.0, target.y, 1),
    ]);
    let mut memory = UiMemory::new();
    let mut values = [1.0, 2.0, 3.0];
    let mut states = [
        TextEditState::new("1"),
        TextEditState::new("2"),
        TextEditState::new("3"),
    ];
    let mut ui = Ui::new(&input, &mut memory, &theme);
    ui.vector3_scrub_input(
        "vector",
        vector_rect,
        "Position",
        &mut values,
        &mut states,
        VectorScrubInputConfig::default(),
    );
    assert_caret_start(&ui.finish_output(), component_rects[0].value_rect);
}

#[test]
fn domain_drag_access_transition_fences_then_recovers_selection() {
    let theme = default_dark_theme();
    let mut memory = UiMemory::new();
    let mut state = TextEditState::new("12");
    let mut value = 12.0;

    let input = canonical([press(8.0, 8.0, 1)]);
    let mut ui = Ui::new(&input, &mut memory, &theme);
    ui.numeric_scrub_input(
        "number",
        FIELD_RECT,
        &mut value,
        &mut state,
        NumericScrubInputConfig::new(1.0),
    );
    let _ = ui.finish_output();
    assert_eq!(memory.focused(), None);

    let input = canonical([moved(10.0, 8.0, 2.0)]);
    let mut ui = Ui::new(&input, &mut memory, &theme);
    let transition = ui.numeric_scrub_input(
        "number",
        FIELD_RECT,
        &mut value,
        &mut state,
        NumericScrubInputConfig::new(1.0).read_only(true),
    );
    let frame = ui.finish_output();
    assert!(!transition.scrubbed);
    assert_eq!(memory.focused(), None);
    assert!(
        !frame
            .platform_requests
            .iter()
            .any(|request| matches!(request, PlatformRequest::StartTextInput { .. }))
    );

    let input = canonical([press(8.0, 8.0, 1), release(9.0, 8.0, 1)]);
    let mut ui = Ui::new(&input, &mut memory, &theme);
    let recovered = ui.numeric_scrub_input(
        "number",
        FIELD_RECT,
        &mut value,
        &mut state,
        NumericScrubInputConfig::new(1.0).read_only(true),
    );
    let frame = ui.finish_output();
    assert!(!recovered.scrubbed);
    assert_eq!(memory.focused(), Some(root_child("number")));
    assert!(
        !frame
            .platform_requests
            .iter()
            .any(|request| matches!(request, PlatformRequest::StartTextInput { .. }))
    );
}

#[test]
fn nonfinite_and_overflowing_scrub_arithmetic_fails_closed() {
    let theme = default_dark_theme();

    let input = canonical([press(8.0, 8.0, 1), moved(14.0, 8.0, 6.0)]);
    let mut memory = UiMemory::new();
    let mut state = TextEditState::new("2");
    let mut value = 2.0;
    let mut ui = Ui::new(&input, &mut memory, &theme);
    ui.numeric_scrub_input(
        "number",
        FIELD_RECT,
        &mut value,
        &mut state,
        NumericScrubInputConfig::new(1.0),
    );
    let _ = ui.finish_output();
    assert_eq!(value, 8.0);

    let input = canonical([UiInputEvent::PointerMoved {
        position: Point::new(16.0, 8.0),
        delta: Vec2::new(f32::INFINITY, 0.0),
    }]);
    let mut ui = Ui::new(&input, &mut memory, &theme);
    let output = ui.numeric_scrub_input(
        "number",
        FIELD_RECT,
        &mut value,
        &mut state,
        NumericScrubInputConfig::new(1.0),
    );
    let _ = ui.finish_output();
    assert!(output.scrub_response.dragged);
    assert!(!output.scrubbed);
    assert_eq!(value, 8.0);
    assert_eq!(state.text, "8");

    let input = canonical([press(8.0, 8.0, 1), moved(14.0, 8.0, 6.0)]);
    let mut memory = UiMemory::new();
    let mut state = TextEditState::new("2");
    let mut value = 2.0;
    let mut ui = Ui::new(&input, &mut memory, &theme);
    let output = ui.numeric_scrub_input(
        "number",
        FIELD_RECT,
        &mut value,
        &mut state,
        NumericScrubInputConfig::new(f32::MAX),
    );
    let _ = ui.finish_output();
    assert!(output.scrub_response.dragged);
    assert!(!output.scrubbed);
    assert_eq!(value, 2.0);
    assert_eq!(state.text, "2");
}

#[test]
fn browse_press_preempts_path_text_owner_and_emits_only_browse() {
    let theme = default_dark_theme();
    let text_id = root_child("path").child("text");
    let mut memory = UiMemory::new();
    memory.focus(text_id);
    memory.set_text_input_owner(text_id);
    let mut state = TextEditState::new("src/main.rs");
    let input = canonical([press(140.0, 8.0, 1), release(140.0, 8.0, 1)]);
    let mut ui = Ui::new(&input, &mut memory, &theme);
    let output = ui.path_field(
        "path",
        FIELD_RECT,
        "Source",
        &mut state,
        PathFieldConfig::new().open(true),
    );
    let frame = ui.finish_output();

    assert!(output.browse_requested);
    assert!(!output.open_requested);
    assert_eq!(memory.focused(), None);
    assert_eq!(memory.text_input_owner(), None);
    assert!(frame.warnings.is_empty());
}
