//! Windowless conformance for retained property-label end ellipsis.

use stern_core::{
    FrameOutput, Primitive, Rect, SemanticRole, TextPrimitive, Theme, UiInput, UiMemory, WidgetId,
    default_dark_theme,
};
use stern_text::{TextLayoutStore, TextOverflow};
use stern_widgets::{
    ItemId, Ui,
    inspector::{
        PropertyGridAccess, PropertyGridConfig, PropertyGridLayout, PropertyGridOutput,
        PropertyGridRow, PropertyGridRowRect, PropertyGridRowStatus,
    },
};

const BOUNDS: Rect = Rect::new(10.0, 20.0, 360.0, 140.0);

#[derive(Debug, Clone, Copy, PartialEq)]
struct ValueObservation {
    row: ItemId,
    access: PropertyGridAccess,
    geometry: PropertyGridRowRect,
    value_rect: Rect,
    row_widget_id: WidgetId,
    value_widget_id: WidgetId,
}

fn layout(label_width: f32) -> PropertyGridLayout {
    PropertyGridLayout::new(24.0, 26.0, label_width, 6.0, 12.0)
}

fn retained_grid(
    store: &mut TextLayoutStore,
    memory: &mut UiMemory,
    rows: &[PropertyGridRow],
    bounds: Rect,
    config: PropertyGridConfig,
    input: &UiInput,
    theme: &Theme,
) -> (PropertyGridOutput<ValueObservation>, FrameOutput) {
    let mut ui = Ui::new(input, memory, theme).with_text_layouts(store);
    let output = ui
        .property_grid("grid", bounds, rows, config, |_, cell| {
            ValueObservation {
                row: cell.row.id,
                access: cell.access,
                geometry: cell.geometry,
                value_rect: cell.value_rect,
                row_widget_id: cell.row_widget_id(),
                value_widget_id: cell.value_widget_id(),
            }
        })
        .expect("valid property rows");
    (output, ui.finish_output())
}

fn retained_default(
    store: &mut TextLayoutStore,
    memory: &mut UiMemory,
    rows: &[PropertyGridRow],
) -> (PropertyGridOutput<ValueObservation>, FrameOutput) {
    retained_grid(
        store,
        memory,
        rows,
        BOUNDS,
        PropertyGridConfig::default(),
        &UiInput::default(),
        &default_dark_theme(),
    )
}

fn label_text<'a>(frame: &'a FrameOutput, source: &str) -> &'a TextPrimitive {
    frame
        .primitives
        .iter()
        .find_map(|primitive| match primitive {
            Primitive::Text(text) if text.text == source => Some(text),
            _ => None,
        })
        .expect("property label primitive")
}

fn marker_count(store: &TextLayoutStore, text: &TextPrimitive) -> usize {
    store
        .stored_layout(text.layout.expect("registered property label"))
        .expect("resident property label")
        .layout
        .runs
        .iter()
        .flat_map(|run| &run.glyphs)
        .filter(|glyph| glyph.elided)
        .count()
}

#[test]
fn label_width_matrix_preserves_exact_operation_order_and_positive_zero() {
    let cases = [
        (
            "No trailing state reserves no width",
            PropertyGridRow::property(ItemId::from_raw(1), "No trailing state reserves no width", 0),
            0.0_f32,
            0x42E2_999A_u32,
        ),
        (
            "Status reserves its fixed glyph origin",
            PropertyGridRow::property(
                ItemId::from_raw(2),
                "Status reserves its fixed glyph origin",
                0,
            )
            .with_status(PropertyGridRowStatus::warning("Warning")),
            10.0_f32,
            0x42CE_999A_u32,
        ),
        (
            "Help reserves its fixed glyph origin",
            PropertyGridRow::property(
                ItemId::from_raw(3),
                "Help reserves its fixed glyph origin",
                0,
            )
            .with_help_text("Help"),
            22.0_f32,
            0x42B6_999A_u32,
        ),
        (
            "Help wins over status reservation",
            PropertyGridRow::property(
                ItemId::from_raw(4),
                "Help wins over status reservation",
                0,
            )
            .with_help_text("Help")
            .with_status(PropertyGridRowStatus::error("Error")),
            22.0_f32,
            0x42B6_999A_u32,
        ),
    ];

    for (source, row, reserved_right, expected_bits) in cases {
        let config = PropertyGridConfig::new(layout(119.3)).with_overscan(0);
        let mut store = TextLayoutStore::new();
        let mut memory = UiMemory::new();
        let (output, frame) = retained_grid(
            &mut store,
            &mut memory,
            &[row],
            BOUNDS,
            config,
            &UiInput::default(),
            &default_dark_theme(),
        );
        let geometry = output.visible_rows[0];
        let label = label_text(&frame, source);
        let stored = store
            .stored_layout(label.layout.expect("explicit label layout"))
            .expect("resident label layout");
        let raw_span = (geometry.label_rect.width - 6.0_f32) - reserved_right;
        let expected_width = raw_span.max(0.0_f32);

        assert_eq!(geometry.label_rect.width.to_bits(), 119.3_f32.to_bits());
        assert_eq!(stored.key.width_bits, expected_width.to_bits());
        assert_eq!(stored.key.width_bits, expected_bits);
        assert_eq!(
            (label.origin.x + f32::from_bits(stored.key.width_bits)).to_bits(),
            (geometry.label_rect.max_x() - reserved_right).to_bits()
        );
        assert_eq!(stored.key.overflow, TextOverflow::EndEllipsis);
    }

    for (label_width, row) in [
        (
            5.0_f32,
            PropertyGridRow::property(ItemId::from_raw(11), "Tiny plain label", 0),
        ),
        (
            12.0_f32,
            PropertyGridRow::property(ItemId::from_raw(12), "Tiny status label", 0)
                .with_status(PropertyGridRowStatus::info("Info")),
        ),
        (
            20.0_f32,
            PropertyGridRow::property(ItemId::from_raw(13), "Tiny help label", 0)
                .with_help_text(""),
        ),
    ] {
        let reserved_right = if row.state.help_text.is_some() {
            22.0_f32
        } else if row.state.status.presentation().accented {
            10.0_f32
        } else {
            0.0_f32
        };
        let source = row.label.clone();
        let mut store = TextLayoutStore::new();
        let mut memory = UiMemory::new();
        let (output, frame) = retained_grid(
            &mut store,
            &mut memory,
            &[row],
            BOUNDS,
            PropertyGridConfig::new(layout(label_width)).with_overscan(0),
            &UiInput::default(),
            &default_dark_theme(),
        );
        let geometry = output.visible_rows[0];
        let raw_span = (geometry.label_rect.width - 6.0_f32) - reserved_right;
        assert!(raw_span <= 0.0);
        let label = label_text(&frame, &source);
        let stored = store
            .stored_layout(label.layout.expect("registered zero-width policy"))
            .expect("resident zero-width policy");
        assert_eq!(stored.key.width_bits, raw_span.max(0.0_f32).to_bits());
        assert_eq!(stored.key.width_bits, 0.0_f32.to_bits());
        assert_eq!(stored.key.overflow, TextOverflow::EndEllipsis);
    }
}

#[test]
fn ordinary_required_and_fitting_labels_preserve_complete_sources() {
    let long = "Complete ordinary property label source remains intact while its presentation elides";
    let required =
        "Complete required property label source keeps its presentation-only suffix while eliding";
    let rows = [
        PropertyGridRow::property(ItemId::from_raw(21), long, 0),
        PropertyGridRow::property(ItemId::from_raw(22), required, 0).with_required(true),
        PropertyGridRow::property(ItemId::from_raw(23), "Fit", 0),
    ];
    let mut store = TextLayoutStore::new();
    let mut memory = UiMemory::new();
    let (_, frame) = retained_default(&mut store, &mut memory, &rows);

    for (source, semantic, elided) in [
        (long.to_owned(), long, true),
        (format!("{required} *"), required, true),
        ("Fit".to_owned(), "Fit", false),
    ] {
        let label = label_text(&frame, &source);
        let stored = store
            .stored_layout(label.layout.expect("explicit property label layout"))
            .expect("resident property label layout");
        assert_eq!(label.text, source);
        assert_eq!(stored.key.text, source);
        assert_eq!(stored.key.style.family, label.family);
        assert_eq!(stored.key.style.size_bits, label.size.to_bits());
        assert_eq!(stored.key.style.line_height_bits, label.line_height.to_bits());
        assert!(!stored.key.wrap);
        assert_eq!(stored.key.overflow, TextOverflow::EndEllipsis);
        assert_eq!(stored.layout.is_elided(), elided);
        assert_eq!(marker_count(&store, label), usize::from(elided));
        assert!(frame.semantics.nodes().iter().any(|node| {
            node.role == SemanticRole::Row && node.label.as_deref() == Some(semantic)
        }));
    }
}
