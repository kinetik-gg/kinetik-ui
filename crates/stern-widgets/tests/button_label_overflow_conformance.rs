//! Windowless conformance for retained standard-button label end ellipsis.

use stern_core::{
    CursorShape, FrameOutput, PlatformRequest, Point, PointerButtonState, PointerInput, Primitive,
    Rect, Response, SemanticRole, TextPrimitive, UiInput, UiMemory, WidgetId, default_dark_theme,
};
use stern_text::{TextFeatureSet, TextLayoutStore, TextOverflow};
use stern_widgets::{Ui, button};

const BUTTON: Rect = Rect::new(7.0, 11.0, 119.3, 28.0);

fn retained_button(
    store: &mut TextLayoutStore,
    memory: &mut UiMemory,
    rect: Rect,
    source: &str,
    disabled: bool,
    input: &UiInput,
) -> (Response, FrameOutput) {
    let theme = default_dark_theme();
    let mut ui = Ui::new(input, memory, &theme).with_text_layouts(store);
    let response = ui.button("retained-button", rect, source, disabled);
    (response, ui.finish_output())
}

fn button_text<'a>(frame: &'a FrameOutput, source: &str) -> &'a TextPrimitive {
    frame
        .primitives
        .iter()
        .find_map(|primitive| match primitive {
            Primitive::Text(text) if text.text == source => Some(text),
            _ => None,
        })
        .expect("standard button label primitive")
}

fn marker_count(store: &TextLayoutStore, text: &TextPrimitive) -> usize {
    store
        .stored_layout(text.layout.expect("registered button label"))
        .expect("resident button label")
        .layout
        .runs
        .iter()
        .flat_map(|run| &run.glyphs)
        .filter(|glyph| glyph.elided)
        .count()
}

#[test]
fn exact_width_matrix_preserves_formula_bits_and_positive_endpoint_equality() {
    let theme = default_dark_theme();
    assert_eq!(theme.controls.padding_x.to_bits(), 8.0_f32.to_bits());
    let cases = [
        (119.3_f32, 0x42CE_999A_u32),
        (80.0_f32, 0x4280_0000_u32),
        (16.0_f32, 0.0_f32.to_bits()),
        (15.999_f32, 0.0_f32.to_bits()),
        (1.0_f32, 0.0_f32.to_bits()),
    ];

    for (rect_width, expected_bits) in cases {
        let rect = Rect::new(BUTTON.x, BUTTON.y, rect_width, BUTTON.height);
        let mut store = TextLayoutStore::new();
        let mut memory = UiMemory::new();
        let (_, frame) = retained_button(
            &mut store,
            &mut memory,
            rect,
            "Exact button label width",
            false,
            &UiInput::default(),
        );
        let label = button_text(&frame, "Exact button label width");
        let stored = store
            .stored_layout(label.layout.expect("explicit button label layout"))
            .expect("resident button label layout");
        let raw_span = rect.width - theme.controls.padding_x * 2.0_f32;
        let label_width = raw_span.max(0.0_f32);

        assert_eq!(stored.key.width_bits, label_width.to_bits());
        assert_eq!(stored.key.width_bits, expected_bits);
        assert_eq!(stored.key.overflow, TextOverflow::EndEllipsis);
        if label_width.is_finite() && label_width > 0.0 {
            assert_eq!(
                (label.origin.x + label_width).to_bits(),
                (rect.max_x() - theme.controls.padding_x).to_bits()
            );
        }
    }
}

#[test]
fn long_standard_button_registers_complete_source_and_one_end_marker() {
    let source =
        "Complete standard button source remains intact while its retained presentation elides";
    let mut store = TextLayoutStore::new();
    let mut memory = UiMemory::new();
    let (response, frame) = retained_button(
        &mut store,
        &mut memory,
        BUTTON,
        source,
        false,
        &UiInput::default(),
    );
    let label = button_text(&frame, source);
    let id = label.layout.expect("explicit retained button layout");
    let stored = store
        .stored_layout(id)
        .expect("resident retained button layout");

    assert_eq!(stored.key.text, source);
    assert_eq!(stored.key.style.family, label.family);
    assert_eq!(stored.key.style.size_bits, label.size.to_bits());
    assert_eq!(
        stored.key.style.line_height_bits,
        label.line_height.to_bits()
    );
    assert_eq!(stored.key.style.features, TextFeatureSet::NONE);
    assert!(!stored.key.wrap);
    assert_eq!(stored.key.overflow, TextOverflow::EndEllipsis);
    assert!(stored.layout.is_elided());
    assert_eq!(marker_count(&store, label), 1);
    assert_eq!(label.text, source);
    assert_eq!(response.rect, BUTTON);
    assert_eq!(frame.semantics.nodes().len(), 1);
    assert_eq!(frame.semantics.nodes()[0].id, response.id);
    assert_eq!(frame.semantics.nodes()[0].role, SemanticRole::Button);
    assert_eq!(frame.semantics.nodes()[0].label.as_deref(), Some(source));
    assert!(frame.warnings.is_empty());
}

#[test]
fn fitting_empty_layoutless_and_direct_buttons_keep_complete_sources() {
    for source in ["Fit", ""] {
        let mut store = TextLayoutStore::new();
        let mut memory = UiMemory::new();
        let (_, frame) = retained_button(
            &mut store,
            &mut memory,
            BUTTON,
            source,
            false,
            &UiInput::default(),
        );
        let label = button_text(&frame, source);
        let stored = store
            .stored_layout(label.layout.expect("explicit fitting button policy"))
            .expect("resident fitting button policy");
        assert_eq!(stored.key.text, source);
        assert_eq!(stored.key.overflow, TextOverflow::EndEllipsis);
        assert!(!stored.layout.is_elided());
        assert_eq!(marker_count(&store, label), 0);
        assert_eq!(label.text, source);
        assert_eq!(frame.semantics.nodes()[0].label.as_deref(), Some(source));
    }

    let source = "Layoutless retained facade keeps the complete button source";
    let theme = default_dark_theme();
    let input = UiInput::default();
    let mut memory = UiMemory::new();
    let mut ui = Ui::new(&input, &mut memory, &theme);
    let response = ui.button("layoutless", BUTTON, source, false);
    let frame = ui.finish_output();
    assert_eq!(button_text(&frame, source).layout, None);
    assert_eq!(frame.semantics.nodes()[0].label.as_deref(), Some(source));
    assert_eq!(response.rect, BUTTON);

    let direct = button(
        WidgetId::from_key("direct-button"),
        Rect::new(1.0, 2.0, 8.0, 20.0),
        source,
        &UiInput::default(),
        &mut UiMemory::new(),
        &theme,
        false,
    );
    let direct_label = direct
        .primitives
        .iter()
        .find_map(|primitive| match primitive {
            Primitive::Text(text) => Some(text),
            _ => None,
        })
        .expect("direct button label");
    assert_eq!(direct_label.text, source);
    assert_eq!(direct_label.layout, None);
    assert_eq!(direct.semantics[0].label.as_deref(), Some(source));
    assert_eq!(direct_label.origin, Point::new(9.0, direct_label.origin.y));
}

#[test]
fn narrow_nonpositive_and_multiline_labels_keep_registered_full_source_policy() {
    for width in [16.0_f32, 15.999, 1.0, 0.0, -20.0] {
        let source = "Complete narrow button source remains visible";
        let rect = Rect::new(BUTTON.x, BUTTON.y, width, BUTTON.height);
        let mut store = TextLayoutStore::new();
        let mut memory = UiMemory::new();
        let (_, frame) = retained_button(
            &mut store,
            &mut memory,
            rect,
            source,
            false,
            &UiInput::default(),
        );
        let label = button_text(&frame, source);
        let stored = store
            .stored_layout(label.layout.expect("registered zero-width button policy"))
            .expect("resident zero-width button policy");

        assert_eq!(stored.key.width_bits, 0.0_f32.to_bits());
        assert_eq!(stored.key.overflow, TextOverflow::EndEllipsis);
        assert_eq!(stored.key.text, source);
        assert!(!stored.layout.is_elided());
        assert_eq!(marker_count(&store, label), 0);
        assert_eq!(label.text, source);
        assert_eq!(frame.semantics.nodes()[0].label.as_deref(), Some(source));
    }

    for source in [
        "First complete line\nSecond complete line",
        "First complete line\r\nSecond complete line",
        "First complete paragraph\u{2029}Second complete paragraph",
    ] {
        let mut store = TextLayoutStore::new();
        let mut memory = UiMemory::new();
        let (_, frame) = retained_button(
            &mut store,
            &mut memory,
            BUTTON,
            source,
            false,
            &UiInput::default(),
        );
        let label = button_text(&frame, source);
        let stored = store
            .stored_layout(label.layout.expect("registered multiline button policy"))
            .expect("resident multiline button policy");

        assert_eq!(stored.key.text, source);
        assert_eq!(stored.key.overflow, TextOverflow::EndEllipsis);
        assert!(!stored.layout.is_elided());
        assert_eq!(marker_count(&store, label), 0);
        assert_eq!(label.text, source);
        assert_eq!(frame.semantics.nodes()[0].label.as_deref(), Some(source));
    }
}

#[test]
fn over_budget_source_rejects_without_store_mutation_or_identity_leak() {
    const RETAINED_PAYLOAD_CEILING: usize = 32 * 1024 * 1024;

    let mut store = TextLayoutStore::new();
    let mut memory = UiMemory::new();
    let _ = retained_button(
        &mut store,
        &mut memory,
        BUTTON,
        "Warm retained button label",
        false,
        &UiInput::default(),
    );
    let accounting = (
        store.len(),
        store.retained_payload_bytes(),
        store.change_cursor(),
    );

    let source = "x".repeat(RETAINED_PAYLOAD_CEILING + 1);
    let (response, frame) = retained_button(
        &mut store,
        &mut memory,
        BUTTON,
        &source,
        false,
        &UiInput::default(),
    );
    let label = button_text(&frame, &source);

    assert_eq!(label.layout, None);
    assert_eq!(label.text, source);
    assert_eq!(
        (
            store.len(),
            store.retained_payload_bytes(),
            store.change_cursor()
        ),
        accounting
    );
    assert_eq!(frame.primitives.len(), 2);
    assert_eq!(frame.semantics.nodes().len(), 1);
    assert_eq!(frame.semantics.nodes()[0].id, response.id);
    assert_eq!(
        frame.semantics.nodes()[0].label.as_deref(),
        Some(source.as_str())
    );
    assert!(frame.actions.is_empty());
    assert!(frame.warnings.is_empty());
}

#[test]
fn hot_frames_translation_source_and_width_obey_retained_identity_boundaries() {
    let source = "Stable complete button source remains retained across hot frames";
    let mut store = TextLayoutStore::new();
    let mut memory = UiMemory::new();
    let (_, first) = retained_button(
        &mut store,
        &mut memory,
        BUTTON,
        source,
        false,
        &UiInput::default(),
    );
    let first_label = button_text(&first, source);
    let first_id = first_label
        .layout
        .expect("initial retained button identity");
    let first_origin = first_label.origin;
    let first_width_bits = store
        .stored_layout(first_id)
        .expect("initial retained button entry")
        .key
        .width_bits;
    let accounting = (
        store.len(),
        store.retained_payload_bytes(),
        store.change_cursor(),
    );

    for _ in 0..4 {
        let (_, frame) = retained_button(
            &mut store,
            &mut memory,
            BUTTON,
            source,
            false,
            &UiInput::default(),
        );
        assert_eq!(button_text(&frame, source).layout, Some(first_id));
        assert_eq!(
            (
                store.len(),
                store.retained_payload_bytes(),
                store.change_cursor()
            ),
            accounting
        );
    }

    let translated = Rect::new(
        BUTTON.x + 40.0,
        BUTTON.y + 20.0,
        BUTTON.width,
        BUTTON.height,
    );
    let (_, moved) = retained_button(
        &mut store,
        &mut memory,
        translated,
        source,
        false,
        &UiInput::default(),
    );
    let moved_label = button_text(&moved, source);
    assert_eq!(moved_label.layout, Some(first_id));
    assert_eq!(
        store
            .stored_layout(first_id)
            .expect("translated retained button entry")
            .key
            .width_bits,
        first_width_bits
    );
    assert_eq!(
        (moved_label.origin.x - first_origin.x).to_bits(),
        40.0_f32.to_bits()
    );
    assert_eq!(
        (moved_label.origin.y - first_origin.y).to_bits(),
        20.0_f32.to_bits()
    );
    assert_eq!(
        (
            store.len(),
            store.retained_payload_bytes(),
            store.change_cursor()
        ),
        accounting
    );

    let changed_source = "Distinct complete button source receives distinct retained identity";
    let (_, changed) = retained_button(
        &mut store,
        &mut memory,
        BUTTON,
        changed_source,
        false,
        &UiInput::default(),
    );
    let changed_id = button_text(&changed, changed_source)
        .layout
        .expect("changed-source button identity");
    assert_ne!(changed_id, first_id);

    let wider = Rect::new(BUTTON.x, BUTTON.y, BUTTON.width + 20.0, BUTTON.height);
    let (_, resized) = retained_button(
        &mut store,
        &mut memory,
        wider,
        source,
        false,
        &UiInput::default(),
    );
    let resized_id = button_text(&resized, source)
        .layout
        .expect("resized button identity");
    assert_ne!(resized_id, first_id);
    assert_ne!(resized_id, changed_id);
    assert_ne!(
        store
            .stored_layout(resized_id)
            .expect("resized retained button entry")
            .key
            .width_bits,
        first_width_bits
    );
}

#[test]
fn interaction_states_preserve_label_identity_and_existing_surface_order() {
    let source = "Complete stateful button source retains one presentation identity";
    let mut store = TextLayoutStore::new();

    let mut default_memory = UiMemory::new();
    let (default_response, default_frame) = retained_button(
        &mut store,
        &mut default_memory,
        BUTTON,
        source,
        false,
        &UiInput::default(),
    );
    let expected_id = button_text(&default_frame, source)
        .layout
        .expect("default button label identity");

    let hover_input = UiInput {
        pointer: PointerInput {
            position: Some(BUTTON.center()),
            ..PointerInput::default()
        },
        ..UiInput::default()
    };
    let mut hover_memory = UiMemory::new();
    let (hover_response, hover_frame) = retained_button(
        &mut store,
        &mut hover_memory,
        BUTTON,
        source,
        false,
        &hover_input,
    );

    let pressed_input = UiInput {
        pointer: PointerInput {
            position: Some(BUTTON.center()),
            primary: PointerButtonState::new(true, true, false),
            ..PointerInput::default()
        },
        ..UiInput::default()
    };
    let mut pressed_memory = UiMemory::new();
    let (pressed_response, pressed_frame) = retained_button(
        &mut store,
        &mut pressed_memory,
        BUTTON,
        source,
        false,
        &pressed_input,
    );

    let mut focused_memory = UiMemory::new();
    focused_memory.focus(default_response.id);
    let (focused_response, focused_frame) = retained_button(
        &mut store,
        &mut focused_memory,
        BUTTON,
        source,
        false,
        &UiInput::default(),
    );

    let mut disabled_memory = UiMemory::new();
    let (disabled_response, disabled_frame) = retained_button(
        &mut store,
        &mut disabled_memory,
        BUTTON,
        source,
        true,
        &hover_input,
    );

    assert!(!default_response.state.hovered);
    assert!(hover_response.state.hovered);
    assert!(pressed_response.state.pressed);
    assert!(focused_response.state.focused);
    assert!(disabled_response.state.disabled);
    assert!(!disabled_response.state.hovered);
    assert!(!disabled_response.state.focused);

    for frame in [
        &default_frame,
        &hover_frame,
        &pressed_frame,
        &focused_frame,
        &disabled_frame,
    ] {
        let label = button_text(frame, source);
        assert_eq!(label.layout, Some(expected_id));
        assert_eq!(label.text, source);
        assert!(matches!(frame.primitives.first(), Some(Primitive::Rect(_))));
        assert!(matches!(frame.primitives.last(), Some(Primitive::Text(_))));
        assert_eq!(frame.semantics.nodes().len(), 1);
        assert_eq!(frame.semantics.nodes()[0].label.as_deref(), Some(source));
    }

    for frame in [
        &default_frame,
        &hover_frame,
        &pressed_frame,
        &disabled_frame,
    ] {
        assert_eq!(frame.primitives.len(), 2);
    }
    assert_eq!(focused_frame.primitives.len(), 4);
    assert!(matches!(focused_frame.primitives[1], Primitive::Path(_)));
    assert!(matches!(focused_frame.primitives[2], Primitive::Path(_)));
    assert!(
        hover_frame
            .platform_requests
            .contains(&PlatformRequest::SetCursor(CursorShape::PointingHand))
    );
    assert!(
        pressed_frame
            .platform_requests
            .contains(&PlatformRequest::SetCursor(CursorShape::PointingHand))
    );
    assert!(
        !disabled_frame
            .platform_requests
            .contains(&PlatformRequest::SetCursor(CursorShape::PointingHand))
    );
}
