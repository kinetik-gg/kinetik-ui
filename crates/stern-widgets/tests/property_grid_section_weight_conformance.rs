//! Windowless conformance for retained property-grid section typography.

use stern_core::{
    FrameOutput, Primitive, Rect, SemanticRole, TextPrimitive, Theme, UiInput, UiMemory,
    default_dark_theme,
};
use stern_text::{TextFeatureSet, TextLayoutStore, TextOverflow, fonts};
use stern_widgets::{
    ItemId, Ui,
    inspector::{PropertyGridConfig, PropertyGridOutput, PropertyGridRow, PropertyGridRowStatus},
};

const BOUNDS: Rect = Rect::new(10.0, 20.0, 360.0, 140.0);

fn retained_grid(
    store: &mut TextLayoutStore,
    memory: &mut UiMemory,
    rows: &[PropertyGridRow],
    bounds: Rect,
    theme: &Theme,
) -> (PropertyGridOutput<()>, FrameOutput) {
    let input = UiInput::default();
    let mut ui = Ui::new(&input, memory, theme).with_text_layouts(store);
    let output = ui
        .property_grid(
            "grid",
            bounds,
            rows,
            PropertyGridConfig::default().with_overscan(0),
            |_, _| (),
        )
        .expect("valid property rows");
    (output, ui.finish_output())
}

fn text<'a>(frame: &'a FrameOutput, source: &str) -> &'a TextPrimitive {
    frame
        .primitives
        .iter()
        .find_map(|primitive| match primitive {
            Primitive::Text(text) if text.text == source => Some(text),
            _ => None,
        })
        .expect("text primitive")
}

#[test]
fn default_section_uses_exact_title_metrics_weight_and_selected_inter_face() {
    let section_source = "Transform";
    let property_source = "Opacity";
    let rows = [
        PropertyGridRow::section(ItemId::from_raw(1), section_source),
        PropertyGridRow::property(ItemId::from_raw(2), property_source, 0)
            .with_help_text("Opacity help")
            .with_status(PropertyGridRowStatus::error("Opacity error")),
    ];
    let theme = default_dark_theme();
    let mut store = TextLayoutStore::new();
    let mut memory = UiMemory::new();
    let (output, frame) = retained_grid(&mut store, &mut memory, &rows, BOUNDS, &theme);

    assert_eq!(output.visible_rows.len(), 2);
    assert_eq!(output.values.len(), 1);
    assert!(output.intents.is_empty());

    let section = text(&frame, section_source);
    assert_eq!(section.family, "Inter");
    assert_eq!(section.size.to_bits(), 14.0_f32.to_bits());
    assert_eq!(section.line_height.to_bits(), 19.0_f32.to_bits());
    assert_eq!(
        section.origin.x.to_bits(),
        (output.visible_rows[0].label_rect.x + 8.0).to_bits()
    );
    assert_eq!(
        section.origin.y.to_bits(),
        (output.visible_rows[0].label_rect.y + 20.0).to_bits()
    );

    let section_id = section.layout.expect("retained section layout");
    let retained = store
        .stored_layout(section_id)
        .expect("resident section layout");
    assert_eq!(retained.key.text.as_bytes(), section_source.as_bytes());
    assert_eq!(retained.key.style.family, section.family);
    assert_eq!(retained.key.style.size().to_bits(), section.size.to_bits());
    assert_eq!(
        retained.key.style.line_height().to_bits(),
        section.line_height.to_bits()
    );
    assert_eq!(retained.key.style.weight, 600);
    assert_eq!(retained.key.style.features, TextFeatureSet::NONE);
    assert_eq!(retained.key.width_bits, 0.0_f32.to_bits());
    assert!(!retained.key.wrap);
    assert_eq!(retained.key.overflow, TextOverflow::Visible);
    assert!(!retained.layout.is_empty());
    assert!(!retained.layout.is_elided());
    assert_eq!(retained.layout.lines.first().expect("line").text_start, 0);
    assert_eq!(
        retained.layout.lines.last().expect("line").text_end,
        section_source.len()
    );
    assert!(retained.layout.runs.iter().all(|run| {
        run.font.data.data() == fonts::INTER_VARIABLE && run.normalized_coords == [0, 5_898]
    }));

    let property = text(&frame, property_source);
    let help = text(&frame, "?");
    let status = text(&frame, "x");
    for primitive in [property, help, status] {
        assert_eq!(primitive.family, "Inter");
        assert_eq!(primitive.size.to_bits(), 12.0_f32.to_bits());
        assert_eq!(primitive.line_height.to_bits(), 16.0_f32.to_bits());
        let stored = store
            .stored_layout(primitive.layout.expect("retained label text"))
            .expect("resident label text");
        assert_eq!(stored.key.style.weight, 400);
        assert_eq!(stored.key.style.features, TextFeatureSet::NONE);
    }
    assert_eq!(
        store
            .stored_layout(property.layout.expect("property layout"))
            .expect("property entry")
            .key
            .overflow,
        TextOverflow::EndEllipsis
    );
    assert!(frame.semantics.nodes().iter().any(|node| {
        node.role == SemanticRole::Label && node.label.as_deref() == Some(section_source)
    }));
    assert!(frame.semantics.nodes().iter().any(|node| {
        node.role == SemanticRole::Row && node.label.as_deref() == Some(property_source)
    }));
}
