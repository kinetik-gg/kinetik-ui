use super::helpers::{
    Point, Primitive, Rect, SemanticActionKind, SemanticRole, ShowcaseApp, ShowcasePage, Size,
    UiInput, click, contains_text_in_order, count_primitives, count_semantic_role, frame_context,
    has_text, semantic_node,
};
use kinetik_ui::{
    core::{CornerRadius, FrameOutput, RectPrimitive, UiMemory, default_dark_theme},
    widgets::Ui,
};

fn prior_frame_with_rects(count: usize) -> FrameOutput {
    let mut output = FrameOutput::new();
    for index in 0..count {
        output.push_primitive(Primitive::Rect(RectPrimitive {
            rect: Rect::new(index as f32, 0.0, 1.0, 1.0),
            fill: None,
            stroke: None,
            radius: CornerRadius::all(0.0),
        }));
    }
    output
}

fn primitive_texts(output: &FrameOutput) -> Vec<&str> {
    output
        .primitives
        .iter()
        .filter_map(|primitive| match primitive {
            Primitive::Text(text) => Some(text.text.as_str()),
            _ => None,
        })
        .collect()
}

#[test]
fn systems_palette_invokes_actions() {
    let mut app = ShowcaseApp::new();
    app.set_page(ShowcasePage::Systems);

    click(&mut app, Point::new(930.0, 160.0));

    assert_eq!(app.action_count(), 1);
    assert!(app.workspace_snapshot.is_some());
}

#[test]
fn systems_page_exposes_runtime_diagnostics() {
    let mut app = ShowcaseApp::new();
    app.set_page(ShowcasePage::Systems);

    let has_snapshot = app.primitives().iter().any(
        |primitive| matches!(primitive, Primitive::Text(text) if text.text == "Runtime Snapshot"),
    );

    assert!(has_snapshot);
}

#[test]
fn current_frame_diagnostics_runtime_snapshot_reads_ui_prefix() {
    let mut app = ShowcaseApp::new();
    app.output = prior_frame_with_rects(4);
    let theme = default_dark_theme();
    let mut memory = UiMemory::new();
    let mut ui = Ui::begin_frame(
        frame_context(Size::new(1440.0, 900.0), UiInput::default()),
        &mut memory,
        &theme,
    );
    ui.label(Rect::new(0.0, 0.0, 80.0, 20.0), "current prefix");

    assert_eq!(ui.output().primitives.len(), 1);

    app.draw_runtime_snapshot(&mut ui, Rect::new(20.0, 20.0, 320.0, 188.0), true);
    let output = ui.finish_output();
    let texts = primitive_texts(&output);

    assert!(texts.contains(&"Primitive count: 1"));
    assert!(texts.contains(&"#0 Text"));
    assert!(!texts.contains(&"Primitive count: 4"));
    assert!(!texts.contains(&"#0 Rect"));
}

#[test]
fn current_frame_diagnostics_chrome_badge_reads_ui_prefix() {
    let mut app = ShowcaseApp::new();
    app.output = prior_frame_with_rects(5);
    let theme = default_dark_theme();
    let mut memory = UiMemory::new();
    let mut ui = Ui::begin_frame(
        frame_context(Size::new(1440.0, 900.0), UiInput::default()),
        &mut memory,
        &theme,
    );
    ui.label(Rect::new(0.0, 0.0, 80.0, 20.0), "current prefix A");
    ui.label(Rect::new(0.0, 22.0, 80.0, 20.0), "current prefix B");

    assert_eq!(ui.output().primitives.len(), 2);

    app.chrome_status(&mut ui);
    let output = ui.finish_output();
    let texts = primitive_texts(&output);
    let primitive_label = texts
        .iter()
        .position(|text| *text == "Primitives")
        .expect("primitive status label");

    assert_eq!(texts.get(primitive_label + 1), Some(&"2"));
    assert!(!texts.contains(&"5"));
}

#[test]
fn systems_page_structural_smoke_emits_actions_overlays_palette_and_stress() {
    let mut app = ShowcaseApp::new();
    app.set_page(ShowcasePage::Systems);

    assert_eq!(app.output().warnings, Vec::new());
    assert!(app.output().primitives.len() > 180);
    assert!(count_primitives(&app, |primitive| matches!(primitive, Primitive::Rect(_))) > 130);
    assert!(count_primitives(&app, |primitive| matches!(primitive, Primitive::Text(_))) > 20);
    assert!(
        count_primitives(&app, |primitive| matches!(
            primitive,
            Primitive::ClipBegin { .. }
        )) >= 1
    );
    assert!(contains_text_in_order(
        &app,
        &[
            "Actions, Overlays, Diagnostics, Stress",
            "Action Router",
            "Overlay Stack",
            "Command Palette",
            "Primitive Stress",
            "Runtime Snapshot",
        ]
    ));

    assert!(semantic_node(
        &app,
        &SemanticRole::Button,
        "Record Dispatch"
    ));
    assert!(semantic_node(&app, &SemanticRole::Button, "Menu Save"));
    assert!(semantic_node(&app, &SemanticRole::Menu, "Menu"));
    assert!(semantic_node(
        &app,
        &SemanticRole::CommandPalette,
        "Command Palette"
    ));
    assert!(semantic_node(
        &app,
        &SemanticRole::Custom("popover".to_owned()),
        "Popover"
    ));
    assert!(count_semantic_role(&app, &SemanticRole::ListItem) >= 3);
    assert!(app.output().semantics.nodes().iter().any(|node| {
        node.role == SemanticRole::Menu
            && node
                .actions
                .iter()
                .any(|action| action.kind == SemanticActionKind::Dismiss)
    }));

    click(&mut app, Point::new(100.0, 210.0));

    assert_eq!(app.action_count(), 1);
    assert!(app.workspace_snapshot.is_some());
    assert!(has_text(&app, "Workspace snapshot captured in memory"));

    let mut app = ShowcaseApp::new();
    app.set_page(ShowcasePage::Systems);

    click(&mut app, Point::new(930.0, 160.0));

    assert_eq!(app.action_count(), 1);
    assert!(app.workspace_snapshot.is_some());
    assert!(has_text(&app, "Workspace snapshot captured in memory"));
}

#[test]
fn showcase_action_truth_disabled_palette_row_cannot_invoke() {
    let mut app = ShowcaseApp::new();
    app.set_page(ShowcasePage::Systems);

    click(&mut app, Point::new(930.0, 192.0));

    assert_eq!(app.action_count(), 0);
    assert_eq!(app.workspace_snapshot, None);
}
