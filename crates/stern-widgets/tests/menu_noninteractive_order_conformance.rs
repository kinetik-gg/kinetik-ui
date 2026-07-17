//! Deterministic conformance for noninteractive menu rows.

use std::{cell::Cell, time::Duration};

use stern_core::{
    ActionContext, ActionDescriptor, ActionId, ActionInvocation, ActionSource, FrameContext, Key,
    KeyEvent, KeyState, KeyboardInput, Modifiers, PhysicalSize, Point, PointerButtonState,
    PointerInput, PointerOrder, PointerRoute, Primitive, Rect, SemanticActionKind, SemanticRole,
    Shortcut, ShortcutLabelLocalizer, ShortcutLabelToken, ShortcutPlatform, Size, TimeInfo,
    UiInput, UiMemory, ViewportInfo, WidgetId, default_dark_theme,
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

struct RecordingLocalizer(Cell<usize>);

impl ShortcutLabelLocalizer for RecordingLocalizer {
    fn token_label(
        &self,
        _platform: ShortcutPlatform,
        _token: ShortcutLabelToken<'_>,
    ) -> Option<String> {
        self.0.set(self.0.get() + 1);
        Some("localized".to_owned())
    }

    fn separator(&self, _platform: ShortcutPlatform) -> &str {
        self.0.set(self.0.get() + 1);
        "+"
    }
}

fn run_observed(
    scene: &mut OverlayScene,
    memory: &mut UiMemory,
    input: UiInput,
    presentation: Option<(ShortcutPlatform, &dyn ShortcutLabelLocalizer)>,
) -> (PointerRoute, OverlaySceneOutput, stern_core::FrameOutput) {
    let theme = default_dark_theme();
    let mut ui = Ui::begin_frame(context(input), memory, &theme);
    ui.resolve_pointer_targets(|plan| {
        scene.declare_pointer_targets(plan, PointerOrder::new(100));
    })
    .expect("valid pointer plan");
    let route = ui.memory().pointer_route();
    let output = match presentation {
        Some((platform, localizer)) => {
            ui.overlay_scene_with_menu_presentation(scene, platform, localizer)
        }
        None => ui.overlay_scene(scene),
    };
    (route, output, ui.finish_output())
}

fn passive_nodes(frame: &stern_core::FrameOutput) -> Vec<(WidgetId, SemanticRole, Rect)> {
    frame
        .semantics
        .nodes()
        .iter()
        .filter(|node| {
            node.role == SemanticRole::Label
                || node.role == SemanticRole::Custom("separator".to_owned())
        })
        .map(|node| (node.id, node.role.clone(), node.bounds))
        .collect()
}

fn assert_passive_isolation(
    route: PointerRoute,
    output: &OverlaySceneOutput,
    frame: &stern_core::FrameOutput,
) {
    let passive = passive_nodes(frame);
    assert!(!passive.is_empty());
    for (id, _, _) in &passive {
        let node = frame.semantics.get(*id).expect("passive semantics");
        assert!(!node.focusable && node.actions.is_empty());
        assert!(!node.state.focused && !node.state.pressed);
        assert!(output.responses.iter().all(|response| response.id != *id));
        assert_ne!(route, PointerRoute::Target(*id));
    }
    assert!(output.intents.is_empty() && frame.actions.is_empty());
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

#[test]
fn passive_rows_have_no_pointer_targets_responses_or_semantic_actions() {
    let mut scene = menu_scene(mixed_menu());
    let (_, output, frame) =
        run_observed(&mut scene, &mut UiMemory::new(), UiInput::default(), None);
    assert_passive_isolation(PointerRoute::Blocked, &output, &frame);
    let surface = frame
        .semantics
        .get(WidgetId::from_raw(73))
        .expect("surface");
    assert_eq!(surface.children.len(), 8);
    let enabled = [
        row_id("file.open"),
        row_id("file.save"),
        row_id("file.share"),
    ];
    for child in &surface.children {
        let node = frame.semantics.get(*child).expect("child semantics");
        assert_eq!(node.focusable, enabled.contains(child));
        if enabled.contains(child) {
            assert!(node.actions.iter().any(|action| {
                matches!(
                    action.kind,
                    SemanticActionKind::Invoke | SemanticActionKind::Open
                )
            }));
        } else {
            assert!(node.actions.is_empty());
        }
        let (route, probe, _) = run_observed(
            &mut scene,
            &mut UiMemory::new(),
            pointer_input(node.bounds.center(), true),
            None,
        );
        if enabled.contains(child) {
            assert_eq!(route, PointerRoute::Target(*child));
            assert!(probe.responses.iter().any(|response| response.id == *child));
        } else {
            assert_eq!(route, PointerRoute::Blocked);
            assert!(probe.responses.iter().all(|response| response.id != *child));
        }
    }
    assert_eq!(
        frame
            .semantics
            .focus_order()
            .into_iter()
            .filter(|id| surface.children.contains(id))
            .collect::<Vec<_>>(),
        enabled
    );
}

#[test]
fn legacy_and_presented_paths_preserve_passive_scene_isolation() {
    let shortcut = Shortcut::new(
        Modifiers::new(true, false, false, false),
        Key::Character("r".to_owned()),
    );
    let mut submenu = action("file.recent", "Recent");
    submenu.shortcut = Some(shortcut);
    let mut menu = Menu::new();
    menu.push(MenuItem::Label("Projects".to_owned()));
    menu.push_submenu(submenu, Menu::from_actions([action("file.child", "Child")]));
    menu.push(MenuItem::Separator);
    menu.push(MenuItem::Action(action("file.quit", "Quit")));
    let mut legacy_scene = menu_scene(menu.clone());
    let mut presented_scene = menu_scene(menu);
    let mut legacy_memory = UiMemory::new();
    let mut presented_memory = UiMemory::new();
    let localizer = RecordingLocalizer(Cell::new(0));
    let (legacy_route, legacy_output, legacy_frame) = run_observed(
        &mut legacy_scene,
        &mut legacy_memory,
        UiInput::default(),
        None,
    );
    let (presented_route, presented_output, presented_frame) = run_observed(
        &mut presented_scene,
        &mut presented_memory,
        UiInput::default(),
        Some((ShortcutPlatform::Windows, &localizer)),
    );
    assert_eq!(legacy_route, presented_route);
    assert_eq!(legacy_output, presented_output);
    assert_eq!(legacy_frame.semantics, presented_frame.semantics);
    assert_eq!(legacy_memory.focused(), presented_memory.focused());
    assert_eq!(
        passive_nodes(&legacy_frame),
        passive_nodes(&presented_frame)
    );
    assert_passive_isolation(legacy_route, &legacy_output, &legacy_frame);
    assert_passive_isolation(presented_route, &presented_output, &presented_frame);
    assert!(localizer.0.get() > 0);
    assert!(presented_frame.primitives.iter().any(
        |primitive| matches!(primitive, Primitive::Text(text) if text.text.contains("localized"))
    ));
    assert!(
        presented_frame
            .primitives
            .iter()
            .any(|primitive| matches!(primitive, Primitive::Text(text) if text.text == "›"))
    );
    for (_, _, bounds) in passive_nodes(&legacy_frame) {
        let input = pointer_input(bounds.center(), true);
        let (legacy_route, legacy_output, legacy_frame) =
            run_observed(&mut legacy_scene, &mut legacy_memory, input.clone(), None);
        let (presented_route, presented_output, presented_frame) = run_observed(
            &mut presented_scene,
            &mut presented_memory,
            input,
            Some((ShortcutPlatform::Windows, &localizer)),
        );
        assert_eq!(legacy_route, PointerRoute::Blocked);
        assert_eq!(legacy_route, presented_route);
        assert_eq!(legacy_output, presented_output);
        assert_eq!(legacy_frame.semantics, presented_frame.semantics);
    }
}

#[test]
fn repeated_evaluation_and_visibility_changes_preserve_passive_identity_and_order() {
    let items = |hidden: bool| {
        let mut earlier = action("file.earlier", "Earlier");
        earlier.state.visible = !hidden;
        [
            MenuItem::Label("Before".to_owned()),
            MenuItem::Action(earlier),
            MenuItem::Separator,
            MenuItem::Label("After".to_owned()),
            MenuItem::Action(action("file.final", "Final")),
        ]
    };
    let mut scene = menu_scene(Menu::from_actions([]));
    let OverlaySceneSurface::Menu { overlay, .. } = &mut scene.surfaces_mut()[0] else {
        panic!("menu surface");
    };
    overlay.menu.replace_items(items(false));
    let final_id = row_id("file.final");
    let mut memory = UiMemory::new();
    memory.focus(final_id);
    let (route_a, output_a, frame_a) =
        run_observed(&mut scene, &mut memory, UiInput::default(), None);
    let (route_b, output_b, frame_b) =
        run_observed(&mut scene, &mut memory, UiInput::default(), None);
    assert_eq!(frame_a.semantics, frame_b.semantics);
    assert_eq!(output_a, output_b);
    assert_eq!(passive_nodes(&frame_a), passive_nodes(&frame_b));
    let OverlaySceneSurface::Menu { overlay, .. } = &mut scene.surfaces_mut()[0] else {
        panic!("menu surface");
    };
    overlay.menu.replace_items(items(true));
    let (route_c, output_c, frame_c) =
        run_observed(&mut scene, &mut memory, UiInput::default(), None);
    let before = passive_nodes(&frame_b);
    let after = passive_nodes(&frame_c);
    assert_eq!(
        before
            .iter()
            .map(|row| (&row.0, &row.1))
            .collect::<Vec<_>>(),
        after.iter().map(|row| (&row.0, &row.1)).collect::<Vec<_>>()
    );
    assert_eq!(before[0].2, after[0].2);
    for index in 1..before.len() {
        assert_eq!(
            before[index].2.y - scene.metrics().row_height,
            after[index].2.y
        );
    }
    for (route, output, frame) in [
        (route_a, &output_a, &frame_a),
        (route_b, &output_b, &frame_b),
        (route_c, &output_c, &frame_c),
    ] {
        assert_passive_isolation(route, output, frame);
        assert!(!passive_nodes(frame).iter().any(|row| row.0 == final_id));
    }
    assert_eq!(memory.focused(), Some(final_id));
    assert_eq!(
        frame_c
            .semantics
            .focus_order()
            .into_iter()
            .filter(|id| *id != WidgetId::from_raw(73))
            .collect::<Vec<_>>(),
        [final_id]
    );
}
