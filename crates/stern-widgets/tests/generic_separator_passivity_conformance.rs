//! Public generic-separator passivity conformance.

use stern_core::{
    FrameOutput, Rect, Response, SemanticRole, UiInput, UiMemory, WidgetId, default_dark_theme,
};
use stern_widgets::{Ui, separator};

const LEFT: Rect = Rect::new(0.0, 0.0, 30.0, 20.0);
const SEPARATOR: Rect = Rect::new(40.0, 0.0, 30.0, 20.0);
const RIGHT: Rect = Rect::new(80.0, 0.0, 30.0, 20.0);

fn sentry_ids() -> [WidgetId; 2] {
    let root = WidgetId::from_key("root");
    [root.child("left-sentry"), root.child("right-sentry")]
}

fn sentry_frame(input: UiInput, memory: &mut UiMemory) -> (Response, Response, FrameOutput) {
    let theme = default_dark_theme();
    let mut ui = Ui::new(&input, memory, &theme);
    let left = ui.button("left-sentry", LEFT, "Left", false);
    ui.separator(SEPARATOR);
    let right = ui.button("right-sentry", RIGHT, "Right", false);
    (left, right, ui.finish_output())
}

fn separator_only(rect: Rect) -> (FrameOutput, UiMemory) {
    let theme = default_dark_theme();
    let input = UiInput::default();
    let mut memory = UiMemory::new();
    let mut ui = Ui::new(&input, &mut memory, &theme);
    ui.separator(rect);
    (ui.finish_output(), memory)
}

fn assert_separator_only(rect: Rect) {
    let theme = default_dark_theme();
    let (frame, memory) = separator_only(rect);

    assert_eq!(frame.primitives, [separator(rect, &theme)]);
    assert!(frame.semantics.nodes().is_empty());
    assert!(frame.semantics.focus_order().is_empty());
    assert!(frame.actions.is_empty());
    assert!(frame.platform_requests.is_empty());
    assert!(frame.warnings.is_empty());
    assert_eq!(memory.focused(), None);
    assert_eq!(memory.hovered(), None);
    assert_eq!(memory.active(), None);
    assert_eq!(memory.pressed(), None);
    assert_eq!(memory.pointer_capture(), None);
}

#[test]
fn ui_separator_emits_only_passive_presentation() {
    assert_separator_only(SEPARATOR);
}

#[test]
fn ui_separator_does_not_enter_control_focus_order() {
    let theme = default_dark_theme();
    let ids = sentry_ids();
    let (left, right, frame) = sentry_frame(UiInput::default(), &mut UiMemory::new());

    assert_eq!([left.id, right.id], ids);
    assert!(frame.primitives.contains(&separator(SEPARATOR, &theme)));
    assert_eq!(
        frame
            .semantics
            .nodes()
            .iter()
            .map(|node| (node.id, node.role.clone()))
            .collect::<Vec<_>>(),
        ids.map(|id| (id, SemanticRole::Button))
    );
    assert_eq!(frame.semantics.focus_order(), ids);
    assert!(frame.actions.is_empty());
    assert!(frame.warnings.is_empty());
}

#[test]
fn ui_separator_passivity_holds_for_fractional_and_zero_bounds() {
    for rect in [
        Rect::new(10.25, 20.5, 30.75, 0.5),
        Rect::new(10.0, 20.0, 0.0, 12.0),
        Rect::new(10.0, 20.0, 12.0, 0.0),
    ] {
        assert_separator_only(rect);
    }
}
