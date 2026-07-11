//! Public conformance for UAX #29 editing and caret-affinity state.

use kinetik_ui_core::{
    Key, KeyEvent, KeyState, Modifiers, PlatformRequest, TextInputEvent, TextRange, UiInputEvent,
    WidgetId,
};
use kinetik_ui_text::{
    TextAffinity, TextCaret, TextComposition, TextEditMode, TextEditState, TextSelection,
};
use unicode_segmentation::UnicodeSegmentation;

fn key(key: Key, modifiers: Modifiers) -> UiInputEvent {
    UiInputEvent::Key(KeyEvent::new(key, KeyState::Pressed, modifiers, false))
}

#[test]
fn combining_graphemes_are_atomic_for_clamp_navigation_and_deletion() {
    let text = "Ae\u{301}B";
    let mut state = TextEditState::new(text);

    state.set_caret(3);
    assert_eq!(state.caret(), 1);
    state.move_right();
    assert_eq!(state.caret(), 4);
    state.move_left();
    assert_eq!(state.caret(), 1);
    state.move_left();
    assert_eq!(state.caret(), 0);

    let mut backward = TextEditState::new(text);
    backward.set_caret(4);
    backward.backspace();
    assert_eq!(backward.text, "AB");
    assert_eq!(backward.caret(), 1);

    let mut forward = TextEditState::new(text);
    forward.set_caret(1);
    forward.delete_forward();
    assert_eq!(forward.text, "AB");
    assert_eq!(forward.caret(), 1);

    let mut clamped = TextEditState::new(text);
    clamped.set_selection(TextSelection::new(2, 3));
    assert_eq!(clamped.selection, TextSelection::new(1, 1));
}

#[test]
fn emoji_modifier_flag_and_zwj_sequences_are_single_editing_units() {
    for grapheme in ["👍🏽", "🇮🇩", "👩‍🚀"] {
        let text = format!("A{grapheme}B");
        let after = 1 + grapheme.len();
        let mut state = TextEditState::new(&text);

        state.set_caret(after);
        state.move_left();
        assert_eq!(state.caret(), 1, "left enters {grapheme:?} atomically");
        state.move_right();
        assert_eq!(state.caret(), after, "right exits {grapheme:?} atomically");
        state.backspace();
        assert_eq!(
            state.text, "AB",
            "backspace removes {grapheme:?} atomically"
        );

        let mut forward = TextEditState::new(&text);
        forward.set_caret(1);
        forward.delete_forward();
        assert_eq!(forward.text, "AB", "delete removes {grapheme:?} atomically");
    }
}

#[test]
fn crlf_and_multiline_columns_use_extended_graphemes() {
    let mut line_break = TextEditState::new("A\r\nB");
    line_break.set_caret("A\r\n".len());
    line_break.move_left();
    assert_eq!(line_break.caret(), 1);
    line_break.set_caret("A\r\n".len());
    line_break.backspace();
    assert_eq!(line_break.text, "AB");

    let text = "Ae\u{301}\r\nX👩‍🚀Y";
    let first_line_end = "Ae\u{301}".len();
    let second_line_target = "Ae\u{301}\r\nX👩‍🚀".len();
    let mut state = TextEditState::new(text);
    state.set_caret(first_line_end);

    state.move_line_down();
    assert_eq!(state.caret(), second_line_target);
    assert_eq!(state.caret_position().affinity, TextAffinity::After);
    state.move_line_up();
    assert_eq!(state.caret(), first_line_end);
    assert_eq!(state.caret_position().affinity, TextAffinity::After);

    state.set_caret(1);
    state.move_line_end();
    assert_eq!(state.caret(), first_line_end);
    assert_eq!(state.caret_position().affinity, TextAffinity::Before);
    state.move_line_home();
    assert_eq!(state.caret(), 0);
    assert_eq!(state.caret_position().affinity, TextAffinity::After);
}

#[test]
fn full_buffer_uax_word_ties_are_deterministic() {
    let text = "can't!? \u{2003}café";
    let punctuation = text
        .split_word_bound_indices()
        .find(|(start, _)| *start == 5)
        .expect("UAX segment begins at punctuation");
    let punctuation_end = punctuation.0 + punctuation.1.len();

    let mut state = TextEditState::new(text);
    state.set_caret(3);
    state.move_word_left();
    assert_eq!(state.caret(), 0);
    state.set_caret(3);
    state.move_word_right();
    assert_eq!(state.caret(), 5);

    state.set_caret(5);
    state.move_word_left();
    assert_eq!(state.caret(), 0);
    state.set_caret(5);
    state.move_word_right();
    assert_eq!(state.caret(), punctuation_end);
    state.select_word_at(5);
    assert_eq!(state.selected_text(), Some(punctuation.1));

    state.set_caret(8);
    state.move_word_right();
    assert_eq!(state.caret(), 11);
    state.set_caret(text.len());
    state.move_word_left();
    assert_eq!(state.caret(), 11);
}

#[test]
fn unicode_word_extension_and_deletion_use_logical_uax_spans() {
    let text = "café  crème";
    let second_word = "café  ".len();
    let mut state = TextEditState::new(text);

    state.set_caret(0);
    state.extend_word_right();
    assert_eq!(state.selection, TextSelection::new(0, second_word));
    state.extend_word_right();
    assert_eq!(state.selection, TextSelection::new(0, text.len()));

    let mut forward = TextEditState::new(text);
    forward.set_caret(0);
    forward.delete_word_forward();
    assert_eq!(forward.text, "crème");
    assert!(forward.undo());
    assert_eq!(forward.text, text);

    let mut backward = TextEditState::new(text);
    backward.backspace_word();
    assert_eq!(backward.text, "café  ");
    backward.backspace_word();
    assert_eq!(backward.text, "");
}

#[test]
fn affinity_defaults_transitions_invalidation_and_semantic_equality_are_fixed() {
    let empty = TextEditState::new("");
    assert_eq!(
        empty.caret_position(),
        TextCaret::new(0, TextAffinity::After)
    );

    let mut state = TextEditState::new("ab");
    assert_eq!(
        state.caret_position(),
        TextCaret::new(2, TextAffinity::Before)
    );
    state.move_left();
    assert_eq!(
        state.caret_position(),
        TextCaret::new(1, TextAffinity::After)
    );
    state.move_right();
    assert_eq!(
        state.caret_position(),
        TextCaret::new(2, TextAffinity::Before)
    );

    state.set_caret_position(TextCaret::new(1, TextAffinity::Before));
    assert_eq!(
        state.caret_position(),
        TextCaret::new(1, TextAffinity::Before)
    );
    state.selection = TextSelection::new(0, 0);
    assert_eq!(
        state.caret_position(),
        TextCaret::new(0, TextAffinity::After)
    );

    let other = TextEditState::new("ab");
    let mut other = other;
    other.set_caret(0);
    assert_eq!(state, other, "stale private affinity is not semantic state");
}

#[test]
fn undo_restores_effective_affinity_and_edge_noops_preserve_it() {
    let mut state = TextEditState::new("ab");
    state.set_caret_position(TextCaret::new(1, TextAffinity::After));
    state.insert_text("X");
    assert!(state.undo());
    assert_eq!(
        state.caret_position(),
        TextCaret::new(1, TextAffinity::After)
    );
    assert!(state.redo());
    assert_eq!(
        state.caret_position(),
        TextCaret::new(2, TextAffinity::Before)
    );

    let mut at_start = TextEditState::new("ab");
    at_start.set_caret(0);
    let start = at_start.caret_position();
    at_start.backspace();
    at_start.backspace_word();
    assert_eq!(at_start.caret_position(), start);

    let mut at_end = TextEditState::new("ab");
    let end = at_end.caret_position();
    at_end.delete_forward();
    at_end.delete_word_forward();
    assert_eq!(at_end.caret_position(), end);
}

#[test]
fn raw_public_endpoints_canonicalize_before_delete_and_preserve_true_noop_redo() {
    let mut caret_delete = TextEditState::new("Ae\u{301}B");
    caret_delete.selection = TextSelection::new(2, 3);
    caret_delete.backspace();
    assert_eq!(caret_delete.text, "e\u{301}B");
    assert_eq!(caret_delete.caret(), 0);
    assert!(caret_delete.undo());
    assert_eq!(caret_delete.text, "Ae\u{301}B");

    let mut word_delete = TextEditState::new("Ae\u{301}B");
    word_delete.selection = TextSelection::new(2, 3);
    word_delete.backspace_word();
    assert_eq!(word_delete.text, "e\u{301}B");

    let mut no_op = TextEditState::new("e\u{301}");
    no_op.insert_text("x");
    assert!(no_op.undo());
    no_op.selection = TextSelection::new(0, 1);
    no_op.backspace();
    assert_eq!(no_op.text, "e\u{301}");
    assert_eq!(no_op.selection, TextSelection::new(0, 0));
    assert!(no_op.redo(), "canonical no-op keeps redo available");
    assert_eq!(no_op.text, "e\u{301}x");
}

#[test]
fn composition_ranges_and_ordered_insert_then_navigation_are_grapheme_safe() {
    let composition = TextComposition::new("Ae\u{301}B", Some(TextRange { start: 2, end: 3 }));
    assert_eq!(composition.selection, Some(TextRange::new(1, 1)));

    let target = WidgetId::from_key("unicode-field");
    let mut state = TextEditState::new("");
    let events = [
        UiInputEvent::Text(TextInputEvent::Commit("e\u{301}".to_owned())),
        key(Key::ArrowLeft, Modifiers::default()),
    ];
    let requests = state.apply_ordered_input(&events, target, TextEditMode::SingleLine);
    assert!(requests.is_empty());
    assert_eq!(state.text, "e\u{301}");
    assert_eq!(state.caret(), 0);
}

#[test]
fn read_only_uses_unicode_navigation_and_copy_without_mutation_or_ime() {
    let mut state = TextEditState::new("Ae\u{301}B");
    state.set_caret(4);
    let shift = Modifiers::new(true, false, false, false);
    let command = Modifiers::new(false, true, false, false);
    let events = [
        key(Key::ArrowLeft, shift),
        key(Key::Character("c".to_owned()), command),
        key(Key::Backspace, Modifiers::default()),
        UiInputEvent::Text(TextInputEvent::CompositionStart),
        UiInputEvent::Text(TextInputEvent::Commit("ignored".to_owned())),
    ];

    let requests = state.apply_read_only_ordered_input(&events, TextEditMode::SingleLine);

    assert_eq!(state.text, "Ae\u{301}B");
    assert_eq!(state.selection, TextSelection::new(4, 1));
    assert_eq!(
        requests,
        vec![PlatformRequest::CopyToClipboard("e\u{301}".to_owned())]
    );
    assert_eq!(state.composition, None);
}
