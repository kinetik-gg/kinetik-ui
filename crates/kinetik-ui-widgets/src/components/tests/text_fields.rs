use super::*;

#[test]
fn text_field_applies_input_while_focused() {
    let theme = default_dark_theme();
    let id = WidgetId::from_key("text");
    let mut memory = UiMemory::new();
    memory.focus(id);
    let mut state = TextEditState::new("");
    let input = UiInput {
        text_events: vec![kinetik_ui_core::TextInputEvent::Commit("a".to_owned())],
        ..UiInput::default()
    };

    let output = text_field(
        id,
        Rect::new(0.0, 0.0, 80.0, 24.0),
        &mut state,
        &input,
        &mut memory,
        &theme,
        false,
    );

    assert!(output.changed);
    assert_eq!(state.text, "a");
}

#[test]
fn text_field_ignores_text_input_while_unfocused() {
    let theme = default_dark_theme();
    let id = WidgetId::from_key("text");
    let mut memory = UiMemory::new();
    let mut state = TextEditState::new("base");
    let input = UiInput {
        text_events: vec![kinetik_ui_core::TextInputEvent::Commit(
            "ignored".to_owned(),
        )],
        ..UiInput::default()
    };

    let output = text_field(
        id,
        Rect::new(0.0, 0.0, 80.0, 24.0),
        &mut state,
        &input,
        &mut memory,
        &theme,
        false,
    );

    assert!(!output.changed);
    assert_eq!(state.text, "base");
    assert!(output.widget.platform_requests.is_empty());
}

#[test]
fn text_field_applies_editing_shortcuts_only_while_focused() {
    let theme = default_dark_theme();
    let id = WidgetId::from_key("text");
    let mut memory = UiMemory::new();
    let mut state = TextEditState::new("abcd");
    let input = shortcut_input("a");

    let unfocused = text_field(
        id,
        Rect::new(0.0, 0.0, 80.0, 24.0),
        &mut state,
        &input,
        &mut memory,
        &theme,
        false,
    );

    assert!(!unfocused.changed);
    assert_eq!(state.selection, TextSelection::new(4, 4));

    memory.focus(id);
    let focused = text_field(
        id,
        Rect::new(0.0, 0.0, 80.0, 24.0),
        &mut state,
        &input,
        &mut memory,
        &theme,
        false,
    );

    assert!(!focused.changed);
    assert_eq!(state.selection, TextSelection::new(0, 4));
}

#[test]
fn text_field_single_line_input_drops_newlines_and_enter_key() {
    let theme = default_dark_theme();
    let id = WidgetId::from_key("text");
    let mut memory = UiMemory::new();
    memory.focus(id);
    let mut state = TextEditState::new("");
    let input = UiInput {
        keyboard: KeyboardInput {
            modifiers: Modifiers::default(),
            events: vec![KeyEvent::new(
                Key::Enter,
                KeyState::Pressed,
                Modifiers::default(),
                false,
            )],
        },
        text_events: vec![kinetik_ui_core::TextInputEvent::Commit(
            "a\nb\r\nc".to_owned(),
        )],
        ..UiInput::default()
    };

    let output = text_field(
        id,
        Rect::new(0.0, 0.0, 80.0, 24.0),
        &mut state,
        &input,
        &mut memory,
        &theme,
        false,
    );

    assert!(output.changed);
    assert_eq!(state.text, "abc");
}

#[test]
fn text_field_copies_selected_text_through_platform_request() {
    let theme = default_dark_theme();
    let id = WidgetId::from_key("text");
    let mut memory = UiMemory::new();
    memory.focus(id);
    let mut state = TextEditState::new("abcd");
    state.set_selection(TextSelection::new(1, 3));
    let input = shortcut_input("c");

    let output = text_field(
        id,
        Rect::new(0.0, 0.0, 80.0, 24.0),
        &mut state,
        &input,
        &mut memory,
        &theme,
        false,
    );

    assert!(!output.changed);
    assert_eq!(state.text, "abcd");
    assert!(output.widget.platform_requests.iter().any(|request| {
        matches!(request, PlatformRequest::CopyToClipboard(text) if text == "bc")
    }));
}

#[test]
fn text_field_cuts_selected_text_through_platform_request_and_undo() {
    let theme = default_dark_theme();
    let id = WidgetId::from_key("text");
    let mut memory = UiMemory::new();
    memory.focus(id);
    let mut state = TextEditState::new("abcd");
    state.set_selection(TextSelection::new(1, 3));
    let input = shortcut_input("x");

    let output = text_field(
        id,
        Rect::new(0.0, 0.0, 80.0, 24.0),
        &mut state,
        &input,
        &mut memory,
        &theme,
        false,
    );

    assert!(output.changed);
    assert_eq!(state.text, "ad");
    assert!(output.widget.platform_requests.iter().any(|request| {
        matches!(request, PlatformRequest::CopyToClipboard(text) if text == "bc")
    }));
    assert!(state.undo());
    assert_eq!(state.text, "abcd");
}

#[test]
fn text_field_requests_targeted_clipboard_text_on_paste() {
    let theme = default_dark_theme();
    let id = WidgetId::from_key("text");
    let mut memory = UiMemory::new();
    memory.focus(id);
    let mut state = TextEditState::new("abcd");
    state.set_caret(2);
    let input = shortcut_input("v");

    let output = text_field(
        id,
        Rect::new(0.0, 0.0, 80.0, 24.0),
        &mut state,
        &input,
        &mut memory,
        &theme,
        false,
    );

    assert!(!output.changed);
    assert_eq!(state.text, "abcd");
    assert!(output.widget.platform_requests.iter().any(|request| {
        matches!(request, PlatformRequest::RequestClipboardText { target } if *target == id)
    }));
}

#[test]
fn text_field_switch_stops_previous_owner_before_starting_new_owner() {
    let theme = default_dark_theme();
    let first = WidgetId::from_key("first");
    let second = WidgetId::from_key("second");
    let mut first_state = TextEditState::new("one");
    let mut second_state = TextEditState::new("two");
    let mut memory = UiMemory::new();
    memory.focus(first);
    memory.set_text_input_owner(first);
    let mut input = input_at(4.0, 34.0);
    input.pointer.primary = PointerButtonState::new(true, true, false);

    let first_output = text_field(
        first,
        Rect::new(0.0, 0.0, 80.0, 24.0),
        &mut first_state,
        &input,
        &mut memory,
        &theme,
        false,
    );
    let second_output = text_field(
        second,
        Rect::new(0.0, 30.0, 80.0, 24.0),
        &mut second_state,
        &input,
        &mut memory,
        &theme,
        false,
    );

    assert!(
        !first_output
            .widget
            .platform_requests
            .iter()
            .any(|request| matches!(request, PlatformRequest::StartTextInput { .. }))
    );
    let stop_index = second_output
        .widget
        .platform_requests
        .iter()
        .position(|request| matches!(request, PlatformRequest::StopTextInput))
        .expect("previous text input stopped");
    let start_index = second_output
        .widget
        .platform_requests
        .iter()
        .position(|request| {
            matches!(request, PlatformRequest::StartTextInput { rect: Some(rect) } if *rect == Rect::new(0.0, 30.0, 80.0, 24.0))
    })
    .expect("new text input started");
    assert!(stop_index < start_index);
    assert_eq!(memory.text_input_owner(), Some(second));
}

#[test]
fn text_field_applies_only_targeted_clipboard_text() {
    let theme = default_dark_theme();
    let id = WidgetId::from_key("text");
    let other = WidgetId::from_key("other");
    let mut memory = UiMemory::new();
    memory.focus(id);
    let mut state = TextEditState::new("a");
    state.set_caret(1);
    let input = UiInput {
        clipboard_text: vec![
            ClipboardText::new(other, "wrong"),
            ClipboardText::new(id, "b\nc"),
        ],
        ..UiInput::default()
    };

    let output = text_field(
        id,
        Rect::new(0.0, 0.0, 80.0, 24.0),
        &mut state,
        &input,
        &mut memory,
        &theme,
        false,
    );

    assert!(output.changed);
    assert_eq!(state.text, "abc");
    assert!(state.undo());
    assert_eq!(state.text, "a");
}

#[test]
fn text_field_ignores_clipboard_text_for_other_target() {
    let theme = default_dark_theme();
    let id = WidgetId::from_key("text");
    let other = WidgetId::from_key("other");
    let mut memory = UiMemory::new();
    memory.focus(id);
    let mut state = TextEditState::new("a");
    state.set_caret(1);
    let input = UiInput {
        clipboard_text: vec![ClipboardText::new(other, "wrong")],
        ..UiInput::default()
    };

    let output = text_field(
        id,
        Rect::new(0.0, 0.0, 80.0, 24.0),
        &mut state,
        &input,
        &mut memory,
        &theme,
        false,
    );

    assert!(!output.changed);
    assert_eq!(state.text, "a");
}

#[test]
fn clipboard_text_targets_focused_requesting_field() {
    let theme = default_dark_theme();
    let first = WidgetId::from_key("first");
    let second = WidgetId::from_key("second");
    let mut memory = UiMemory::new();
    memory.focus(second);
    let mut first_state = TextEditState::new("one");
    let mut second_state = TextEditState::new("two");
    second_state.set_caret(3);
    let input = UiInput {
        clipboard_text: vec![
            ClipboardText::new(first, " wrong"),
            ClipboardText::new(second, " pasted"),
        ],
        ..UiInput::default()
    };

    let first_output = text_field(
        first,
        Rect::new(0.0, 0.0, 80.0, 24.0),
        &mut first_state,
        &input,
        &mut memory,
        &theme,
        false,
    );
    let second_output = text_field(
        second,
        Rect::new(0.0, 30.0, 80.0, 24.0),
        &mut second_state,
        &input,
        &mut memory,
        &theme,
        false,
    );

    assert!(!first_output.changed);
    assert_eq!(first_state.text, "one");
    assert!(second_output.changed);
    assert_eq!(second_state.text, "two pasted");
}

#[test]
fn ui_text_field_losing_focus_to_non_text_stops_platform_text_input() {
    let theme = default_dark_theme();
    let field = WidgetId::from_key("root").child("field");
    let other = WidgetId::from_key("root").child("other");
    let mut memory = UiMemory::new();
    memory.focus(field);
    memory.set_text_input_owner(field);
    let mut state = TextEditState::new("abc");
    let mut input = input_at(104.0, 4.0);
    input.pointer.primary = PointerButtonState::new(true, true, false);

    let mut ui = Ui::new(&input, &mut memory, &theme);
    ui.text_field("field", Rect::new(0.0, 0.0, 80.0, 24.0), &mut state, false);
    ui.focusable("other", Rect::new(100.0, 0.0, 80.0, 24.0), false);
    let press_output = ui.finish_output();
    assert!(
        press_output
            .platform_requests
            .contains(&PlatformRequest::StopTextInput)
    );

    let mut input = input_at(104.0, 4.0);
    input.pointer.primary = PointerButtonState::new(false, false, true);
    let mut ui = Ui::new(&input, &mut memory, &theme);
    ui.text_field("field", Rect::new(0.0, 0.0, 80.0, 24.0), &mut state, false);
    ui.focusable("other", Rect::new(100.0, 0.0, 80.0, 24.0), false);
    let output = ui.finish_output();

    assert_eq!(memory.focused(), Some(other));
    assert_eq!(memory.text_input_owner(), None);
    assert!(
        !output
            .platform_requests
            .contains(&PlatformRequest::StopTextInput)
    );
}

#[test]
fn text_field_places_caret_from_pointer_press_with_shaped_layout() {
    let theme = default_dark_theme();
    let id = WidgetId::from_key("text");
    let rect = Rect::new(0.0, 0.0, 180.0, 28.0);
    let mut memory = UiMemory::new();
    let mut state = TextEditState::new("abcdef");
    let mut text_layouts = TextLayoutStore::new();
    let mut input = input_at(rect.max_x() - 4.0, 12.0);
    input.pointer.primary = PointerButtonState::new(true, true, false);

    let output = text_field_with_text_layouts(
        id,
        rect,
        &mut state,
        &input,
        &mut memory,
        &theme,
        false,
        Some(&mut text_layouts),
    );

    assert_eq!(state.caret(), state.text.len());
    assert!(
        output
            .widget
            .response
            .as_ref()
            .expect("text field response")
            .state
            .focused
    );
    assert!(!text_layouts.is_empty());
}

#[test]
fn multi_line_text_field_preserves_targeted_clipboard_newlines() {
    let theme = default_dark_theme();
    let id = WidgetId::from_key("multiline");
    let mut memory = UiMemory::new();
    memory.focus(id);
    let mut state = TextEditState::new("first");
    state.set_caret(5);
    let input = UiInput {
        clipboard_text: vec![ClipboardText::new(id, "\r\nsecond\rthird")],
        ..UiInput::default()
    };

    let output = multi_line_text_field(
        id,
        Rect::new(0.0, 0.0, 180.0, 80.0),
        &mut state,
        &input,
        &mut memory,
        &theme,
        false,
    );

    assert!(output.changed);
    assert_eq!(state.text, "first\nsecond\nthird");
    assert_eq!(output.visible_lines, 3);
}

#[test]
fn multi_line_text_field_accepts_enter_while_focused() {
    let theme = default_dark_theme();
    let id = WidgetId::from_key("multiline");
    let mut memory = UiMemory::new();
    memory.focus(id);
    let mut state = TextEditState::new("first");
    let input = UiInput {
        keyboard: kinetik_ui_core::KeyboardInput {
            modifiers: Modifiers::default(),
            events: vec![KeyEvent::new(
                Key::Enter,
                KeyState::Pressed,
                Modifiers::default(),
                false,
            )],
        },
        ..UiInput::default()
    };

    let output = multi_line_text_field(
        id,
        Rect::new(0.0, 0.0, 180.0, 80.0),
        &mut state,
        &input,
        &mut memory,
        &theme,
        false,
    );

    assert!(output.changed);
    assert!(state.text.ends_with('\n'));
    assert!(
        output
            .widget
            .primitives
            .iter()
            .any(|primitive| matches!(primitive, Primitive::ClipBegin { .. }))
    );
}

#[test]
fn multi_line_text_field_places_caret_on_clicked_line() {
    let theme = default_dark_theme();
    let id = WidgetId::from_key("multiline");
    let rect = Rect::new(0.0, 0.0, 180.0, 80.0);
    let mut memory = UiMemory::new();
    let mut state = TextEditState::new("one\ntwo");
    let mut text_layouts = TextLayoutStore::new();
    let mut input = input_at(rect.max_x() - 4.0, 42.0);
    input.pointer.primary = PointerButtonState::new(true, true, false);

    multi_line_text_field_with_text_layouts(
        id,
        rect,
        &mut state,
        &input,
        &mut memory,
        &theme,
        false,
        Some(&mut text_layouts),
    );

    assert_eq!(state.caret(), state.text.len());
}

#[test]
fn numeric_input_reports_parse_state() {
    let theme = default_dark_theme();
    let mut state = TextEditState::new("42");
    let output = numeric_input(
        WidgetId::from_key("number"),
        Rect::new(0.0, 0.0, 80.0, 24.0),
        &mut state,
        &UiInput::default(),
        &mut UiMemory::new(),
        &theme,
        false,
    );

    assert!(output.valid);
    assert_eq!(output.value, Some(42.0));
}

#[test]
fn search_field_reports_query() {
    let theme = default_dark_theme();
    let mut state = TextEditState::new("media");
    let output = search_field(
        WidgetId::from_key("search"),
        Rect::new(0.0, 0.0, 80.0, 24.0),
        &mut state,
        &UiInput::default(),
        &mut UiMemory::new(),
        &theme,
        false,
    );

    assert_eq!(output.query, "media");
    assert!(!output.empty);
}

#[test]
fn text_and_search_fields_expose_semantic_role_label_focus_and_value() {
    let theme = default_dark_theme();
    let field = WidgetId::from_key("field");
    let search = WidgetId::from_key("search");
    let mut memory = UiMemory::new();
    memory.focus(field);
    let mut field_state = TextEditState::new("Project");
    let mut search_state = TextEditState::new("media");

    let field_output = text_field(
        field,
        Rect::new(0.0, 0.0, 120.0, 24.0),
        &mut field_state,
        &UiInput::default(),
        &mut memory,
        &theme,
        false,
    );
    let search_output = search_field(
        search,
        Rect::new(0.0, 30.0, 120.0, 24.0),
        &mut search_state,
        &UiInput::default(),
        &mut memory,
        &theme,
        false,
    );

    let field_node = &field_output.widget.semantics[0];
    assert_eq!(field_node.role, SemanticRole::TextField);
    assert_eq!(field_node.label.as_deref(), Some("Text field"));
    assert!(field_node.focusable);
    assert!(field_node.state.focused);
    assert!(
        matches!(field_node.state.value, Some(SemanticValue::Text(ref text)) if text == "Project")
    );

    let search_node = &search_output.field.widget.semantics[0];
    assert_eq!(search_node.role, SemanticRole::SearchField);
    assert_eq!(search_node.label.as_deref(), Some("Search"));
    assert!(search_node.focusable);
    assert!(!search_node.state.focused);
    assert!(
        matches!(search_node.state.value, Some(SemanticValue::Text(ref text)) if text == "media")
    );
}

#[test]
fn widget_semantics_map_roles_states_values_and_actions() {
    let button = button_semantics(
        WidgetId::from_key("button"),
        Rect::new(0.0, 0.0, 80.0, 24.0),
        "Analyze",
        false,
    );
    let checkbox = checkbox_semantics(
        WidgetId::from_key("checkbox"),
        Rect::new(0.0, 28.0, 20.0, 20.0),
        "Enabled",
        true,
        false,
    );
    let slider = slider_semantics(
        WidgetId::from_key("slider"),
        Rect::new(0.0, 56.0, 100.0, 12.0),
        "Strength",
        0.62,
        0.0..=1.0,
        false,
    );
    let field = text_field_semantics(
        WidgetId::from_key("field"),
        Rect::new(0.0, 72.0, 120.0, 24.0),
        "Name",
        "Project",
        false,
    );
    let search = search_field_semantics(
        WidgetId::from_key("search"),
        Rect::new(0.0, 100.0, 120.0, 24.0),
        "Search",
        "media",
        false,
    );
    let panel = panel_semantics(
        WidgetId::from_key("panel"),
        Rect::new(0.0, 0.0, 200.0, 200.0),
        "Inspector",
    );

    assert_eq!(button.role, SemanticRole::Button);
    assert!(button.focusable);
    assert!(
        button
            .actions
            .iter()
            .any(|action| action.kind == SemanticActionKind::Invoke)
    );
    assert_eq!(checkbox.state.checked, Some(true));
    assert!(matches!(
        slider.state.value,
        Some(SemanticValue::Number { current, .. }) if (current - 0.62).abs() < f32::EPSILON
    ));
    assert!(matches!(field.state.value, Some(SemanticValue::Text(ref text)) if text == "Project"));
    assert_eq!(search.role, SemanticRole::SearchField);
    assert_eq!(panel.role, SemanticRole::Panel);
}
