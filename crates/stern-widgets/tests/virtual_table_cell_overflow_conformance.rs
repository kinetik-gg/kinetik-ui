//! Windowless conformance for retained virtual-table body-cell end ellipsis.

use std::time::Duration;

use stern_core::{
    FrameContext, FrameOutput, PhysicalSize, PointerOrder, Primitive, Rect, ScaleFactor,
    SemanticRole, Size, TextPrimitive, TimeInfo, UiInput, UiMemory, ViewportInfo, WidgetId,
    default_dark_theme,
};
use stern_text::{TextLayoutStore, TextOverflow};
use stern_widgets::{
    CollectionProjectedItem, CollectionProjection, ItemId, TableColumn, TableLayout, Ui,
    VirtualTableConfig, VirtualTableOutput, VirtualTableRow, VirtualTableSelection,
    VirtualTableSelectionMode,
};

const BOUNDS: Rect = Rect::new(7.0, 11.0, 320.0, 88.0);

fn id(raw: u64) -> ItemId {
    ItemId::from_raw(raw)
}

fn projection(raw_ids: &[u64]) -> CollectionProjection {
    CollectionProjection::from_source_ids(&raw_ids.iter().copied().map(id).collect::<Vec<_>>())
}

fn config(
    bounds: Rect,
    widths: impl IntoIterator<Item = f32>,
    mode: VirtualTableSelectionMode,
) -> VirtualTableConfig {
    let columns = widths
        .into_iter()
        .enumerate()
        .map(|(index, width)| {
            TableColumn::new(
                id(10 + u64::try_from(index).expect("fixture column index")),
                format!("Header {index}"),
                width,
            )
        })
        .collect();
    VirtualTableConfig::new(
        bounds,
        TableLayout {
            columns,
            header_height: 20.0,
            row_height: 20.0,
            sort: None,
        },
    )
    .label("Retained cell overflow fixture")
    .overscan(0)
    .selection_mode(mode)
    .resizable(false)
}

fn context(input: UiInput) -> FrameContext {
    FrameContext::new(
        ViewportInfo::new(
            Size::new(640.0, 360.0),
            PhysicalSize::new(640, 360),
            ScaleFactor::ONE,
        ),
        input,
        TimeInfo::new(Duration::from_millis(500), Duration::from_millis(16), 1),
    )
}

struct Run {
    root: WidgetId,
    output: VirtualTableOutput,
    callbacks: Vec<ItemId>,
    frame: FrameOutput,
}

fn run_table(
    store: Option<&mut TextLayoutStore>,
    projection: &CollectionProjection,
    config: VirtualTableConfig,
    selection: &mut VirtualTableSelection,
    memory: &mut UiMemory,
    input: UiInput,
    mut row: impl FnMut(CollectionProjectedItem) -> VirtualTableRow,
) -> Run {
    let theme = default_dark_theme();
    let mut ui = Ui::begin_frame(context(input), memory, &theme);
    if let Some(store) = store {
        ui = ui.with_text_layouts(store);
    }
    let table = ui
        .prepare_virtual_table("retained-cell-table", config, projection)
        .expect("valid retained table fixture");
    let root = table.widget_id();
    ui.resolve_pointer_targets(|plan| {
        table.declare_pointer_targets(plan, PointerOrder::new(100));
    })
    .expect("valid retained table pointer plan");
    let mut callbacks = Vec::new();
    let output = ui.virtual_table(&table, selection, |item| {
        callbacks.push(item.id);
        row(item)
    });
    Run {
        root,
        output,
        callbacks,
        frame: ui.finish_output(),
    }
}

fn body_texts<'a>(frame: &'a FrameOutput, source: &str) -> Vec<&'a TextPrimitive> {
    frame
        .primitives
        .iter()
        .filter_map(|primitive| match primitive {
            Primitive::Text(text) if text.text == source => Some(text),
            _ => None,
        })
        .collect()
}

fn body_semantics<'a>(frame: &'a FrameOutput, source: &str) -> Vec<&'a stern_core::SemanticNode> {
    frame
        .semantics
        .nodes()
        .iter()
        .filter(|node| node.role == SemanticRole::Cell && node.label.as_deref() == Some(source))
        .collect()
}

fn marker_count(store: &TextLayoutStore, text: &TextPrimitive) -> usize {
    store
        .stored_layout(text.layout.expect("registered body-cell label"))
        .expect("resident body-cell layout")
        .layout
        .runs
        .iter()
        .flat_map(|run| &run.glyphs)
        .filter(|glyph| glyph.elided)
        .count()
}

#[test]
fn exact_prepared_cell_width_matrix_preserves_formula_bits_and_pinned_endpoints() {
    let theme = default_dark_theme();
    assert_eq!(theme.controls.padding_x.to_bits(), 8.0_f32.to_bits());
    let cases = [
        (119.3_f32, 0x42CE_999A_u32, true),
        (80.0_f32, 0x4280_0000_u32, true),
        (16.0_f32, 0.0_f32.to_bits(), false),
        (15.999_f32, 0.0_f32.to_bits(), false),
        (1.0_f32, 0.0_f32.to_bits(), false),
    ];

    for (column_width, expected_bits, assert_endpoint) in cases {
        let source = format!("Exact prepared cell width {column_width:?}");
        let matrix_bounds = Rect::new(0.0, BOUNDS.y, BOUNDS.width, BOUNDS.height);
        let items = projection(&[1]);
        let mut store = TextLayoutStore::new();
        let mut memory = UiMemory::new();
        let mut selection = VirtualTableSelection::new();
        let run = run_table(
            Some(&mut store),
            &items,
            config(
                matrix_bounds,
                [column_width],
                VirtualTableSelectionMode::Cell,
            ),
            &mut selection,
            &mut memory,
            UiInput::default(),
            |_| VirtualTableRow::new([source.clone()]),
        );
        let texts = body_texts(&run.frame, &source);
        let semantics = body_semantics(&run.frame, &source);
        assert_eq!(texts.len(), 1);
        assert_eq!(semantics.len(), 1);
        let text = texts[0];
        let cell = semantics[0].bounds;
        assert_eq!(cell.width.to_bits(), column_width.to_bits());
        let stored = store
            .stored_layout(text.layout.expect("explicit cell layout"))
            .expect("resident explicit cell layout");
        let padding_x = theme.controls.padding_x;
        let raw_span = cell.width - padding_x * 2.0_f32;
        let label_width = raw_span.max(0.0_f32);
        assert_eq!(stored.key.width_bits, label_width.to_bits());
        assert_eq!(stored.key.width_bits, expected_bits);
        assert_eq!(stored.key.overflow, TextOverflow::EndEllipsis);
        assert_eq!(stored.key.text, source);
        if assert_endpoint {
            assert_eq!(
                (text.origin.x + label_width).to_bits(),
                (cell.max_x() - padding_x).to_bits()
            );
        } else {
            assert_eq!(label_width.to_bits(), 0.0_f32.to_bits());
        }
    }
}
