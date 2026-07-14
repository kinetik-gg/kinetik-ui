use stern_core::{
    ClipboardText, InputWheelDelta, Key, KeyEvent, KeyState, Modifiers, MouseButton, PhysicalKey,
    PlatformRequest, Point, Primitive, SemanticActionKind, TextInputEvent, TextInputOwnerMode,
    UiInput, UiInputEvent, UiMemory, Vec2,
};
use stern_text::{TextComposition, TextEditState, TextSelection};
use stern_widgets::{TextFieldAccess, Ui};

use super::{ComponentState, Rect, default_dark_theme, has_semantic_action, root_child};

const RECT: Rect = Rect::new(0.0, 0.0, 160.0, 24.0);

fn focused_memory() -> UiMemory {
    let mut memory = UiMemory::new();
    memory.focus(root_child("field"));
    memory
}

fn canonical(events: impl IntoIterator<Item = UiInputEvent>) -> UiInput {
    let mut input = UiInput::default();
    for event in events {
        input.push_event(event);
    }
    input
}

fn ctrl_key(character: &str, physical: PhysicalKey) -> UiInputEvent {
    let modifiers = Modifiers::new(false, true, false, false);
    UiInputEvent::Key(KeyEvent::with_physical_key(
        Key::Character(character.to_owned()),
        physical,
        KeyState::Pressed,
        modifiers,
        false,
    ))
}

#[test]
fn access_modes_expose_distinct_semantics_focus_and_native_ime() {
    let theme = default_dark_theme();

    for access in [
        TextFieldAccess::Editable,
        TextFieldAccess::ReadOnly,
        TextFieldAccess::Disabled,
    ] {
        let mut memory = focused_memory();
        if access == TextFieldAccess::Disabled {
            memory.set_text_input_owner(root_child("field"));
        }
        let mut state = TextEditState::new("alpha");
        state.set_selection(TextSelection::new(0, 3));
        let input = UiInput::default();
        let mut ui = Ui::new(&input, &mut memory, &theme);
        let field = ui.text_field_with_access("field", RECT, &mut state, access);
        let output = ui.finish_output();
        let node = &field.widget.semantics[0];

        assert_eq!(node.focusable, access != TextFieldAccess::Disabled);
        assert_eq!(node.state.disabled, access == TextFieldAccess::Disabled);
        assert_eq!(
            has_semantic_action(node, &SemanticActionKind::SetText),
            access == TextFieldAccess::Editable
        );
        assert_eq!(
            memory.focused(),
            (access != TextFieldAccess::Disabled).then(|| root_child("field"))
        );
        assert_eq!(
            memory.text_input_owner_mode(),
            match access {
                TextFieldAccess::Editable => Some(TextInputOwnerMode::Editable),
                TextFieldAccess::ReadOnly => Some(TextInputOwnerMode::ReadOnly),
                TextFieldAccess::Disabled => None,
            }
        );
        assert_eq!(
            output.platform_requests.iter().any(|request| matches!(
                request,
                PlatformRequest::StartTextInput { .. }
                    | PlatformRequest::UpdateTextInputRect { .. }
            )),
            access == TextFieldAccess::Editable
        );
    }
}

#[test]
fn read_only_selects_navigates_and_copies_without_mutating_text_or_starting_ime() {
    let theme = default_dark_theme();
    let mut memory = focused_memory();
    let mut state = TextEditState::new("alpha");
    state.set_caret(2);
    state.composition = Some(TextComposition::new("pre", None));
    let input = canonical([
        ctrl_key("a", PhysicalKey::KeyA),
        ctrl_key("ignored", PhysicalKey::KeyC),
        UiInputEvent::Text(TextInputEvent::Commit("MUTATE".to_owned())),
        UiInputEvent::Key(KeyEvent::new(
            Key::Backspace,
            KeyState::Pressed,
            Modifiers::default(),
            false,
        )),
        ctrl_key("v", PhysicalKey::KeyV),
    ]);

    let mut ui = Ui::new(&input, &mut memory, &theme);
    let field = ui.text_field_with_access("field", RECT, &mut state, TextFieldAccess::ReadOnly);
    let output = ui.finish_output();

    assert!(!field.changed);
    assert_eq!(state.text, "alpha");
    assert_eq!(state.selection, TextSelection::new(0, 5));
    assert_eq!(state.composition, None);
    assert!(
        output
            .platform_requests
            .contains(&PlatformRequest::CopyToClipboard("alpha".to_owned()))
    );
    assert!(!output.platform_requests.iter().any(|request| matches!(
        request,
        PlatformRequest::RequestClipboardText { .. }
            | PlatformRequest::StartTextInput { .. }
            | PlatformRequest::UpdateTextInputRect { .. }
    )));
}

#[test]
fn read_only_preserves_populated_undo_and_redo_history() {
    let theme = default_dark_theme();
    let mut memory = focused_memory();
    let mut state = TextEditState::new("a");
    state.insert_text("b");
    state.insert_text("c");
    assert!(state.undo());
    state.composition = Some(TextComposition::new("preedit", None));
    let mut expected = state.clone();
    expected.composition = None;
    let input = canonical([
        UiInputEvent::Text(TextInputEvent::CompositionStart),
        UiInputEvent::Text(TextInputEvent::Composition {
            text: "incoming".to_owned(),
            selection: None,
        }),
        UiInputEvent::Text(TextInputEvent::Commit("ignored".to_owned())),
        UiInputEvent::Key(KeyEvent::new(
            Key::Backspace,
            KeyState::Pressed,
            Modifiers::default(),
            false,
        )),
        ctrl_key("z", PhysicalKey::KeyZ),
        ctrl_key("y", PhysicalKey::KeyY),
    ]);

    let mut ui = Ui::new(&input, &mut memory, &theme);
    ui.text_field_with_access("field", RECT, &mut state, TextFieldAccess::ReadOnly);
    let _ = ui.finish_output();

    assert_eq!(state, expected);
    for operation in [
        TextEditState::undo,
        TextEditState::redo,
        TextEditState::redo,
    ] {
        assert_eq!(operation(&mut state), operation(&mut expected));
        assert_eq!(state, expected);
    }
}

#[test]
fn unfocused_read_only_clears_composition_before_empty_and_pointer_entry_geometry() {
    let theme = default_dark_theme();

    let mut idle_memory = UiMemory::new();
    let mut idle_state = TextEditState::new("ab");
    idle_state.set_caret(1);
    idle_state.composition = Some(TextComposition::new("XXXXXXXX", None));
    let mut expected_idle = idle_state.clone();
    expected_idle.composition = None;
    let input = UiInput::default();
    let mut ui = Ui::new(&input, &mut idle_memory, &theme);
    let output =
        ui.text_field_with_access("field", RECT, &mut idle_state, TextFieldAccess::ReadOnly);
    let _ = ui.finish_output();
    assert_eq!(idle_state, expected_idle);
    assert!(!output.changed);

    let mut press_memory = UiMemory::new();
    let mut press_state = TextEditState::new("ab");
    press_state.set_caret(1);
    press_state.composition = Some(TextComposition::new("XXXXXXXX", None));
    let mut input = UiInput::default();
    input.push_event(UiInputEvent::PointerButton {
        button: MouseButton::Primary,
        down: true,
        click_count: 1,
        position: Some(Point::new(30.0, 12.0)),
    });
    let mut ui = Ui::new(&input, &mut press_memory, &theme);
    ui.text_field_with_access("field", RECT, &mut press_state, TextFieldAccess::ReadOnly);
    let _ = ui.finish_output();
    assert_eq!(press_state.composition, None);
    assert_eq!(press_state.caret(), 2);
}

#[test]
fn disabled_preserves_caller_state_clears_stale_owner_and_paints_no_selection_or_caret() {
    let theme = default_dark_theme();
    let field = root_child("field");
    let mut memory = focused_memory();
    memory.set_text_input_owner(field);
    let mut state = TextEditState::new("alpha");
    state.insert_text("1");
    state.insert_text("2");
    assert!(state.undo());
    state.set_selection(TextSelection::new(0, state.text.len()));
    state.composition = Some(TextComposition::new("pre", None));
    let before = state.clone();

    let mut input = canonical([
        UiInputEvent::PointerButton {
            button: MouseButton::Primary,
            down: true,
            click_count: 1,
            position: Some(Point::new(8.0, 8.0)),
        },
        ctrl_key("c", PhysicalKey::KeyC),
        UiInputEvent::Text(TextInputEvent::Commit("ignored".to_owned())),
        UiInputEvent::Wheel {
            delta: InputWheelDelta::Pixels(Vec2::new(-40.0, 0.0)),
            position: Some(Point::new(8.0, 8.0)),
        },
    ]);
    input
        .clipboard_text
        .push(ClipboardText::new(field, "ignored"));
    let mut ui = Ui::new(&input, &mut memory, &theme);
    ui.text_field_with_access("field", RECT, &mut state, TextFieldAccess::Disabled);
    let output = ui.finish_output();

    assert_eq!(state, before);
    assert_eq!(memory.focused(), None);
    assert_eq!(memory.text_input_owner(), None);
    assert_eq!(memory.pointer_capture(), None);
    assert_eq!(memory.scroll_offset(field), Vec2::ZERO);
    assert!(
        output
            .platform_requests
            .contains(&PlatformRequest::StopTextInput)
    );
    assert!(!output.platform_requests.iter().any(|request| matches!(
        request,
        PlatformRequest::CopyToClipboard(_) | PlatformRequest::RequestClipboardText { .. }
    )));

    let recipe = theme.text_field(ComponentState {
        hovered: false,
        pressed: false,
        focused: true,
        disabled: false,
        selected: false,
    });
    assert!(!output.primitives.iter().any(|primitive| {
        matches!(
            primitive,
            Primitive::Rect(rect)
                if rect.fill.as_ref() == Some(&recipe.selection)
                    || (rect.fill == Some(stern_core::Brush::Solid(recipe.caret))
                        && rect.rect.width <= 1.0)
        )
    }));
}

#[test]
fn focus_loss_fences_later_gain_and_input_for_editable_and_read_only() {
    let theme = default_dark_theme();
    for access in [TextFieldAccess::Editable, TextFieldAccess::ReadOnly] {
        let mut memory = focused_memory();
        let mut state = TextEditState::new("ab");
        state.set_caret(2);
        let input = canonical([
            UiInputEvent::Key(KeyEvent::new(
                Key::ArrowLeft,
                KeyState::Pressed,
                Modifiers::default(),
                false,
            )),
            UiInputEvent::WindowFocusChanged(false),
            UiInputEvent::WindowFocusChanged(true),
            UiInputEvent::Text(TextInputEvent::Commit("X".to_owned())),
            UiInputEvent::Key(KeyEvent::new(
                Key::ArrowRight,
                KeyState::Pressed,
                Modifiers::default(),
                false,
            )),
        ]);

        let mut ui = Ui::new(&input, &mut memory, &theme);
        ui.text_field_with_access("field", RECT, &mut state, access);
        let _ = ui.finish_output();

        assert_eq!(state.text, "ab");
        assert_eq!(state.selection, TextSelection::new(1, 1));
        assert_eq!(memory.focused(), None);
        assert_eq!(memory.text_input_owner(), None);
    }
}

#[test]
fn boolean_methods_map_exactly_to_editable_and_disabled() {
    let theme = default_dark_theme();
    for (disabled, access) in [
        (false, TextFieldAccess::Editable),
        (true, TextFieldAccess::Disabled),
    ] {
        let mut bool_memory = UiMemory::new();
        let mut access_memory = UiMemory::new();
        let mut bool_state = TextEditState::new("same");
        let mut access_state = bool_state.clone();
        let bool_output = {
            let input = UiInput::default();
            let mut ui = Ui::new(&input, &mut bool_memory, &theme);
            let field = ui.text_field("field", RECT, &mut bool_state, disabled);
            let frame = ui.finish_output();
            (field, frame)
        };
        let access_output = {
            let input = UiInput::default();
            let mut ui = Ui::new(&input, &mut access_memory, &theme);
            let field = ui.text_field_with_access("field", RECT, &mut access_state, access);
            let frame = ui.finish_output();
            (field, frame)
        };

        assert_eq!(bool_state, access_state);
        assert_eq!(bool_output.0, access_output.0);
        assert_eq!(bool_output.1.primitives, access_output.1.primitives);
        assert_eq!(bool_output.1.semantics, access_output.1.semantics);
        assert_eq!(
            bool_output.1.platform_requests,
            access_output.1.platform_requests
        );

        let mut bool_memory = UiMemory::new();
        let mut access_memory = UiMemory::new();
        let mut bool_state = TextEditState::new("same\nlines");
        let mut access_state = bool_state.clone();
        let bool_output = {
            let input = UiInput::default();
            let mut ui = Ui::new(&input, &mut bool_memory, &theme);
            let field = ui.multi_line_text_field("field", RECT, &mut bool_state, disabled);
            let frame = ui.finish_output();
            (field, frame)
        };
        let access_output = {
            let input = UiInput::default();
            let mut ui = Ui::new(&input, &mut access_memory, &theme);
            let field =
                ui.multi_line_text_field_with_access("field", RECT, &mut access_state, access);
            let frame = ui.finish_output();
            (field, frame)
        };

        assert_eq!(bool_state, access_state);
        assert_eq!(bool_output.0, access_output.0);
        assert_eq!(bool_output.1.primitives, access_output.1.primitives);
        assert_eq!(bool_output.1.semantics, access_output.1.semantics);
        assert_eq!(
            bool_output.1.platform_requests,
            access_output.1.platform_requests
        );
    }
}
