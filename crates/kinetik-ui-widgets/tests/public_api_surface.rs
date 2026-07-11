//! Compile-level coverage for public widget crate access paths.

use kinetik_ui_core::{Rect, UiInput, UiMemory, WidgetId, default_dark_theme};
use kinetik_ui_widgets::{self as widgets, prelude};

#[test]
fn root_exports_common_and_compatibility_widget_items() {
    let theme = default_dark_theme();
    let output = widgets::label(Rect::new(0.0, 0.0, 80.0, 18.0), "Label", &theme);
    let panel = widgets::Panel::new(widgets::PanelId::from_raw(1), "Inspector");
    let node = widgets::NodeDescriptor::new(
        widgets::NodeId::from_raw(1),
        "Node",
        widgets::GraphRect::new(0.0, 0.0, 120.0, 80.0),
    );
    let frame_move_request = widgets::NodeGraphFrameMoveRequest {
        frame: widgets::NodeGraphFrameMove {
            frame: widgets::NodeFrameId::from_raw(2),
            delta: widgets::GraphVector::new(1.0, 0.0),
        },
        screen_delta: widgets::GraphVector::new(1.0, 0.0),
        graph_delta: widgets::GraphVector::new(1.0, 0.0),
        children: Vec::new(),
    };

    assert_eq!(output.primitives.len(), 1);
    assert_eq!(panel.title, "Inspector");
    assert_eq!(node.title, "Node");
    assert!(!frame_move_request.is_noop());
}

#[test]
fn prelude_exports_common_application_widget_items() {
    let theme = default_dark_theme();
    let mut memory = UiMemory::new();
    let button = prelude::button(
        WidgetId::from_key("button"),
        Rect::new(0.0, 0.0, 80.0, 24.0),
        "Run",
        &UiInput::default(),
        &mut memory,
        &theme,
        false,
    );
    let surface = prelude::ViewportSurface {
        texture: kinetik_ui_core::TextureId::from_raw(1),
        source_size: kinetik_ui_core::Size::new(100.0, 100.0),
        bounds: Rect::new(0.0, 0.0, 100.0, 100.0),
        pan_zoom: widgets::PanZoom::default(),
    };

    assert!(button.response.is_some());
    assert!((surface.content_scale() - 1.0).abs() <= f32::EPSILON);
}

#[test]
fn modules_remain_available_for_advanced_apis() {
    let descriptor = widgets::dock::PanelTypeDescriptor::new(
        widgets::dock::PanelTypeId::from_raw(1),
        "Inspector",
    );
    let target = widgets::node_graph::NodeGraphContextTarget::Canvas;
    let viewport_tool = widgets::viewport::ViewportToolDescriptor::new(
        widgets::viewport::ViewportToolId::from_raw(1),
        "Select",
    )
    .active(true);

    assert_eq!(descriptor.title, "Inspector");
    assert_eq!(target, widgets::node_graph::NodeGraphContextTarget::Canvas);
    assert!(viewport_tool.active);
}

#[test]
fn root_exports_access_aware_text_field_api_without_expanding_prelude_usage() {
    fn assert_traits<T: core::fmt::Debug + Clone + Copy + PartialEq + Eq + core::hash::Hash>() {}
    assert_traits::<widgets::TextFieldAccess>();

    let theme = default_dark_theme();
    let input = UiInput::default();
    let mut memory = UiMemory::new();
    let mut single = kinetik_ui_text::TextEditState::new("single");
    let mut multi = kinetik_ui_text::TextEditState::new("multi\nline");
    let mut ui = widgets::Ui::new(&input, &mut memory, &theme);
    let single_output = ui.text_field_with_access(
        "single",
        Rect::new(0.0, 0.0, 120.0, 24.0),
        &mut single,
        widgets::TextFieldAccess::ReadOnly,
    );
    let multi_output = ui.multi_line_text_field_with_access(
        "multi",
        Rect::new(0.0, 28.0, 120.0, 60.0),
        &mut multi,
        widgets::TextFieldAccess::Disabled,
    );
    let _ = ui.finish_output();

    assert!(!single_output.changed);
    assert!(!multi_output.changed);
    assert_eq!(
        widgets::TextFieldAccess::Editable,
        widgets::TextFieldAccess::Editable
    );
}
