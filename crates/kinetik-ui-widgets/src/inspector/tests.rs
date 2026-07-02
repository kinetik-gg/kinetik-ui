use super::{
    PropertyGridAffordanceLayout, PropertyGridError, PropertyGridLayout, PropertyGridRow,
    PropertyGridRowAffordances, PropertyGridRowState, PropertyGridRowStatus,
    PropertyGridStatusSeverity, VectorComponentLayout, VectorComponentRect,
    property_grid_row_affordance_controls, property_grid_row_affordance_rects,
    property_grid_row_status_semantics, vector2_component_rects, vector3_component_rects,
    vector4_component_rects,
};
use crate::ItemId;
use kinetik_ui_core::{
    Point, PointerButtonState, PointerInput, Rect, SemanticActionKind, SemanticRole, SemanticValue,
    UiInput, UiMemory, WidgetId, default_dark_theme,
};

fn assert_approx(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < f32::EPSILON,
        "expected {actual} to equal {expected}"
    );
}

fn assert_rect_finite(rect: Rect) {
    assert!(rect.x.is_finite(), "rect x must be finite: {rect:?}");
    assert!(rect.y.is_finite(), "rect y must be finite: {rect:?}");
    assert!(
        rect.width.is_finite(),
        "rect width must be finite: {rect:?}"
    );
    assert!(
        rect.height.is_finite(),
        "rect height must be finite: {rect:?}"
    );
}

fn assert_vector_components_finite_and_non_overlapping(components: &[VectorComponentRect]) {
    for component in components {
        assert_rect_finite(component.rect);
        assert_rect_finite(component.label_rect);
        assert_rect_finite(component.value_rect);
        assert!(component.label_rect.max_x() <= component.value_rect.x);
        assert!(component.value_rect.max_x() <= component.rect.max_x());
    }

    for pair in components.windows(2) {
        assert!(pair[0].rect.max_x() <= pair[1].rect.x);
    }
}

fn rows() -> Vec<PropertyGridRow> {
    vec![
        PropertyGridRow::section(ItemId::from_raw(1), "Transform"),
        PropertyGridRow::property(ItemId::from_raw(2), "Position", 0),
        PropertyGridRow::property(ItemId::from_raw(3), "X", 1),
        PropertyGridRow::property(ItemId::from_raw(4), "Y", 1),
    ]
}

fn pointer_input(x: f32, y: f32, down: bool, pressed: bool, released: bool) -> UiInput {
    UiInput {
        pointer: PointerInput {
            position: Some(Point::new(x, y)),
            primary: PointerButtonState::new(down, pressed, released),
            ..PointerInput::default()
        },
        ..UiInput::default()
    }
}

#[test]
fn property_grid_validates_duplicate_row_ids() {
    let rows = vec![
        PropertyGridRow::property(ItemId::from_raw(1), "A", 0)
            .with_status(PropertyGridRowStatus::warning("Check value")),
        PropertyGridRow::property(ItemId::from_raw(1), "B", 0)
            .with_disabled(true)
            .with_required(true),
    ];

    assert_eq!(
        PropertyGridLayout::validate_rows(&rows),
        Err(PropertyGridError::DuplicateRowId {
            id: ItemId::from_raw(1)
        })
    );
}

#[test]
fn property_grid_row_metadata_defaults_to_neutral_state() {
    let section = PropertyGridRow::section(ItemId::from_raw(1), "Transform");
    let property = PropertyGridRow::property(ItemId::from_raw(2), "Position", 0);

    assert_eq!(section.state, PropertyGridRowState::neutral());
    assert_eq!(property.state, PropertyGridRowState::neutral());
    assert!(!section.is_interactable());
    assert!(!section.is_editable());
    assert!(property.is_interactable());
    assert!(property.is_editable());
    assert!(!property.has_blocking_error());
    assert_eq!(
        property.state.affordances,
        PropertyGridRowAffordances::neutral()
    );
    assert!(!property.can_request_reset());
    assert!(!property.can_request_keyframe_toggle());
}

#[test]
fn property_grid_row_builder_attaches_state_metadata() {
    let row = PropertyGridRow::property(ItemId::from_raw(1), "Exposure", 0)
        .with_disabled(true)
        .with_read_only(true)
        .with_required(true)
        .with_help_text("Use scene-referred values")
        .with_status(PropertyGridRowStatus::warning(
            "Value is above preview range",
        ))
        .with_resettable(true, false)
        .with_keyframeable(true, true);

    assert!(row.state.disabled);
    assert!(row.state.read_only);
    assert!(row.state.required);
    assert_eq!(
        row.state.help_text.as_deref(),
        Some("Use scene-referred values")
    );
    assert_eq!(
        row.state.status.severity,
        PropertyGridStatusSeverity::Warning
    );
    assert_eq!(
        row.state.status.message.as_deref(),
        Some("Value is above preview range")
    );
    assert_eq!(
        row.state.affordances,
        PropertyGridRowAffordances::neutral()
            .with_reset(true, false)
            .with_keyframe(true, true)
    );
    assert!(!row.is_interactable());
    assert!(!row.is_editable());
    assert!(!row.has_blocking_error());
    assert!(!row.can_request_reset());
    assert!(!row.can_request_keyframe_toggle());
}

#[test]
fn property_grid_row_helpers_reflect_editability_and_error_state() {
    let read_only =
        PropertyGridRow::property(ItemId::from_raw(1), "Script", 0).with_read_only(true);
    let disabled =
        PropertyGridRow::property(ItemId::from_raw(2), "Collider", 0).with_disabled(true);
    let error = PropertyGridRow::property(ItemId::from_raw(3), "Mass", 0)
        .with_status(PropertyGridRowStatus::error("Mass must be positive"));
    let info = PropertyGridRow::property(ItemId::from_raw(4), "Material", 0)
        .with_status(PropertyGridRowStatus::info("Inherited from parent"));

    assert!(read_only.is_interactable());
    assert!(!read_only.is_editable());
    assert!(!read_only.has_blocking_error());
    assert!(!disabled.is_interactable());
    assert!(!disabled.is_editable());
    assert!(error.is_interactable());
    assert!(error.is_editable());
    assert!(error.has_blocking_error());
    assert!(!info.has_blocking_error());
}

#[test]
fn property_grid_computes_content_and_scroll_extents() {
    let rows = rows();
    let layout = PropertyGridLayout::new(20.0, 24.0, 90.0, 8.0, 12.0);

    assert_approx(layout.content_height(&rows), 84.0);
    assert_approx(layout.max_scroll_offset(&rows, 44.0), 40.0);
    assert_approx(layout.clamp_scroll_offset(&rows, 44.0, 500.0), 40.0);
    assert_eq!(layout.visible_range(&rows, 20.0, 44.0, 0), 0..3);
    assert_eq!(layout.visible_range(&rows, 44.0, 20.0, 0), 2..3);
}

#[test]
fn property_grid_assigns_section_label_and_value_rects() {
    let rows = rows();
    let layout = PropertyGridLayout::new(20.0, 24.0, 90.0, 8.0, 12.0);
    let rects = layout.visible_row_rects(Rect::new(10.0, 100.0, 220.0, 84.0), &rows, 0.0, 0);

    assert_eq!(rects.len(), 4);
    assert_eq!(rects[0].id, ItemId::from_raw(1));
    assert_eq!(rects[0].label_rect, rects[0].rect);
    assert_approx(rects[1].label_rect.x, 10.0);
    assert_approx(rects[1].label_rect.width, 90.0);
    assert_approx(rects[1].value_rect.x, 108.0);
    assert_approx(rects[2].label_rect.x, 22.0);
    assert_approx(rects[2].value_rect.x, 120.0);
}

#[test]
fn property_grid_metadata_does_not_change_row_rectangles() {
    let plain = rows();
    let annotated = vec![
        PropertyGridRow::section(ItemId::from_raw(1), "Transform")
            .with_help_text("Object transform"),
        PropertyGridRow::property(ItemId::from_raw(2), "Position", 0)
            .with_required(true)
            .with_status(PropertyGridRowStatus::severity(
                PropertyGridStatusSeverity::Info,
            )),
        PropertyGridRow::property(ItemId::from_raw(3), "X", 1)
            .with_status(PropertyGridRowStatus::warning("Outside guide range")),
        PropertyGridRow::property(ItemId::from_raw(4), "Y", 1)
            .with_read_only(true)
            .with_status(PropertyGridRowStatus::error("Missing linked property")),
    ];
    let layout = PropertyGridLayout::new(20.0, 24.0, 90.0, 8.0, 12.0);
    let bounds = Rect::new(10.0, 100.0, 220.0, 84.0);

    assert_eq!(
        layout.visible_row_rects(bounds, &plain, 0.0, 0),
        layout.visible_row_rects(bounds, &annotated, 0.0, 0)
    );
}

#[test]
fn property_grid_status_presentation_is_deterministic() {
    assert_eq!(
        PropertyGridStatusSeverity::None.presentation().label,
        "None"
    );
    assert!(!PropertyGridStatusSeverity::None.presentation().accented);
    assert_eq!(
        PropertyGridStatusSeverity::Info.presentation().label,
        "Info"
    );
    assert!(PropertyGridStatusSeverity::Warning.presentation().accented);
    assert!(!PropertyGridStatusSeverity::Warning.presentation().blocking);
    assert!(PropertyGridStatusSeverity::Error.presentation().blocking);
    assert_eq!(
        PropertyGridRowStatus::error("Invalid").presentation(),
        PropertyGridStatusSeverity::Error.presentation()
    );
}

#[test]
fn property_grid_status_semantics_include_severity_and_message_without_layout_changes() {
    let rows = [
        PropertyGridRow::property(ItemId::from_raw(1), "Mode", 0),
        PropertyGridRow::property(ItemId::from_raw(2), "Guide", 0)
            .with_status(PropertyGridRowStatus::info("Inherited from parent")),
        PropertyGridRow::property(ItemId::from_raw(3), "Exposure", 0)
            .with_status(PropertyGridRowStatus::warning("Preview range exceeded")),
        PropertyGridRow::property(ItemId::from_raw(4), "Mass", 0)
            .with_status(PropertyGridRowStatus::error("Mass must be positive")),
    ];
    let layout = PropertyGridLayout::new(20.0, 24.0, 90.0, 8.0, 12.0);
    let bounds = Rect::new(0.0, 0.0, 240.0, 80.0);
    let rects = layout.visible_row_rects(bounds, &rows, 0.0, 0);
    let plain_rows = [
        PropertyGridRow::property(ItemId::from_raw(1), "Mode", 0),
        PropertyGridRow::property(ItemId::from_raw(2), "Guide", 0),
        PropertyGridRow::property(ItemId::from_raw(3), "Exposure", 0),
        PropertyGridRow::property(ItemId::from_raw(4), "Mass", 0),
    ];

    assert_eq!(rects, layout.visible_row_rects(bounds, &plain_rows, 0.0, 0));
    assert!(
        property_grid_row_status_semantics(WidgetId::from_key("mode"), &rows[0], rects[0])
            .is_none()
    );

    for (index, expected) in [
        (1, "Info: Inherited from parent"),
        (2, "Warning: Preview range exceeded"),
        (3, "Error: Mass must be positive"),
    ] {
        let node = property_grid_row_status_semantics(
            WidgetId::from_key(rows[index].label.as_str()),
            &rows[index],
            rects[index],
        )
        .expect("status semantics");
        let expected_label = format!("{} status", rows[index].label);
        assert_eq!(node.role, SemanticRole::Label);
        assert_eq!(node.label.as_deref(), Some(expected_label.as_str()));
        assert_eq!(node.description.as_deref(), Some(expected));
        assert_eq!(
            node.state.value,
            Some(SemanticValue::Text(expected.to_owned()))
        );
    }
}

#[test]
fn property_grid_affordance_rects_reserve_controls_without_changing_row_rect() {
    let row = PropertyGridRow::property(ItemId::from_raw(2), "Exposure", 0)
        .with_status(PropertyGridRowStatus::error("Too bright"))
        .with_resettable(true, false)
        .with_keyframeable(true, true);
    let layout = PropertyGridLayout::new(20.0, 24.0, 90.0, 8.0, 12.0);
    let row_rects = layout.visible_row_rects(
        Rect::new(0.0, 0.0, 220.0, 20.0),
        std::slice::from_ref(&row),
        0.0,
        0,
    );
    let row_rect = row_rects[0];
    let row_rect_without_status = layout.visible_row_rects(
        Rect::new(0.0, 0.0, 220.0, 20.0),
        &[
            PropertyGridRow::property(ItemId::from_raw(2), "Exposure", 0)
                .with_resettable(true, false)
                .with_keyframeable(true, true),
        ],
        0.0,
        0,
    )[0];

    assert_eq!(row_rect, row_rect_without_status);

    let affordances = property_grid_row_affordance_rects(
        &row,
        row_rect.value_rect,
        PropertyGridAffordanceLayout::new(18.0, 4.0),
    );
    assert!(affordances.reset_rect.is_some());
    assert!(affordances.keyframe_rect.is_some());
    assert!(affordances.value_rect.width < row_rect.value_rect.width);
    assert!(affordances.value_rect.max_x() <= affordances.reset_rect.unwrap().x);
}

#[test]
fn property_grid_affordance_controls_emit_requests_only() {
    let theme = default_dark_theme();
    let row = PropertyGridRow::property(ItemId::from_raw(2), "Exposure", 0)
        .with_resettable(true, false)
        .with_keyframeable(true, false);
    let rects = property_grid_row_affordance_rects(
        &row,
        Rect::new(0.0, 0.0, 88.0, 20.0),
        PropertyGridAffordanceLayout::new(18.0, 4.0),
    );
    let reset_center = rects.reset_rect.expect("reset rect").center();
    let mut memory = UiMemory::new();

    let _ = property_grid_row_affordance_controls(
        WidgetId::from_key("exposure"),
        &row,
        rects,
        &pointer_input(reset_center.x, reset_center.y, true, true, false),
        &mut memory,
        &theme,
    );
    let reset = property_grid_row_affordance_controls(
        WidgetId::from_key("exposure"),
        &row,
        rects,
        &pointer_input(reset_center.x, reset_center.y, false, false, true),
        &mut memory,
        &theme,
    );

    assert!(reset.reset_requested);
    assert!(!reset.keyframe_toggle_requested);
    assert!(!reset.requested_keyed);

    let keyframe_center = rects.keyframe_rect.expect("keyframe rect").center();
    let mut memory = UiMemory::new();
    let _ = property_grid_row_affordance_controls(
        WidgetId::from_key("exposure"),
        &row,
        rects,
        &pointer_input(keyframe_center.x, keyframe_center.y, true, true, false),
        &mut memory,
        &theme,
    );
    let keyframe = property_grid_row_affordance_controls(
        WidgetId::from_key("exposure"),
        &row,
        rects,
        &pointer_input(keyframe_center.x, keyframe_center.y, false, false, true),
        &mut memory,
        &theme,
    );

    assert!(!keyframe.reset_requested);
    assert!(keyframe.keyframe_toggle_requested);
    assert!(keyframe.requested_keyed);
    assert!(!row.state.affordances.keyframe.keyed);
}

#[test]
fn property_grid_affordance_controls_suppress_disabled_and_read_only_requests() {
    let theme = default_dark_theme();
    for row in [
        PropertyGridRow::property(ItemId::from_raw(2), "Exposure", 0)
            .with_disabled(true)
            .with_resettable(true, false)
            .with_keyframeable(true, false),
        PropertyGridRow::property(ItemId::from_raw(3), "Mass", 0)
            .with_read_only(true)
            .with_resettable(true, false)
            .with_keyframeable(true, false),
        PropertyGridRow::property(ItemId::from_raw(4), "Scale", 0)
            .with_resettable(true, true)
            .with_keyframeable(true, false),
    ] {
        let rects = property_grid_row_affordance_rects(
            &row,
            Rect::new(0.0, 0.0, 88.0, 20.0),
            PropertyGridAffordanceLayout::new(18.0, 4.0),
        );
        let reset_center = rects.reset_rect.expect("reset rect").center();
        let output = property_grid_row_affordance_controls(
            WidgetId::from_key(row.label.as_str()),
            &row,
            rects,
            &pointer_input(reset_center.x, reset_center.y, true, true, false),
            &mut UiMemory::new(),
            &theme,
        );

        assert!(!output.reset_requested);
        assert!(!output.keyframe_toggle_requested);
        assert!(
            output
                .reset_response
                .expect("reset response")
                .state
                .disabled
        );
        if row.state.disabled || row.state.read_only {
            assert!(
                output
                    .keyframe_response
                    .expect("keyframe response")
                    .state
                    .disabled
            );
        }
    }
}

#[test]
fn property_grid_affordance_controls_expose_semantics() {
    let theme = default_dark_theme();
    let row = PropertyGridRow::property(ItemId::from_raw(2), "Exposure", 0)
        .with_resettable(true, false)
        .with_keyframeable(true, true);
    let rects = property_grid_row_affordance_rects(
        &row,
        Rect::new(0.0, 0.0, 88.0, 20.0),
        PropertyGridAffordanceLayout::new(18.0, 4.0),
    );

    let output = property_grid_row_affordance_controls(
        WidgetId::from_key("exposure"),
        &row,
        rects,
        &UiInput::default(),
        &mut UiMemory::new(),
        &theme,
    );

    assert_eq!(output.widget.semantics.len(), 2);
    assert!(output.widget.semantics.iter().all(|node| {
        node.role == SemanticRole::IconButton
            && node
                .actions
                .iter()
                .any(|action| action.kind == SemanticActionKind::Invoke)
    }));
    assert!(output.widget.semantics.iter().any(|node| {
        node.label.as_deref() == Some("Reset Exposure to default") && !node.state.selected
    }));
    assert!(output.widget.semantics.iter().any(|node| {
        node.label.as_deref() == Some("Toggle keyframe for Exposure") && node.state.selected
    }));
}

#[test]
fn property_grid_sanitizes_invalid_sizes() {
    let rows = rows();
    let layout = PropertyGridLayout::new(f32::NAN, -1.0, f32::NAN, f32::NAN, -12.0);

    assert_approx(layout.content_height(&rows), 0.0);
    assert_eq!(layout.visible_range(&rows, 0.0, 44.0, 0), 0..0);
    let rects = layout.visible_row_rects(Rect::new(10.0, 20.0, 100.0, 44.0), &rows, 0.0, 0);
    assert!(rects.is_empty());
}

#[test]
fn vector_component_rects_split_vec2_vec3_and_vec4_without_overlap() {
    let layout = VectorComponentLayout::new(6.0, 10.0, 3.0, 24.0);
    let bounds = Rect::new(10.0, 20.0, 300.0, 24.0);

    let vec2 = vector2_component_rects(bounds, layout);
    assert_eq!(vec2.len(), 2);
    assert_eq!(vec2[0].label, "X");
    assert_eq!(vec2[1].label, "Y");
    assert_approx(vec2[0].rect.width, 147.0);
    assert_approx(vec2[1].rect.x, 163.0);

    let vec3 = vector3_component_rects(bounds, layout);
    assert_eq!(vec3.len(), 3);
    assert_eq!(vec3[2].label, "Z");
    assert!(vec3[0].rect.max_x() <= vec3[1].rect.x);
    assert!(vec3[1].rect.max_x() <= vec3[2].rect.x);

    let vec4 = vector4_component_rects(bounds, layout);
    assert_eq!(vec4.len(), 4);
    assert_eq!(vec4[3].label, "W");
    for component in vec4 {
        assert!(component.label_rect.max_x() <= component.value_rect.x);
        assert!(component.value_rect.max_x() <= component.rect.max_x());
    }
}

#[test]
fn vector_component_rects_clamp_narrow_and_invalid_widths() {
    let layout = VectorComponentLayout::new(f32::NAN, 12.0, f32::INFINITY, 40.0);
    let narrow = vector3_component_rects(Rect::new(0.0, 0.0, 42.0, 18.0), layout);

    assert_approx(narrow[0].rect.width, 14.0);
    assert_vector_components_finite_and_non_overlapping(&narrow);
    for component in narrow {
        assert!(component.label_rect.width <= component.rect.width);
        assert!(component.value_rect.width >= 0.0);
        assert!(component.value_rect.max_x() <= component.rect.max_x());
    }

    let invalid = vector4_component_rects(
        Rect::new(0.0, 0.0, f32::NAN, 18.0),
        VectorComponentLayout::default(),
    );
    assert!(
        invalid
            .iter()
            .all(|component| { component.rect.width == 0.0 && component.value_rect.width == 0.0 })
    );
}

#[test]
fn vector_component_rects_sanitize_invalid_gaps_for_placement() {
    let bounds = Rect::new(10.0, 20.0, 120.0, 24.0);
    let invalid_gaps = [f32::NAN, f32::INFINITY, -8.0];

    for component_gap in invalid_gaps {
        let components = vector4_component_rects(
            bounds,
            VectorComponentLayout::new(component_gap, 10.0, 3.0, 24.0),
        );
        assert_vector_components_finite_and_non_overlapping(&components);
    }
}
