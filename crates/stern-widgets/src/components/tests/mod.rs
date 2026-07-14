use super::{
    PanelFrame, button, button_semantics, checkbox, checkbox_semantics, checkbox_with_label,
    icon_button, icon_button_with_label, icon_button_with_library, image, image_icon_button,
    image_icon_button_sized, image_icon_selectable_button, label, list_row, multi_line_text_field,
    multi_line_text_field_with_text_layouts, numeric_input, panel, panel_semantics,
    radio_button_with_label, search_field, search_field_semantics, slider, slider_semantics,
    slider_with_label, tab_button, text_field, text_field_semantics, text_field_with_text_layouts,
    toggle, toggle_with_label,
};
use crate::{IconGraphic, IconId, IconLibrary, IconPath, Ui};
use stern_core::{
    ClipboardText, ImageId, Insets, Key, KeyEvent, KeyState, KeyboardInput, Modifiers, PathElement,
    PlatformRequest, Point, PointerButtonState, PointerInput, Primitive, Rect, RectPrimitive,
    SemanticActionKind, SemanticRole, SemanticValue, UiInput, UiMemory, WidgetId,
    default_dark_theme,
};
use stern_text::{TextEditState, TextLayoutStore, TextSelection};

fn input_at(x: f32, y: f32) -> UiInput {
    UiInput {
        pointer: PointerInput {
            position: Some(Point::new(x, y)),
            ..PointerInput::default()
        },
        ..UiInput::default()
    }
}

fn shortcut_input(character: &str) -> UiInput {
    let modifiers = Modifiers::new(false, true, false, false);
    UiInput {
        keyboard: KeyboardInput {
            modifiers,
            events: vec![KeyEvent::new(
                Key::Character(character.to_owned()),
                KeyState::Pressed,
                modifiers,
                false,
            )],
        },
        ..UiInput::default()
    }
}

fn assert_approx(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < f32::EPSILON,
        "expected {actual} to equal {expected}"
    );
}

fn check_icon() -> IconGraphic {
    IconGraphic::new(
        Rect::new(0.0, 0.0, 24.0, 24.0),
        [IconPath::stroked(
            vec![
                PathElement::MoveTo(Point::new(5.0, 12.0)),
                PathElement::LineTo(Point::new(10.0, 17.0)),
                PathElement::LineTo(Point::new(19.0, 7.0)),
            ],
            2.0,
        )],
    )
}

mod basic;
mod text_fields;
