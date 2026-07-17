//! Deterministic conformance for noninteractive menu rows.

use std::time::Duration;

use stern_core::{
    ActionContext, ActionDescriptor, ActionId, ActionInvocation, ActionSource, FrameContext, Key,
    KeyEvent, KeyState, KeyboardInput, Modifiers, PhysicalSize, Point, PointerButtonState,
    PointerInput, PointerOrder, Rect, Size, TimeInfo, UiInput, UiMemory, ViewportInfo, WidgetId,
    default_dark_theme,
};
use stern_widgets::overlays::{OverlayNavigationInput, TypeaheadBuffer};
use stern_widgets::{
    Menu, MenuItem, MenuOverlay, OverlayEntry, OverlayId, OverlayKind, OverlayScene,
    OverlaySceneIntent, OverlaySceneOutput, OverlaySceneSurface, Ui,
};

const OVERLAY_ID: OverlayId = OverlayId::from_raw(73);
const SURFACE_RECT: Rect = Rect::new(20.0, 20.0, 320.0, 260.0);

fn action(id: &str, label: &str) -> ActionDescriptor {
    ActionDescriptor::new(id, label)
}

fn disabled_action(id: &str, label: &str) -> ActionDescriptor {
    let mut action = action(id, label);
    action.state.enabled = false;
    action
}

fn hidden_action(id: &str, label: &str) -> ActionDescriptor {
    let mut action = action(id, label);
    action.state.visible = false;
    action
}

fn mixed_menu() -> Menu {
    let mut menu = Menu::new();
    menu.push(MenuItem::Label("Heading Match".to_owned()));
    menu.push(MenuItem::Action(action("file.open", "Open")));
    menu.push(MenuItem::Separator);
    menu.push(MenuItem::Action(disabled_action(
        "file.disabled",
        "Disabled Match",
    )));
    menu.push(MenuItem::Action(action("file.save", "Save")));
    menu.push(MenuItem::Label("Section Match".to_owned()));
    menu.push(MenuItem::Separator);
    menu.push(MenuItem::Action(hidden_action(
        "file.hidden",
        "Hidden Match",
    )));
    menu.push(MenuItem::Action(action("file.share", "Share")));
    menu
}

fn menu_scene(menu: Menu) -> OverlayScene {
    let mut scene = OverlayScene::new();
    scene.push(OverlaySceneSurface::menu(
        "File commands",
        MenuOverlay::new(
            OverlayEntry::new(OVERLAY_ID, OverlayKind::Menu, SURFACE_RECT),
            menu,
            ActionSource::Menu,
            ActionContext::Frame(WidgetId::from_key("document:passive-order")),
        ),
    ));
    scene
}

fn context(input: UiInput) -> FrameContext {
    FrameContext::new(
        ViewportInfo::new(
            Size::new(640.0, 480.0),
            PhysicalSize::new(640, 480),
            stern_core::ScaleFactor::ONE,
        ),
        input,
        TimeInfo::new(Duration::from_millis(500), Duration::from_millis(16), 1),
    )
}

fn run_frame(
    scene: &mut OverlayScene,
    memory: &mut UiMemory,
    input: UiInput,
) -> (OverlaySceneOutput, stern_core::FrameOutput) {
    let theme = default_dark_theme();
    let mut ui = Ui::begin_frame(context(input), memory, &theme);
    ui.resolve_pointer_targets(|plan| {
        scene.declare_pointer_targets(plan, PointerOrder::new(100));
    })
    .expect("valid pointer plan");
    let output = ui.overlay_scene(scene);
    (output, ui.finish_output())
}

fn pointer_input(position: Point, pressed: bool) -> UiInput {
    UiInput {
        pointer: PointerInput {
            position: Some(position),
            primary: if pressed {
                PointerButtonState::new(true, true, false)
            } else {
                PointerButtonState::new(false, false, true)
            },
            ..PointerInput::default()
        },
        ..UiInput::default()
    }
}

fn key_sequence(keys: &[Key]) -> UiInput {
    UiInput {
        keyboard: KeyboardInput {
            events: keys
                .iter()
                .cloned()
                .map(|key| KeyEvent::new(key, KeyState::Pressed, Modifiers::default(), false))
                .collect(),
            ..KeyboardInput::default()
        },
        ..UiInput::default()
    }
}

fn row_id(action_id: &str) -> WidgetId {
    WidgetId::from_raw(OVERLAY_ID.raw())
        .child("overlay-scene")
        .child(("overlay-action", action_id))
}

#[test]
fn passive_rows_are_excluded_from_navigation_and_typeahead_order() {
    let mut menu = mixed_menu();
    for (input, expected, visible_index) in [
        (OverlayNavigationInput::Next, "file.open", 1),
        (OverlayNavigationInput::Next, "file.save", 4),
        (OverlayNavigationInput::Next, "file.share", 7),
        (OverlayNavigationInput::Next, "file.open", 1),
        (OverlayNavigationInput::Previous, "file.share", 7),
        (OverlayNavigationInput::First, "file.open", 1),
        (OverlayNavigationInput::Last, "file.share", 7),
    ] {
        assert_eq!(menu.move_highlight(input), Some(ActionId::new(expected)));
        assert_eq!(menu.highlighted_visible_index(), Some(visible_index));
    }

    for prefix in ["heading", "section", "separator", "disabled", "hidden"] {
        menu.clear_highlight();
        let mut typeahead = TypeaheadBuffer::default();
        assert_eq!(menu.typeahead(&mut typeahead, prefix, 0), None);
        assert_eq!(menu.highlighted_action_id(), None);
    }
    let mut typeahead = TypeaheadBuffer::default();
    assert_eq!(
        menu.typeahead(&mut typeahead, "s", 0),
        Some(ActionId::new("file.save"))
    );
    assert_eq!(menu.highlighted_visible_index(), Some(4));
    assert_eq!(
        menu.typeahead(&mut typeahead, "s", 100),
        Some(ActionId::new("file.share"))
    );
    assert_eq!(menu.highlighted_visible_index(), Some(7));
}

#[test]
fn passive_rows_cannot_activate_or_enqueue_actions() {
    let mut scene = menu_scene(mixed_menu());
    let mut memory = UiMemory::new();
    let (_, initial) = run_frame(&mut scene, &mut memory, UiInput::default());
    let passive_rows = initial
        .semantics
        .nodes()
        .iter()
        .filter(|node| {
            node.role == stern_core::SemanticRole::Label
                || node.role == stern_core::SemanticRole::Custom("separator".to_owned())
        })
        .map(|node| (node.id, node.bounds.center()))
        .collect::<Vec<_>>();
    assert_eq!(passive_rows.len(), 4);

    let focused = row_id("file.save");
    memory.focus(focused);
    for (passive_id, center) in passive_rows {
        for pressed in [true, false] {
            let (output, frame) =
                run_frame(&mut scene, &mut memory, pointer_input(center, pressed));
            assert_eq!(memory.focused(), Some(focused));
            assert!(output.intents.is_empty());
            assert!(frame.actions.is_empty());
            assert!(output.responses.iter().all(|response| {
                !response.state.hovered
                    && !response.state.pressed
                    && !response.clicked
                    && response.id != passive_id
            }));
        }
    }

    let (output, mut frame) = run_frame(
        &mut scene,
        &mut memory,
        key_sequence(&[Key::End, Key::Enter]),
    );
    let expected = ActionInvocation::new(
        ActionId::new("file.share"),
        ActionSource::Menu,
        ActionContext::Frame(WidgetId::from_key("document:passive-order")),
    );
    assert_eq!(
        output.intents,
        vec![OverlaySceneIntent::Action(expected.clone())]
    );
    assert_eq!(frame.actions.drain().collect::<Vec<_>>(), vec![expected]);
}
