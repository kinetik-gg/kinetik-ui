//! Conformance coverage for non-mutating ordered text input.

use kinetik_ui_core::{
    ClipboardText, Key, KeyEvent, KeyState, Modifiers, PhysicalKey, PlatformRequest,
    TextInputEvent, TextRange, UiInputEvent, WidgetId,
};
use kinetik_ui_text::{TextComposition, TextEditMode, TextEditState, TextSelection};

fn key(key: Key, modifiers: Modifiers) -> UiInputEvent {
    UiInputEvent::Key(KeyEvent::new(key, KeyState::Pressed, modifiers, false))
}

fn physical_key(key: Key, physical_key: PhysicalKey, modifiers: Modifiers) -> UiInputEvent {
    UiInputEvent::Key(KeyEvent::with_physical_key(
        key,
        physical_key,
        KeyState::Pressed,
        modifiers,
        false,
    ))
}

fn command() -> Modifiers {
    Modifiers::new(false, true, false, false)
}

fn word_shift() -> Modifiers {
    Modifiers::new(true, true, false, false)
}

#[test]
fn scalar_and_word_navigation_preserve_text_and_extend_selection() {
    let mut scalar = TextEditState::new("one two");
    scalar.set_caret(4);
    let scalar_requests = scalar.apply_read_only_ordered_input(
        &[key(
            Key::ArrowRight,
            Modifiers::new(true, false, false, false),
        )],
        TextEditMode::SingleLine,
    );
    assert!(scalar_requests.is_empty());
    assert_eq!(scalar.text, "one two");
    assert_eq!(scalar.selection, TextSelection::new(4, 5));
    let scalar_collapse = scalar.apply_read_only_ordered_input(
        &[key(Key::ArrowLeft, Modifiers::default())],
        TextEditMode::SingleLine,
    );
    assert!(scalar_collapse.is_empty());
    assert_eq!(scalar.selection, TextSelection::new(4, 4));

    let mut right = TextEditState::new("one two");
    right.set_caret(0);
    let right_events = [
        key(Key::ArrowRight, command()),
        key(Key::ArrowRight, word_shift()),
    ];
    let right_requests =
        right.apply_read_only_ordered_input(&right_events, TextEditMode::SingleLine);
    assert!(right_requests.is_empty());
    assert_eq!(right.text, "one two");
    assert_eq!(right.selection, TextSelection::new(4, 7));

    let mut left = TextEditState::new("one two");
    let left_events = [
        key(Key::ArrowLeft, command()),
        key(Key::ArrowLeft, word_shift()),
    ];
    let left_requests = left.apply_read_only_ordered_input(&left_events, TextEditMode::SingleLine);

    assert!(left_requests.is_empty());
    assert_eq!(left.text, "one two");
    assert_eq!(left.selection, TextSelection::new(4, 0));
}

#[test]
fn multiline_navigation_uses_existing_line_and_vertical_policy() {
    let mut state = TextEditState::new("ab\ncdef\nxy");
    state.set_caret(5);
    let events = [
        key(Key::ArrowDown, command()),
        key(Key::ArrowUp, command()),
        key(Key::Home, command()),
        key(Key::End, Modifiers::new(true, true, false, false)),
    ];

    let _ = state.apply_read_only_ordered_input(&events, TextEditMode::MultiLine);

    assert_eq!(state.text, "ab\ncdef\nxy");
    assert_eq!(state.selection, TextSelection::new(3, 7));
}

#[test]
fn logical_select_all_and_logical_or_physical_copy_match_editable_shortcuts() {
    let mut state = TextEditState::new("copy me");
    let select_all = key(Key::Character("a".into()), command());
    let logical_copy = key(Key::Character("c".into()), command());
    let physical_copy = physical_key(Key::Unidentified, PhysicalKey::KeyC, command());

    let requests = state.apply_read_only_ordered_input(
        &[select_all, logical_copy, physical_copy],
        TextEditMode::SingleLine,
    );

    assert_eq!(state.selection, TextSelection::new(0, state.text.len()));
    assert_eq!(
        requests,
        vec![
            PlatformRequest::CopyToClipboard("copy me".into()),
            PlatformRequest::CopyToClipboard("copy me".into()),
        ]
    );
}

#[test]
fn physical_a_does_not_invent_a_select_all_fallback_and_empty_copy_is_silent() {
    let mut state = TextEditState::new("copy me");
    state.set_caret(2);
    let physical_a = physical_key(Key::Unidentified, PhysicalKey::KeyA, command());
    let copy = key(Key::Character("c".into()), command());

    let requests =
        state.apply_read_only_ordered_input(&[physical_a, copy], TextEditMode::SingleLine);

    assert_eq!(state.selection, TextSelection::new(2, 2));
    assert!(requests.is_empty());
}

#[test]
fn mutation_composition_clipboard_and_command_intents_are_all_rejected() {
    let target = WidgetId::from_key("read-only");
    let mut state = TextEditState::new("stable text");
    state.set_selection(TextSelection::new(0, 6));
    state.composition = Some(TextComposition::new("old", None));
    let mut hardware_text = KeyEvent::new(
        Key::Character("q".into()),
        KeyState::Pressed,
        Modifiers::default(),
        false,
    )
    .with_text("q");
    hardware_text.physical_key = PhysicalKey::KeyQ;
    let events = vec![
        UiInputEvent::Key(hardware_text),
        key(Key::Backspace, Modifiers::default()),
        key(Key::Delete, Modifiers::default()),
        key(Key::Backspace, command()),
        key(Key::Delete, command()),
        key(Key::Character("x".into()), command()),
        physical_key(Key::Unidentified, PhysicalKey::KeyX, command()),
        key(Key::Character("v".into()), command()),
        physical_key(Key::Unidentified, PhysicalKey::KeyV, command()),
        key(Key::Character("z".into()), command()),
        key(Key::Character("y".into()), command()),
        key(Key::Enter, Modifiers::default()),
        key(Key::Escape, Modifiers::default()),
        UiInputEvent::Text(TextInputEvent::CompositionStart),
        UiInputEvent::Text(TextInputEvent::Composition {
            text: "preedit".into(),
            selection: Some(TextRange::new(0, 3)),
        }),
        UiInputEvent::Text(TextInputEvent::Commit("committed".into())),
        UiInputEvent::Text(TextInputEvent::CompositionEnd),
        UiInputEvent::ClipboardText(ClipboardText::new(target, "pasted")),
    ];

    let requests = state.apply_read_only_ordered_input(&events, TextEditMode::MultiLine);

    assert_eq!(state.text, "stable text");
    assert_eq!(state.selection, TextSelection::new(0, 6));
    assert_eq!(state.composition, None);
    assert!(requests.is_empty());
}

#[test]
fn single_line_enter_and_escape_emit_no_commit_or_revert_effect() {
    let mut state = TextEditState::new("base");
    state.insert_text(" stable");
    state.set_selection(TextSelection::new(1, 4));
    state.composition = Some(TextComposition::new("old", None));
    let events = [
        key(Key::Enter, Modifiers::default()),
        key(Key::Escape, Modifiers::default()),
    ];

    let requests = state.apply_read_only_ordered_input(&events, TextEditMode::SingleLine);

    assert_eq!(state.text, "base stable");
    assert_eq!(state.selection, TextSelection::new(1, 4));
    assert_eq!(state.composition, None);
    assert!(requests.is_empty());

    assert!(state.undo());
    assert_eq!(state.text, "base");
}

#[test]
fn empty_entry_clears_composition_without_changing_selection_or_text() {
    let mut state = TextEditState::new("stable");
    state.set_selection(TextSelection::new(1, 4));
    state.composition = Some(TextComposition::new("old", None));

    let requests = state.apply_read_only_ordered_input(&[], TextEditMode::SingleLine);

    assert_eq!(state.text, "stable");
    assert_eq!(state.selection, TextSelection::new(1, 4));
    assert_eq!(state.composition, None);
    assert!(requests.is_empty());
}

#[test]
fn focus_loss_clears_composition_and_fences_gain_navigation_and_copy() {
    let mut state = TextEditState::new("stable");
    state.set_selection(TextSelection::new(0, 3));
    state.composition = Some(TextComposition::new("old", None));
    let events = [
        UiInputEvent::WindowFocusChanged(false),
        UiInputEvent::WindowFocusChanged(true),
        key(Key::ArrowRight, Modifiers::default()),
        key(Key::Character("c".into()), command()),
    ];

    let requests = state.apply_read_only_ordered_input(&events, TextEditMode::SingleLine);

    assert_eq!(state.selection, TextSelection::new(0, 3));
    assert_eq!(state.composition, None);
    assert!(requests.is_empty());
}

#[test]
fn read_only_pass_preserves_existing_undo_and_redo_histories() {
    let mut state = TextEditState::new("base");
    state.insert_text(" one");
    state.insert_text(" two");
    assert!(state.undo());
    assert_eq!(state.text, "base one");

    let baseline = state;
    let mut rejected_undo = baseline.clone();
    let undo_requests = rejected_undo.apply_read_only_ordered_input(
        &[key(Key::Character("z".into()), command())],
        TextEditMode::SingleLine,
    );
    assert!(undo_requests.is_empty());
    assert_eq!(rejected_undo, baseline);

    let mut rejected_redo = baseline.clone();
    let redo_requests = rejected_redo.apply_read_only_ordered_input(
        &[key(Key::Character("y".into()), command())],
        TextEditMode::SingleLine,
    );
    assert!(redo_requests.is_empty());
    assert_eq!(rejected_redo, baseline);

    let mut undo_branch = baseline.clone();
    let mut redo_branch = baseline;
    assert!(undo_branch.undo());
    assert_eq!(undo_branch.text, "base");
    assert!(redo_branch.redo());
    assert_eq!(redo_branch.text, "base one two");
}
