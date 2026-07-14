use stern_core::{
    Brush, ComponentState, Modifiers, MouseButton, Point, Primitive, TextInputEvent, UiInput,
    UiInputEvent, UiMemory, Vec2,
};
use stern_text::{TextEditState, TextSelection};
use stern_widgets::Ui;

use super::{PlatformRequest, Rect, default_dark_theme, root_child};

const FIRST_RECT: Rect = Rect::new(0.0, 0.0, 120.0, 24.0);
const SECOND_RECT: Rect = Rect::new(0.0, 32.0, 120.0, 24.0);

fn press(position: Option<Point>, click_count: u8) -> UiInputEvent {
    UiInputEvent::PointerButton {
        button: MouseButton::Primary,
        down: true,
        click_count,
        position,
    }
}

fn release(position: Option<Point>, click_count: u8) -> UiInputEvent {
    UiInputEvent::PointerButton {
        button: MouseButton::Primary,
        down: false,
        click_count,
        position,
    }
}

fn input(events: impl IntoIterator<Item = UiInputEvent>) -> UiInput {
    let mut input = UiInput::default();
    for event in events {
        input.push_event(event);
    }
    input
}

fn run_two_fields(
    input: &UiInput,
    second_first: bool,
) -> (TextEditState, TextEditState, UiMemory, Vec<PlatformRequest>) {
    let theme = default_dark_theme();
    let first = root_child("first");
    let mut memory = UiMemory::new();
    memory.focus(first);
    memory.set_text_input_owner(first);
    let mut first_state = TextEditState::new("one");
    let mut second_state = TextEditState::new("two");

    let mut ui = Ui::new(input, &mut memory, &theme);
    if second_first {
        ui.text_field("second", SECOND_RECT, &mut second_state, false);
        ui.text_field("first", FIRST_RECT, &mut first_state, false);
    } else {
        ui.text_field("first", FIRST_RECT, &mut first_state, false);
        ui.text_field("second", SECOND_RECT, &mut second_state, false);
    }
    let output = ui.finish_output();
    (first_state, second_state, memory, output.platform_requests)
}

#[test]
fn press_then_text_and_text_then_press_are_call_order_independent() {
    for second_first in [false, true] {
        let press_then_text = input([
            press(Some(Point::new(4.0, 40.0)), 1),
            UiInputEvent::Text(TextInputEvent::Commit("X".to_owned())),
        ]);
        let (first, second, memory, requests) = run_two_fields(&press_then_text, second_first);
        assert_eq!(first.text, "one");
        assert_eq!(second.text, "Xtwo");
        assert_eq!(memory.focused(), Some(root_child("second")));
        assert_eq!(memory.text_input_owner(), Some(root_child("second")));
        assert_eq!(
            requests
                .iter()
                .filter(|request| matches!(request, PlatformRequest::StartTextInput { .. }))
                .count(),
            1
        );

        let text_then_press = input([
            UiInputEvent::Text(TextInputEvent::Commit("X".to_owned())),
            press(Some(Point::new(4.0, 40.0)), 1),
        ]);
        let (first, second, memory, _) = run_two_fields(&text_then_press, second_first);
        assert_eq!(first.text, "one");
        assert_eq!(second.text, "two");
        assert_eq!(memory.focused(), Some(root_child("second")));
        assert_eq!(memory.text_input_owner(), Some(root_child("second")));
    }
}

#[test]
fn press_time_shift_uses_entry_anchor_not_final_modifiers() {
    let theme = default_dark_theme();
    let mut memory = UiMemory::new();
    let mut state = TextEditState::new("alpha");
    state.set_caret(0);
    let input = input([
        UiInputEvent::ModifiersChanged(Modifiers::new(true, false, false, false)),
        press(Some(Point::new(34.0, 8.0)), 1),
        UiInputEvent::ModifiersChanged(Modifiers::default()),
    ]);

    let mut ui = Ui::new(&input, &mut memory, &theme);
    ui.text_field("field", FIRST_RECT, &mut state, false);
    let _ = ui.finish_output();

    assert_eq!(state.selection.anchor, 0);
    assert!(state.selection.active > 0);
    assert_eq!(input.keyboard.modifiers, Modifiers::default());
}

#[test]
fn double_click_selects_scalar_run_and_never_creates_domain_drag() {
    let theme = default_dark_theme();
    let mut memory = UiMemory::new();
    let mut state = TextEditState::new("alpha beta");
    let input = input([press(Some(Point::new(52.0, 8.0)), 2)]);

    let mut ui = Ui::new(&input, &mut memory, &theme);
    ui.text_field("field", FIRST_RECT, &mut state, false);
    let _ = ui.finish_output();

    assert_eq!(state.selected_text(), Some("beta"));
    assert_eq!(memory.drag_source(), None);
}

#[test]
fn same_frame_drag_clamps_outside_and_cancel_preserves_last_selection() {
    let theme = default_dark_theme();
    let mut memory = UiMemory::new();
    let mut state = TextEditState::new("é中xyz");
    let drag = input([
        press(Some(Point::new(4.0, 8.0)), 1),
        UiInputEvent::PointerMoved {
            position: Point::new(400.0, 8.0),
            delta: Vec2::new(396.0, 0.0),
        },
        UiInputEvent::PointerReleaseAll {
            position: Some(Point::new(400.0, 8.0)),
        },
        UiInputEvent::PointerMoved {
            position: Point::new(4.0, 8.0),
            delta: Vec2::new(-396.0, 0.0),
        },
    ]);

    let mut ui = Ui::new(&drag, &mut memory, &theme);
    ui.text_field("field", FIRST_RECT, &mut state, false);
    let _ = ui.finish_output();

    assert_eq!(state.selection, TextSelection::new(0, state.text.len()));
    assert!(state.text.is_char_boundary(state.selection.anchor));
    assert!(state.text.is_char_boundary(state.selection.active));
    assert_eq!(memory.drag_source(), None);
    assert_eq!(memory.pointer_capture(), None);
}

#[test]
fn ordered_drag_uses_entry_layout_for_hits_before_and_after_text_mutation() {
    let theme = default_dark_theme();
    let field = root_child("field");
    let mut memory = UiMemory::new();
    memory.focus(field);
    let mut state = TextEditState::new("ab");
    let input = input([
        press(Some(Point::new(12.0, 8.0)), 1),
        UiInputEvent::Text(TextInputEvent::Commit("LONG".to_owned())),
        UiInputEvent::PointerMoved {
            position: Point::new(100.0, 8.0),
            delta: Vec2::new(88.0, 0.0),
        },
    ]);

    let mut ui = Ui::new(&input, &mut memory, &theme);
    ui.text_field("field", FIRST_RECT, &mut state, false);
    let output = ui.finish_output();

    assert_eq!(state.text, "aLONGb");
    assert_eq!(state.selection, TextSelection::new(1, 2));
    let recipe = theme.text_field(ComponentState {
        hovered: false,
        pressed: true,
        focused: true,
        disabled: false,
        selected: false,
    });
    assert!(
        output.primitives.iter().any(|primitive| {
            matches!(primitive, Primitive::Text(text) if text.text == "aLONGb")
        })
    );
    assert!(output.primitives.iter().any(|primitive| {
        matches!(primitive, Primitive::Rect(rect) if rect.fill == Some(recipe.selection))
    }));
    let painted_caret = output
        .primitives
        .iter()
        .find_map(|primitive| match primitive {
            Primitive::Rect(rect) if rect.fill == Some(Brush::Solid(recipe.caret)) => {
                Some(rect.rect)
            }
            _ => None,
        });
    let native_caret = output
        .platform_requests
        .iter()
        .find_map(|request| match request {
            PlatformRequest::StartTextInput { rect: Some(rect) } => Some(*rect),
            _ => None,
        });
    let content_rect = Rect::new(
        FIRST_RECT.x + recipe.padding_x,
        FIRST_RECT.y + recipe.padding_y,
        FIRST_RECT.width - recipe.padding_x * 2.0,
        FIRST_RECT.height - recipe.padding_y * 2.0,
    );
    assert_eq!(
        native_caret,
        painted_caret.and_then(|caret| content_rect.intersection(caret))
    );
}

#[test]
fn selection_drag_retains_capture_and_anchor_across_frames() {
    let theme = default_dark_theme();
    let field = root_child("field");
    let mut memory = UiMemory::new();
    let mut state = TextEditState::new("alpha beta");

    let press_frame = input([
        press(Some(Point::new(12.0, 8.0)), 1),
        UiInputEvent::Text(TextInputEvent::Commit("X".to_owned())),
    ]);
    let mut ui = Ui::new(&press_frame, &mut memory, &theme);
    ui.text_field("field", FIRST_RECT, &mut state, false);
    let _ = ui.finish_output();
    assert_eq!(state.text, "aXlpha beta");
    assert_eq!(state.caret(), 2);
    assert_eq!(memory.pointer_capture(), Some(field));
    assert_eq!(memory.drag_source(), None);

    let move_frame = input([UiInputEvent::PointerMoved {
        position: Point::new(100.0, 8.0),
        delta: Vec2::new(88.0, 0.0),
    }]);
    let mut ui = Ui::new(&move_frame, &mut memory, &theme);
    ui.text_field("field", FIRST_RECT, &mut state, false);
    let _ = ui.finish_output();
    let moved_selection = state.selection;
    assert_eq!(moved_selection.anchor, 1);
    assert!(moved_selection.active > moved_selection.anchor);
    assert_eq!(memory.pointer_capture(), Some(field));
    assert_eq!(memory.drag_source(), None);

    let release_frame = input([release(Some(Point::new(100.0, 8.0)), 1)]);
    let mut ui = Ui::new(&release_frame, &mut memory, &theme);
    ui.text_field("field", FIRST_RECT, &mut state, false);
    let _ = ui.finish_output();
    assert_eq!(state.selection, moved_selection);
    assert_eq!(memory.pointer_capture(), None);
    assert_eq!(memory.drag_source(), None);
}

#[test]
fn final_positionless_press_preempts_old_owner_and_fails_closed() {
    let theme = default_dark_theme();
    let field = root_child("field");
    let mut memory = UiMemory::new();
    memory.focus(field);
    memory.set_text_input_owner(field);
    let mut state = TextEditState::new("abc");
    let input = input([
        UiInputEvent::Text(TextInputEvent::Commit("X".to_owned())),
        press(None, 1),
    ]);

    let mut ui = Ui::new(&input, &mut memory, &theme);
    ui.text_field("field", FIRST_RECT, &mut state, false);
    let output = ui.finish_output();

    assert_eq!(state.text, "abc");
    assert_eq!(memory.focused(), None);
    assert_eq!(memory.text_input_owner(), None);
    assert!(
        output
            .platform_requests
            .contains(&PlatformRequest::StopTextInput)
    );
}

#[test]
fn completed_second_press_wins_but_unreleased_second_press_fails_closed() {
    for second_first in [false, true] {
        let completed = input([
            press(Some(Point::new(4.0, 8.0)), 1),
            release(Some(Point::new(4.0, 8.0)), 1),
            press(Some(Point::new(4.0, 40.0)), 1),
            UiInputEvent::Text(TextInputEvent::Commit("X".to_owned())),
        ]);
        let (first, second, memory, _) = run_two_fields(&completed, second_first);
        assert_eq!(first.text, "one");
        assert_eq!(second.text, "Xtwo");
        assert_eq!(memory.text_input_owner(), Some(root_child("second")));

        let ambiguous = input([
            press(Some(Point::new(4.0, 8.0)), 1),
            press(Some(Point::new(4.0, 40.0)), 1),
            UiInputEvent::Text(TextInputEvent::Commit("X".to_owned())),
        ]);
        let (first, second, memory, _) = run_two_fields(&ambiguous, second_first);
        assert_eq!(first.text, "one");
        assert_eq!(second.text, "two");
        assert_eq!(memory.text_input_owner(), None);
    }
}

#[test]
fn legacy_snapshot_press_preempts_old_owner_in_both_call_orders() {
    for second_first in [false, true] {
        let mut snapshot = super::pressed_at(4.0, 40.0);
        snapshot.text_events = vec![TextInputEvent::Commit("X".to_owned())];
        let (first, second, memory, _) = run_two_fields(&snapshot, second_first);
        assert_eq!(first.text, "one");
        assert_eq!(second.text, "Xtwo");
        assert_eq!(memory.text_input_owner(), Some(root_child("second")));
    }
}
