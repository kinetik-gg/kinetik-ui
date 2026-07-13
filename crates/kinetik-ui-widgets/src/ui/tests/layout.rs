use super::{
    FrameContext, Insets, Primitive, Rect, ScaleFactor, Size, TimeInfo, Ui, UiInput, UiMemory,
    Vec2, ViewportInfo, default_dark_theme, scrolled_at,
};
use kinetik_ui_core::{LayoutItem, Measurement, SizeRule};

fn item(width: SizeRule, height: SizeRule, measured: Size) -> LayoutItem {
    LayoutItem::new(width, height, Measurement::new(measured))
}

#[test]
fn ui_layout_row_column_and_grid_allocate_measured_children() {
    let theme = default_dark_theme();
    let input = UiInput::default();
    let mut memory = UiMemory::new();
    let mut ui = Ui::new(&input, &mut memory, &theme);

    let row = ui.row(
        "row",
        Rect::new(0.0, 0.0, 100.0, 24.0),
        &[
            item(SizeRule::Fixed(20.0), SizeRule::Fill, Size::ZERO),
            item(SizeRule::Fit, SizeRule::Fill, Size::new(10.0, 8.0)),
            item(SizeRule::Fill, SizeRule::Fill, Size::ZERO),
        ],
        5.0,
        |_ui, _, rect| rect,
    );
    let column = ui.column(
        "column",
        Rect::new(0.0, 0.0, 24.0, 80.0),
        &[
            item(SizeRule::Fill, SizeRule::Fixed(20.0), Size::ZERO),
            item(
                SizeRule::Fill,
                SizeRule::MinMax {
                    min: 8.0,
                    max: 16.0,
                },
                Size::new(4.0, 30.0),
            ),
            item(SizeRule::Fill, SizeRule::Fill, Size::ZERO),
        ],
        4.0,
        |_ui, _, rect| rect,
    );
    let grid = ui.grid(
        "grid",
        Rect::new(0.0, 0.0, 100.0, 50.0),
        &[SizeRule::Fit, SizeRule::Fill],
        &[SizeRule::Fit, SizeRule::Fill],
        &[
            Measurement::new(Size::new(30.0, 10.0)),
            Measurement::new(Size::new(5.0, 12.0)),
            Measurement::new(Size::new(20.0, 8.0)),
            Measurement::default(),
        ],
        4.0,
        2.0,
        |_ui, _, rect| rect,
    );

    assert_eq!(
        row,
        vec![
            Rect::new(0.0, 0.0, 20.0, 24.0),
            Rect::new(25.0, 0.0, 10.0, 24.0),
            Rect::new(40.0, 0.0, 60.0, 24.0),
        ]
    );
    assert_eq!(
        column,
        vec![
            Rect::new(0.0, 0.0, 24.0, 20.0),
            Rect::new(0.0, 24.0, 24.0, 16.0),
            Rect::new(0.0, 44.0, 24.0, 36.0),
        ]
    );
    assert_eq!(
        grid,
        vec![
            Rect::new(0.0, 0.0, 30.0, 12.0),
            Rect::new(34.0, 0.0, 66.0, 12.0),
            Rect::new(0.0, 14.0, 30.0, 36.0),
            Rect::new(34.0, 14.0, 66.0, 36.0),
        ]
    );
}

#[test]
fn ui_layout_padding_and_stack_nest_without_manual_child_arithmetic() {
    let theme = default_dark_theme();
    let input = UiInput::default();
    let mut memory = UiMemory::new();
    let mut ui = Ui::new(&input, &mut memory, &theme);

    let layers = ui.padding(
        "padding",
        Rect::new(10.0, 20.0, 100.0, 80.0),
        Insets::new(4.0, 6.0, 8.0, 10.0),
        |ui, inner| ui.stack("stack", inner, 2, |_ui, _, rect| rect),
    );

    assert_eq!(layers, vec![Rect::new(14.0, 28.0, 90.0, 62.0); 2]);
}

#[test]
fn ui_layout_child_scopes_keep_repeated_local_keys_distinct() {
    let theme = default_dark_theme();
    let input = UiInput::default();
    let mut memory = UiMemory::new();
    let mut ui = Ui::new(&input, &mut memory, &theme);

    ui.row(
        "labels",
        Rect::new(0.0, 0.0, 100.0, 20.0),
        &[
            item(SizeRule::Fill, SizeRule::Fill, Size::ZERO),
            item(SizeRule::Fill, SizeRule::Fill, Size::ZERO),
        ],
        0.0,
        |ui, index, rect| ui.label_keyed("label", rect, format!("Label {index}")),
    );
    let output = ui.finish_output();

    assert!(output.warnings.is_empty());
    assert_eq!(output.semantics.nodes().len(), 2);
    assert_ne!(
        output.semantics.nodes()[0].id,
        output.semantics.nodes()[1].id
    );
}

#[test]
fn ui_scroll_layouts_derive_overflow_and_keep_one_transform_scope() {
    let theme = default_dark_theme();
    let items = [
        item(SizeRule::Fixed(30.0), SizeRule::Fixed(30.0), Size::ZERO),
        item(SizeRule::Fixed(30.0), SizeRule::Fixed(30.0), Size::ZERO),
    ];

    let horizontal_input = scrolled_at(8.0, 8.0, Vec2::new(-24.0, 0.0));
    let mut horizontal_memory = UiMemory::new();
    let mut horizontal_ui = Ui::new(&horizontal_input, &mut horizontal_memory, &theme);
    let horizontal = horizontal_ui.scroll_row(
        "horizontal",
        Rect::new(0.0, 0.0, 40.0, 40.0),
        &items,
        4.0,
        false,
        |_ui, _, rect| rect,
    );
    let horizontal_frame = horizontal_ui.finish_output();

    assert_eq!(horizontal.scroll.offset, Vec2::new(24.0, 0.0));
    assert_eq!(horizontal.inner[1], Rect::new(34.0, 0.0, 30.0, 30.0));
    assert_eq!(
        horizontal_frame
            .primitives
            .iter()
            .filter(|primitive| matches!(primitive, Primitive::ClipBegin { .. }))
            .count(),
        1
    );
    assert_eq!(
        horizontal_frame
            .primitives
            .iter()
            .filter(|primitive| matches!(primitive, Primitive::TransformBegin(_)))
            .count(),
        1
    );

    let vertical_input = scrolled_at(8.0, 8.0, Vec2::new(0.0, -24.0));
    let mut vertical_memory = UiMemory::new();
    let mut vertical_ui = Ui::new(&vertical_input, &mut vertical_memory, &theme);
    let vertical = vertical_ui.scroll_column(
        "vertical",
        Rect::new(0.0, 0.0, 40.0, 40.0),
        &items,
        4.0,
        false,
        |_ui, _, rect| rect,
    );
    let vertical_frame = vertical_ui.finish_output();

    assert_eq!(vertical.scroll.offset, Vec2::new(0.0, 24.0));
    assert_eq!(vertical.inner[1], Rect::new(0.0, 34.0, 30.0, 30.0));
    assert_eq!(
        vertical_frame
            .primitives
            .iter()
            .filter(|primitive| matches!(primitive, Primitive::ClipBegin { .. }))
            .count(),
        1
    );
    assert_eq!(
        vertical_frame
            .primitives
            .iter()
            .filter(|primitive| matches!(primitive, Primitive::TransformBegin(_)))
            .count(),
        1
    );
}

#[test]
fn ui_layout_allocations_are_independent_from_dpi_scale() {
    fn allocations_at(scale: f64) -> Vec<Rect> {
        let theme = default_dark_theme();
        let mut memory = UiMemory::new();
        let scale = ScaleFactor::new(scale);
        let logical_size = Size::new(100.0, 40.0);
        let context = FrameContext::new(
            ViewportInfo::new(
                logical_size,
                scale.logical_size_to_physical(logical_size),
                scale,
            ),
            UiInput::default(),
            TimeInfo::default(),
        );
        let mut ui = Ui::begin_frame(context, &mut memory, &theme);
        let allocations = ui.row(
            "row",
            Rect::new(0.25, 0.5, 99.5, 30.25),
            &[
                item(SizeRule::Fit, SizeRule::Fill, Size::new(20.25, 10.0)),
                item(SizeRule::Fill, SizeRule::Fill, Size::ZERO),
            ],
            3.5,
            |_ui, _, rect| rect,
        );
        let _ = ui.finish_output();
        allocations
    }

    let baseline = allocations_at(1.0);
    assert_eq!(allocations_at(1.25), baseline);
    assert_eq!(allocations_at(1.5), baseline);
    assert_eq!(allocations_at(1.75), baseline);
}
