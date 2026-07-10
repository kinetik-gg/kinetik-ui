use kinetik_ui_core::{
    Key, KeyEvent, KeyState, Modifiers, PhysicalKey, Rect, TextInputEvent, UiInput, UiInputEvent,
    UiMemory, WidgetId, default_dark_theme,
};
use kinetik_ui_text::TextEditState;
use kinetik_ui_widgets::{multi_line_text_field, text_field};

fn hardware_event(key: Key, text: &str) -> UiInputEvent {
    UiInputEvent::Key(
        KeyEvent::with_physical_key(
            key,
            PhysicalKey::Unidentified,
            KeyState::Pressed,
            Modifiers::default(),
            false,
        )
        .with_text(text),
    )
}

fn focused_memory(id: WidgetId) -> UiMemory {
    let mut memory = UiMemory::new();
    memory.focus(id);
    memory.set_text_input_owner(id);
    memory
}

#[test]
fn focused_field_claims_canonical_input_once_even_when_called_repeatedly() {
    let id = WidgetId::from_key("field");
    let theme = default_dark_theme();
    let mut input = UiInput::default();
    input.push_event(hardware_event(Key::Character("x".to_owned()), "x"));
    let mut memory = focused_memory(id);
    let mut state = TextEditState::new("");

    let first = text_field(
        id,
        Rect::new(0.0, 0.0, 120.0, 24.0),
        &mut state,
        &input,
        &mut memory,
        &theme,
        false,
    );
    let second = text_field(
        id,
        Rect::new(0.0, 0.0, 120.0, 24.0),
        &mut state,
        &input,
        &mut memory,
        &theme,
        false,
    );

    assert!(first.changed);
    assert!(!second.changed);
    assert_eq!(state.text, "x");
}

#[test]
fn preclaim_handoff_routes_to_new_owner_and_postclaim_handoff_never_replays() {
    let first = WidgetId::from_key("first");
    let second = WidgetId::from_key("second");
    let theme = default_dark_theme();
    let mut input = UiInput::default();
    input.push_event(hardware_event(Key::Character("x".to_owned()), "x"));

    let mut preclaim_memory = focused_memory(first);
    preclaim_memory.focus(second);
    preclaim_memory.set_text_input_owner(second);
    let mut first_state = TextEditState::new("");
    let mut second_state = TextEditState::new("");
    let first_output = text_field(
        first,
        Rect::new(0.0, 0.0, 120.0, 24.0),
        &mut first_state,
        &input,
        &mut preclaim_memory,
        &theme,
        false,
    );
    let second_output = text_field(
        second,
        Rect::new(0.0, 30.0, 120.0, 24.0),
        &mut second_state,
        &input,
        &mut preclaim_memory,
        &theme,
        false,
    );
    assert!(!first_output.changed);
    assert!(second_output.changed);
    assert_eq!(first_state.text, "");
    assert_eq!(second_state.text, "x");

    let mut postclaim_memory = focused_memory(first);
    let mut first_state = TextEditState::new("");
    let mut second_state = TextEditState::new("");
    let first_output = text_field(
        first,
        Rect::new(0.0, 0.0, 120.0, 24.0),
        &mut first_state,
        &input,
        &mut postclaim_memory,
        &theme,
        false,
    );
    postclaim_memory.focus(second);
    postclaim_memory.set_text_input_owner(second);
    let second_output = text_field(
        second,
        Rect::new(0.0, 30.0, 120.0, 24.0),
        &mut second_state,
        &input,
        &mut postclaim_memory,
        &theme,
        false,
    );
    assert!(first_output.changed);
    assert!(!second_output.changed);
    assert_eq!(first_state.text, "x");
    assert_eq!(second_state.text, "");
}

#[test]
fn mixed_mode_conflict_claims_but_applies_no_ordered_editing() {
    let id = WidgetId::from_key("field");
    let theme = default_dark_theme();
    let mut input = UiInput::default();
    input.push_event(hardware_event(Key::Character("x".to_owned()), "x"));
    input
        .text_events
        .push(TextInputEvent::Commit("legacy".to_owned()));
    let mut memory = focused_memory(id);
    let mut state = TextEditState::new("base");

    let output = text_field(
        id,
        Rect::new(0.0, 0.0, 120.0, 24.0),
        &mut state,
        &input,
        &mut memory,
        &theme,
        false,
    );

    assert!(!output.changed);
    assert_eq!(state.text, "base");
    assert!(!memory.claim_text_input_events(id));
}

#[test]
fn canonical_multiline_enter_is_inserted_once_at_its_stream_position() {
    let id = WidgetId::from_key("multiline");
    let theme = default_dark_theme();
    let mut input = UiInput::default();
    input.push_event(hardware_event(Key::Character("a".to_owned()), "a"));
    input.push_event(hardware_event(Key::Enter, "\r"));
    input.push_event(hardware_event(Key::Character("b".to_owned()), "b"));
    let mut memory = focused_memory(id);
    let mut state = TextEditState::new("");

    let output = multi_line_text_field(
        id,
        Rect::new(0.0, 0.0, 120.0, 80.0),
        &mut state,
        &input,
        &mut memory,
        &theme,
        false,
    );

    assert!(output.changed);
    assert_eq!(state.text, "a\nb");
}
