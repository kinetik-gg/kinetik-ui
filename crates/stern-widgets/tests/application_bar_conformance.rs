//! Windowless application-bar composition conformance.
use stern_core::{
    ActionDescriptor, Brush, FrameContext, Key, KeyEvent, KeyState, KeyboardInput, Modifiers,
    PhysicalSize, Point, PointerButtonState, PointerInput, PointerOrder, Primitive, Rect,
    ScaleFactor, SemanticActionKind, SemanticRole, Size, TimeInfo, UiInput, UiInputEvent, UiMemory,
    ViewportInfo, WidgetId, default_dark_theme,
};
use stern_widgets::{
    ApplicationBar, ApplicationBarConfig, ApplicationBarIntent, MenuBar, MenuBarMenu,
    MenuBarMenuId, Ui, WorkspaceTab, WorkspaceTabId,
};
const FILE: MenuBarMenuId = MenuBarMenuId::from_raw(1);
const VIEW: MenuBarMenuId = MenuBarMenuId::from_raw(3);
const W1: WorkspaceTabId = WorkspaceTabId::from_raw(10);
const W2: WorkspaceTabId = WorkspaceTabId::from_raw(20);
const W3: WorkspaceTabId = WorkspaceTabId::from_raw(30);
fn bar() -> ApplicationBar {
    let mut config = ApplicationBarConfig::new(
        WidgetId::from_key("app-bar"),
        Rect::new(0.25, 0.5, 360.5, 40.0),
    );
    config.menu_width = 50.25;
    config.workspace_width = 70.0;
    let mut disabled = WorkspaceTab::new(W2, "Editing", false);
    disabled.enabled = false;
    ApplicationBar::new(
        config,
        MenuBar::from_menus([
            MenuBarMenu::from_actions(FILE, "File", [ActionDescriptor::new("open", "Open")]),
            MenuBarMenu::from_actions(MenuBarMenuId::from_raw(2), "Empty", []),
            MenuBarMenu::from_actions(VIEW, "View", [ActionDescriptor::new("grid", "Grid")]),
        ]),
        [
            WorkspaceTab::new(W1, "Compositing", true),
            disabled,
            WorkspaceTab::new(W3, "Grading", false),
        ],
    )
}
fn pointer(point: Point, down: Option<bool>) -> UiInput {
    UiInput {
        pointer: PointerInput {
            position: Some(point),
            primary: down.map_or(PointerButtonState::default(), |down| {
                PointerButtonState::new(down, down, !down)
            }),
            ..PointerInput::default()
        },
        ..UiInput::default()
    }
}
fn key(key: Key) -> UiInput {
    UiInput {
        keyboard: KeyboardInput {
            events: vec![KeyEvent::new(
                key,
                KeyState::Pressed,
                Modifiers::default(),
                false,
            )],
            ..KeyboardInput::default()
        },
        ..UiInput::default()
    }
}
fn cancel(event: UiInputEvent) -> UiInput {
    let mut input = UiInput::default();
    input.push_event(event);
    input
}
fn run(
    bar: &mut ApplicationBar,
    memory: &mut UiMemory,
    input: UiInput,
) -> (stern_widgets::ApplicationBarOutput, stern_core::FrameOutput) {
    let theme = default_dark_theme();
    let context = FrameContext::new(
        ViewportInfo::new(
            Size::new(500.0, 200.0),
            PhysicalSize::new(500, 200),
            ScaleFactor::ONE,
        ),
        input,
        TimeInfo::default(),
    );
    let mut ui = Ui::begin_frame(context, memory, &theme);
    ui.resolve_pointer_targets(|plan| {
        bar.declare_pointer_targets(plan, PointerOrder::new(10));
    })
    .unwrap();
    let output = ui.application_bar(bar);
    (output, ui.finish_output())
}
#[test]
fn composition_geometry_semantics_and_ids_are_exact() {
    let theme = default_dark_theme();
    let mut bar = bar();
    let w3 = bar.workspace_widget_id(W3);
    let (output, frame) = run(&mut bar, &mut UiMemory::new(), UiInput::default());
    assert_eq!(
        bar.config.bounds.height.to_bits(),
        theme.sizes.workspace_bar.to_bits()
    );
    assert_eq!(
        output.drag_safe_regions,
        vec![Rect::new(100.75, 0.5, 50.0, 40.0)]
    );
    assert!(
        matches!(&frame.primitives[0], Primitive::Rect(rect) if rect.fill == Some(Brush::Solid(theme.colors.surface.application)))
    );
    let root = frame.semantics.get(bar.config.root).unwrap();
    assert_eq!(
        (&root.role, root.label.as_deref()),
        (
            &SemanticRole::Custom("application-bar".into()),
            Some("Application bar")
        )
    );
    assert_eq!(root.children.len(), 2);
    let menu = frame.semantics.get(root.children[0]).unwrap();
    let workspaces = frame.semantics.get(root.children[1]).unwrap();
    assert_eq!(
        (&menu.role, menu.label.as_deref()),
        (
            &SemanticRole::Custom("menu-bar".into()),
            Some("Application menu")
        )
    );
    assert_eq!(
        (&workspaces.role, workspaces.label.as_deref()),
        (&SemanticRole::TabList, Some("Workspaces"))
    );
    assert_eq!(
        menu.children
            .iter()
            .filter(|id| frame.semantics.get(**id).unwrap().focusable)
            .count(),
        1
    );
    assert_eq!(
        workspaces
            .children
            .iter()
            .filter(|id| frame.semantics.get(**id).unwrap().focusable)
            .count(),
        1
    );
    for response in &output.responses {
        let node = frame.semantics.get(response.id).unwrap();
        assert_eq!(node.bounds, response.rect);
        assert!(bar.config.bounds.contains_rect(response.rect));
        assert!(response.rect.x.is_finite() && response.rect.max_x().is_finite());
    }
    assert!(
        output
            .responses
            .windows(2)
            .all(|pair| pair[0].rect.max_x() <= pair[1].rect.x)
    );
    let active = frame.semantics.get(bar.workspace_widget_id(W1)).unwrap();
    let disabled = frame.semantics.get(bar.workspace_widget_id(W2)).unwrap();
    assert!(active.state.selected && !active.state.focused);
    assert!(disabled.state.disabled && !disabled.focusable && disabled.actions.is_empty());
    assert_eq!(
        frame.semantics.get(w3).unwrap().actions[0].kind,
        SemanticActionKind::Invoke
    );
    bar.workspaces = vec![WorkspaceTab::new(W3, "Grading", true)];
    assert_eq!(bar.workspace_widget_id(W3), w3);
}
#[test]
fn pointer_transactions_open_replace_activate_and_cancel_exactly_once() {
    let mut menu_bar = bar();
    let mut memory = UiMemory::new();
    run(
        &mut menu_bar,
        &mut memory,
        pointer(Point::new(10.0, 10.0), Some(true)),
    );
    let (opened, _) = run(
        &mut menu_bar,
        &mut memory,
        pointer(Point::new(10.0, 10.0), Some(false)),
    );
    assert!(matches!(
        opened.intents.as_slice(),
        [ApplicationBarIntent::OpenMenu { menu: FILE, .. }]
    ));
    let (replaced, _) = run(
        &mut menu_bar,
        &mut memory,
        pointer(Point::new(75.0, 10.0), None),
    );
    assert!(matches!(
        replaced.intents.as_slice(),
        [ApplicationBarIntent::OpenMenu { menu: VIEW, .. }]
    ));
    assert_ne!(memory.focused(), Some(menu_bar.menu_widget_id(VIEW)));
    let press = Point::new(325.0, 10.0);
    let mut workspace_bar = bar();
    let mut memory = UiMemory::new();
    run(&mut workspace_bar, &mut memory, pointer(press, Some(true)));
    let (activated, _) = run(&mut workspace_bar, &mut memory, pointer(press, Some(false)));
    assert!(
        matches!(activated.intents.as_slice(), [ApplicationBarIntent::ActivateWorkspace(target)] if target.id == W3)
    );
    for release in [
        pointer(Point::new(185.0, 10.0), Some(false)),
        pointer(Point::new(125.0, 10.0), Some(false)),
        cancel(UiInputEvent::PointerReleaseAll {
            position: Some(press),
        }),
        cancel(UiInputEvent::WindowFocusChanged(false)),
    ] {
        let mut candidate = bar();
        let mut memory = UiMemory::new();
        run(&mut candidate, &mut memory, pointer(press, Some(true)));
        assert!(
            !run(&mut candidate, &mut memory, release)
                .0
                .intents
                .iter()
                .any(|intent| matches!(intent, ApplicationBarIntent::ActivateWorkspace(_)))
        );
    }
}
#[test]
fn keyboard_navigation_and_focus_repair_preserve_passive_active_state() {
    let mut bar = bar();
    let mut memory = UiMemory::new();
    memory.focus(bar.workspace_widget_id(W1));
    assert!(
        run(&mut bar, &mut memory, key(Key::ArrowRight))
            .0
            .intents
            .is_empty()
    );
    assert_eq!(memory.focused(), Some(bar.workspace_widget_id(W3)));
    for movement in [Key::Home, Key::End, Key::ArrowLeft, Key::ArrowRight] {
        run(&mut bar, &mut memory, key(movement));
    }
    for activation in [Key::Enter, Key::Space] {
        let (activated, _) = run(&mut bar, &mut memory, key(activation));
        assert!(
            matches!(activated.intents.as_slice(), [ApplicationBarIntent::ActivateWorkspace(target)] if target.id == W3)
        );
    }
    assert!(bar.workspaces[0].active && !bar.workspaces[2].active);
    let (opened, _) = run(&mut bar, &mut memory, key(Key::Function(10)));
    assert!(matches!(
        opened.intents.as_slice(),
        [ApplicationBarIntent::OpenMenu { menu: FILE, .. }]
    ));
    let (moved, _) = run(&mut bar, &mut memory, key(Key::ArrowLeft));
    assert!(matches!(
        moved.intents.as_slice(),
        [ApplicationBarIntent::OpenMenu { menu: VIEW, .. }]
    ));
    run(&mut bar, &mut memory, key(Key::ArrowRight));
    let (dismissed, _) = run(&mut bar, &mut memory, key(Key::Escape));
    assert!(matches!(
        dismissed.intents.as_slice(),
        [ApplicationBarIntent::DismissMenu { menu: FILE }]
    ));
    assert_eq!(memory.focused(), Some(bar.menu_widget_id(FILE)));
    memory.focus(bar.workspace_widget_id(W3));
    run(&mut bar, &mut memory, UiInput::default());
    bar.workspaces = vec![WorkspaceTab::new(W1, "Compositing", true)];
    run(&mut bar, &mut memory, UiInput::default());
    assert_eq!(memory.focused(), Some(bar.workspace_widget_id(W1)));
    bar.workspaces[0].enabled = false;
    run(&mut bar, &mut memory, UiInput::default());
    assert_eq!(memory.focused(), None);
}
#[test]
fn invalid_empty_overlapping_and_nonfinite_geometry_fails_closed() {
    for config in [
        ApplicationBarConfig::new(WidgetId::from_key("invalid"), Rect::ZERO),
        ApplicationBarConfig {
            menu_width: f32::NAN,
            ..ApplicationBarConfig::new(
                WidgetId::from_key("invalid"),
                Rect::new(0.0, 0.0, 360.0, 40.0),
            )
        },
        ApplicationBarConfig {
            menu_width: 100.0,
            workspace_width: 80.0,
            ..ApplicationBarConfig::new(
                WidgetId::from_key("invalid"),
                Rect::new(0.0, 0.0, 360.0, 40.0),
            )
        },
    ] {
        let mut candidate = bar();
        candidate.config = config;
        let mut memory = UiMemory::new();
        let (output, frame) = run(
            &mut candidate,
            &mut memory,
            pointer(Point::new(10.0, 10.0), Some(false)),
        );
        assert!(output.intents.is_empty() && output.drag_safe_regions.is_empty());
        assert!(
            frame
                .semantics
                .get(candidate.menu_widget_id(FILE))
                .is_none()
        );
    }
}
