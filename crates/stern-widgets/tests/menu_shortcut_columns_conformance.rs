//! Deterministic menu shortcut-column presentation conformance.

use std::{cell::RefCell, time::Duration};

use stern_core::{
    ActionContext, ActionDescriptor, ActionIcon, ActionSource, FrameContext, Key, Modifiers,
    PhysicalSize, PointerOrder, Primitive, Rect, Shortcut, ShortcutLabelLocalizer,
    ShortcutLabelToken, ShortcutModifier, ShortcutPlatform, Size, TextPrimitive, TimeInfo, UiInput,
    UiMemory, ViewportInfo, WidgetId, default_dark_theme,
};
use stern_widgets::{
    Menu, MenuItem, MenuOverlay, OverlayEntry, OverlayId, OverlayKind, OverlayScene,
    OverlaySceneOutput, OverlaySceneSurface, Ui,
};

const SURFACE_RECT: Rect = Rect::new(20.0, 20.0, 280.0, 184.0);

#[derive(Debug, Clone, PartialEq, Eq)]
enum Callback {
    Token(ShortcutPlatform, String),
    Separator(ShortcutPlatform),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Failure {
    None,
    RejectAlt,
    EmptyKey,
}

struct RecordingLocalizer {
    separator: String,
    failure: Failure,
    callbacks: RefCell<Vec<Callback>>,
}

impl RecordingLocalizer {
    fn new(separator: &str, failure: Failure) -> Self {
        Self {
            separator: separator.to_owned(),
            failure,
            callbacks: RefCell::new(Vec::new()),
        }
    }

    fn callbacks(&self) -> Vec<Callback> {
        self.callbacks.borrow().clone()
    }
}

impl ShortcutLabelLocalizer for RecordingLocalizer {
    fn token_label(
        &self,
        platform: ShortcutPlatform,
        token: ShortcutLabelToken<'_>,
    ) -> Option<String> {
        let (identity, label) = match token {
            ShortcutLabelToken::Modifier(modifier) => (
                format!("modifier:{modifier:?}"),
                match modifier {
                    ShortcutModifier::Control => "control-label-that-is-intentionally-long",
                    ShortcutModifier::Alt => "alternate-label-that-is-intentionally-long",
                    ShortcutModifier::Shift => "shift-label-that-is-intentionally-long",
                    ShortcutModifier::Super => "super-label-that-is-intentionally-long",
                }
                .to_owned(),
            ),
            ShortcutLabelToken::LogicalKey(key) => {
                (format!("logical:{key:?}"), format!("logical-key:{key:?}"))
            }
            ShortcutLabelToken::PhysicalKey(key) => {
                (format!("physical:{key:?}"), format!("physical-key:{key:?}"))
            }
        };
        self.callbacks
            .borrow_mut()
            .push(Callback::Token(platform, identity));

        if self.failure == Failure::RejectAlt
            && matches!(token, ShortcutLabelToken::Modifier(ShortcutModifier::Alt))
        {
            return None;
        }
        if self.failure == Failure::EmptyKey
            && matches!(
                token,
                ShortcutLabelToken::LogicalKey(_) | ShortcutLabelToken::PhysicalKey(_)
            )
        {
            return Some(String::new());
        }
        Some(label)
    }

    fn separator(&self, platform: ShortcutPlatform) -> &str {
        self.callbacks
            .borrow_mut()
            .push(Callback::Separator(platform));
        &self.separator
    }
}

fn shortcut(key: &str) -> Shortcut {
    Shortcut::new(
        Modifiers::new(true, true, true, false),
        Key::Character(key.to_owned()),
    )
}

fn action_with_shortcut(id: &str, label: &str, shortcut: Shortcut) -> ActionDescriptor {
    let mut action = ActionDescriptor::new(id, label);
    action.shortcut = Some(shortcut);
    action
}

fn menu_scene(rect: Rect, menu: Menu) -> OverlayScene {
    let mut scene = OverlayScene::new();
    scene.push(OverlaySceneSurface::menu(
        "Commands",
        MenuOverlay::new(
            OverlayEntry::new(OverlayId::from_raw(41), OverlayKind::Menu, rect),
            menu,
            ActionSource::Menu,
            ActionContext::Frame(WidgetId::from_key("document:alpha")),
        ),
    ));
    scene
}

fn frame_context(input: UiInput) -> FrameContext {
    FrameContext::new(
        ViewportInfo::new(
            Size::new(640.0, 480.0),
            PhysicalSize::new(640, 480),
            stern_core::ScaleFactor::ONE,
        ),
        input,
        TimeInfo::new(Duration::from_millis(400), Duration::from_millis(16), 1),
    )
}

fn run_presented(
    scene: &mut OverlayScene,
    memory: &mut UiMemory,
    input: UiInput,
    platform: ShortcutPlatform,
    localizer: &dyn ShortcutLabelLocalizer,
) -> (OverlaySceneOutput, stern_core::FrameOutput) {
    let theme = default_dark_theme();
    let mut ui = Ui::begin_frame(frame_context(input), memory, &theme);
    ui.resolve_pointer_targets(|plan| {
        scene.declare_pointer_targets(plan, PointerOrder::new(100));
    })
    .expect("valid pointer plan");
    let output = ui.overlay_scene_with_menu_presentation(scene, platform, localizer);
    (output, ui.finish_output())
}

fn text_primitives(frame: &stern_core::FrameOutput) -> Vec<&TextPrimitive> {
    frame
        .primitives
        .iter()
        .filter_map(|primitive| match primitive {
            Primitive::Text(text) => Some(text),
            _ => None,
        })
        .collect()
}

fn mixed_menu() -> (Menu, Vec<Shortcut>) {
    let primary = shortcut("p");
    let icon = shortcut("i");
    let submenu = shortcut("m");
    let disabled = shortcut("d");
    let hidden = shortcut("h");

    let mut menu = Menu::new();
    menu.push(MenuItem::Action(action_with_shortcut(
        "menu.primary",
        "A label long enough to cross every trailing column without clipping",
        primary.clone(),
    )));
    menu.push(MenuItem::Action(ActionDescriptor::new(
        "menu.no-shortcut",
        "No shortcut",
    )));
    let mut icon_action = action_with_shortcut("menu.icon", "Icon action", icon.clone());
    icon_action.icon = Some(ActionIcon::new("symbolic-save-icon"));
    menu.push(MenuItem::Action(icon_action));
    menu.push_submenu(
        action_with_shortcut("menu.submenu", "Submenu", submenu.clone()),
        Menu::from_actions([ActionDescriptor::new("submenu.child", "Child")]),
    );
    menu.push(MenuItem::Label("Section label".to_owned()));
    menu.push(MenuItem::Separator);
    let mut hidden_action = action_with_shortcut("menu.hidden", "Hidden", hidden);
    hidden_action.state.visible = false;
    menu.push(MenuItem::Action(hidden_action));
    let mut disabled_action = action_with_shortcut("menu.disabled", "Disabled", disabled.clone());
    disabled_action.state.enabled = false;
    menu.push(MenuItem::Action(disabled_action));

    (menu, vec![primary, icon, submenu, disabled])
}

#[test]
#[allow(clippy::too_many_lines)]
fn wide_mixed_menu_emits_stable_clipped_columns_and_decorative_semantics() {
    let (menu, eligible_shortcuts) = mixed_menu();
    let mut scene = menu_scene(SURFACE_RECT, menu);
    let original_scene = scene.clone();
    let localizer = RecordingLocalizer::new("::", Failure::None);
    let (output, frame) = run_presented(
        &mut scene,
        &mut UiMemory::new(),
        UiInput::default(),
        ShortcutPlatform::Windows,
        &localizer,
    );

    assert_eq!(
        scene, original_scene,
        "presentation does not mutate descriptors"
    );
    assert!(output.intents.is_empty());
    assert_eq!(
        output
            .responses
            .iter()
            .map(|response| response.rect)
            .collect::<Vec<_>>(),
        [24.0, 52.0, 80.0, 108.0].map(|y| Rect::new(24.0, y, 272.0, 28.0))
    );

    let texts = text_primitives(&frame);
    assert_eq!(texts.len(), 11);
    let label_texts = texts
        .iter()
        .filter(|text| text.origin.x == 80.0)
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(label_texts.len(), 6);
    assert_eq!(
        label_texts
            .iter()
            .map(|text| text.text.as_str())
            .collect::<Vec<_>>(),
        [
            "A label long enough to cross every trailing column without clipping",
            "No shortcut",
            "Icon action",
            "Submenu",
            "Section label",
            "Disabled",
        ]
    );
    let shortcut_texts = texts
        .iter()
        .filter(|text| text.origin.x == 152.0)
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(shortcut_texts.len(), 4);
    assert!(shortcut_texts.iter().all(|text| text.text.contains("::")));
    assert_eq!(
        texts
            .iter()
            .filter(|text| text.text == "›" && text.origin.x == 272.0)
            .count(),
        1
    );
    assert!(texts.iter().all(|text| text.text != "symbolic-save-icon"));
    assert!(texts.iter().all(|text| text.text != "Hidden"));

    let begins = frame
        .primitives
        .iter()
        .filter_map(|primitive| match primitive {
            Primitive::ClipBegin { rect, .. } => Some(*rect),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        begins.iter().filter(|rect| **rect == SURFACE_RECT).count(),
        1
    );
    assert_eq!(
        begins
            .iter()
            .filter(|rect| rect.x == 80.0 && rect.width == 40.0)
            .count(),
        6
    );
    assert_eq!(
        begins
            .iter()
            .filter(|rect| rect.x == 152.0 && rect.width == 112.0)
            .count(),
        4
    );

    let mut clip_stack = Vec::new();
    for primitive in &frame.primitives {
        match primitive {
            Primitive::ClipBegin { id, rect } => {
                if !clip_stack.is_empty() {
                    assert_ne!(
                        *rect, SURFACE_RECT,
                        "row clips stay inside the surface clip"
                    );
                }
                clip_stack.push(*id);
            }
            Primitive::ClipEnd { id } => assert_eq!(clip_stack.pop(), Some(*id)),
            _ => {}
        }
    }
    assert!(clip_stack.is_empty());

    let surface = frame
        .semantics
        .get(WidgetId::from_raw(41))
        .expect("menu surface semantics");
    assert_eq!(surface.children.len(), 7);
    for child in &surface.children {
        let node = frame.semantics.get(*child).expect("row semantics");
        assert!(node.state.value.is_none());
        assert!(node.description.is_none());
        let label = node.label.as_deref().expect("row label");
        assert!(!label.contains("::"));
        assert!(!label.contains('›'));
        assert!(!label.contains("symbolic-save-icon"));
    }
    let icon_id = WidgetId::from_raw(41)
        .child("overlay-scene")
        .child(("overlay-action", "menu.icon"));
    assert_eq!(
        frame
            .semantics
            .get(icon_id)
            .expect("icon action")
            .label
            .as_deref(),
        Some("Icon action")
    );

    let separator = frame
        .primitives
        .iter()
        .filter_map(|primitive| match primitive {
            Primitive::Rect(rect)
                if rect.rect.x == 24.0
                    && rect.rect.width == 272.0
                    && rect.rect.y >= 164.0
                    && rect.rect.y < 172.0 =>
            {
                Some(rect.rect)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(separator.len(), 1);

    let mut expected_callbacks = Vec::new();
    for shortcut in &eligible_shortcuts {
        let direct = RecordingLocalizer::new("::", Failure::None);
        assert!(
            shortcut
                .localized_label(ShortcutPlatform::Windows, &direct)
                .is_some()
        );
        expected_callbacks.extend(direct.callbacks());
    }
    assert_eq!(localizer.callbacks(), expected_callbacks);

    let repeat_localizer = RecordingLocalizer::new("::", Failure::None);
    let (_, repeated_frame) = run_presented(
        &mut scene,
        &mut UiMemory::new(),
        UiInput::default(),
        ShortcutPlatform::Windows,
        &repeat_localizer,
    );
    assert_eq!(frame.primitives, repeated_frame.primitives);
    assert_eq!(frame.semantics, repeated_frame.semantics);
}

#[test]
fn widget_uses_each_explicit_platform_and_caller_owned_localizer_policy_verbatim() {
    let shortcut = Shortcut::new(
        Modifiers::new(true, true, false, true),
        Key::Character("k".to_owned()),
    );
    for platform in [
        ShortcutPlatform::Windows,
        ShortcutPlatform::MacOs,
        ShortcutPlatform::Linux,
    ] {
        let direct = RecordingLocalizer::new(" / ", Failure::None);
        let expected = shortcut
            .localized_label(platform, &direct)
            .expect("direct core label");
        let expected_callbacks = direct.callbacks();

        let mut scene = menu_scene(
            Rect::new(20.0, 20.0, 280.0, 40.0),
            Menu::from_actions([action_with_shortcut(
                "menu.platform",
                "Platform",
                shortcut.clone(),
            )]),
        );
        let widget = RecordingLocalizer::new(" / ", Failure::None);
        let (_, frame) = run_presented(
            &mut scene,
            &mut UiMemory::new(),
            UiInput::default(),
            platform,
            &widget,
        );
        assert!(
            text_primitives(&frame)
                .iter()
                .any(|text| text.text == expected)
        );
        assert_eq!(widget.callbacks(), expected_callbacks);
        assert_eq!(
            widget
                .callbacks()
                .iter()
                .filter(|callback| matches!(callback, Callback::Separator(_)))
                .count(),
            1
        );
    }
}
