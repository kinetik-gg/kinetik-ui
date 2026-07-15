//! Public conformance evidence for owned virtual-table Cell focus annuli.

#![allow(clippy::cast_precision_loss, clippy::float_cmp, clippy::too_many_lines)]

use std::time::Duration;

use stern_core::{
    Brush, Color, ComponentState, FrameContext, PathElement, PhysicalSize, Point,
    PointerButtonState, PointerInput, PointerOrder, Primitive, Rect, ScaleFactor, SemanticNode,
    SemanticRole, Size, TimeInfo, UiInput, UiMemory, ViewportInfo, WidgetId, default_dark_theme,
};
use stern_widgets::{
    CollectionProjection, ItemId, TableColumn, TableLayout, Ui, VirtualTableConfig,
    VirtualTableOutput, VirtualTableRow, VirtualTableSelection, VirtualTableSelectionMode,
    VirtualTableTarget,
};

const BOUNDS: Rect = Rect::new(3.25, 7.75, 240.0, 84.0);

fn id(raw: u64) -> ItemId {
    ItemId::from_raw(raw)
}

fn projection(count: u64) -> CollectionProjection {
    CollectionProjection::from_source_ids(&(1..=count).map(id).collect::<Vec<_>>())
}

fn columns(order: [u64; 3]) -> Vec<TableColumn> {
    order
        .into_iter()
        .map(|raw| {
            let label = match raw {
                10 => "Name",
                20 => "Kind",
                30 => "Size",
                _ => unreachable!("test column"),
            };
            TableColumn::new(id(raw), label, 80.0)
        })
        .collect()
}

fn config(mode: VirtualTableSelectionMode) -> VirtualTableConfig {
    VirtualTableConfig::new(
        BOUNDS,
        TableLayout {
            columns: columns([10, 20, 30]),
            header_height: 20.25,
            row_height: 20.0,
            sort: None,
        },
    )
    .label("Assets")
    .overscan(0)
    .selection_mode(mode)
}

fn context(input: UiInput) -> FrameContext {
    FrameContext::new(
        ViewportInfo::new(
            Size::new(320.0, 160.0),
            PhysicalSize::new(320, 160),
            ScaleFactor::ONE,
        ),
        input,
        TimeInfo::new(Duration::from_millis(500), Duration::from_millis(16), 1),
    )
}

fn pointer_input(point: Point, pressed: bool, released: bool) -> UiInput {
    UiInput {
        pointer: PointerInput {
            position: Some(point),
            primary: PointerButtonState::new(pressed, pressed, released),
            ..PointerInput::default()
        },
        ..UiInput::default()
    }
}

#[derive(Clone, Copy)]
enum CellInteraction {
    Idle,
    Hover,
    Press,
}

impl CellInteraction {
    fn input(self, point: Point) -> UiInput {
        match self {
            Self::Idle => UiInput::default(),
            Self::Hover => pointer_input(point, false, false),
            Self::Press => pointer_input(point, true, false),
        }
    }

    const fn expected_hovered(self) -> bool {
        !matches!(self, Self::Idle)
    }

    const fn expected_pressed(self) -> bool {
        matches!(self, Self::Press)
    }
}

struct Run {
    root: WidgetId,
    output: VirtualTableOutput,
    callbacks: Vec<ItemId>,
    frame: stern_core::FrameOutput,
}

fn run_frame(
    projection: &CollectionProjection,
    config: VirtualTableConfig,
    selection: &mut VirtualTableSelection,
    memory: &mut UiMemory,
    input: UiInput,
) -> Run {
    let theme = default_dark_theme();
    let mut ui = Ui::begin_frame(context(input), memory, &theme);
    let table = ui
        .prepare_virtual_table("cell-focus-table", config, projection)
        .expect("valid table");
    let root = table.widget_id();
    ui.resolve_pointer_targets(|plan| {
        table.declare_pointer_targets(plan, PointerOrder::new(100));
    })
    .expect("valid table pointer plan");
    let mut callbacks = Vec::new();
    let output = ui.virtual_table(&table, selection, |item| {
        callbacks.push(item.id);
        VirtualTableRow::new([
            format!("Row {} name", item.id.raw()),
            format!("Row {} kind", item.id.raw()),
            format!("Row {} size", item.id.raw()),
        ])
    });
    Run {
        root,
        output,
        callbacks,
        frame: ui.finish_output(),
    }
}

fn cell_point(row: usize, column: usize) -> Point {
    Point::new(
        BOUNDS.x + column as f32 * 80.0 + 40.0,
        BOUNDS.y + 20.25 + row as f32 * 20.0 + 10.0,
    )
}

fn cell_target(row: usize, column: usize) -> VirtualTableTarget {
    VirtualTableTarget::Cell {
        row: id(row as u64 + 1),
        column: id([10, 20, 30][column]),
    }
}

fn select_cell(
    projection: &CollectionProjection,
    table_config: VirtualTableConfig,
    selection: &mut VirtualTableSelection,
    memory: &mut UiMemory,
    row: usize,
    column: usize,
) -> Run {
    let point = cell_point(row, column);
    let _ = run_frame(
        projection,
        table_config.clone(),
        selection,
        memory,
        pointer_input(point, true, false),
    );
    run_frame(
        projection,
        table_config,
        selection,
        memory,
        pointer_input(point, false, true),
    )
}

fn selection_response(run: &Run, target: VirtualTableTarget) -> stern_core::Response {
    run.output
        .selection_responses
        .iter()
        .find(|candidate| candidate.target == target)
        .unwrap_or_else(|| panic!("missing response for {target:?}"))
        .response
}

fn cell_base_index(run: &Run, target: VirtualTableTarget) -> usize {
    let response = selection_response(run, target);
    run.frame
        .primitives
        .iter()
        .position(
            |primitive| matches!(primitive, Primitive::Rect(base) if base.rect == response.rect),
        )
        .expect("cell base")
}

fn path_bounds(elements: &[PathElement]) -> Rect {
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for point in elements.iter().flat_map(|element| match *element {
        PathElement::MoveTo(point) | PathElement::LineTo(point) => vec![point],
        PathElement::QuadTo { ctrl, to } => vec![ctrl, to],
        PathElement::CubicTo { ctrl1, ctrl2, to } => vec![ctrl1, ctrl2, to],
        PathElement::Close => Vec::new(),
    }) {
        min_x = min_x.min(point.x);
        min_y = min_y.min(point.y);
        max_x = max_x.max(point.x);
        max_y = max_y.max(point.y);
    }
    Rect::new(min_x, min_y, max_x - min_x, max_y - min_y)
}

fn endpoint(element: &PathElement) -> Option<Point> {
    match *element {
        PathElement::MoveTo(point)
        | PathElement::LineTo(point)
        | PathElement::QuadTo { to: point, .. }
        | PathElement::CubicTo { to: point, .. } => Some(point),
        PathElement::Close => None,
    }
}

fn winding_at(elements: &[PathElement], point: Point) -> i32 {
    let mut winding = 0;
    let mut current = Point::ZERO;
    let mut start = Point::ZERO;
    for element in elements {
        if let PathElement::MoveTo(to) = *element {
            current = to;
            start = to;
            continue;
        }
        let to = if matches!(element, PathElement::Close) {
            start
        } else {
            endpoint(element).expect("drawable path endpoint")
        };
        let cross =
            (to.x - current.x) * (point.y - current.y) - (point.x - current.x) * (to.y - current.y);
        if current.y <= point.y && to.y > point.y && cross > 0.0 {
            winding += 1;
        } else if current.y > point.y && to.y <= point.y && cross < 0.0 {
            winding -= 1;
        }
        current = to;
    }
    winding
}

fn assert_focused_cell(run: &Run, target: VirtualTableTarget) -> [Primitive; 2] {
    let theme = default_dark_theme();
    let response = selection_response(run, target);
    assert!(response.state.focused);
    assert!(!response.state.disabled);
    let state = ComponentState {
        hovered: response.state.hovered,
        pressed: response.state.pressed,
        focused: response.state.focused,
        disabled: response.state.disabled,
        selected: response.state.selected,
    };
    let recipe = theme.row(state);
    let base_index = cell_base_index(run, target);
    let Primitive::Rect(base) = &run.frame.primitives[base_index] else {
        unreachable!()
    };
    assert_eq!(base.rect, response.rect);
    assert_eq!(base.fill, Some(recipe.background));
    assert_eq!(base.stroke, Some(recipe.border));
    assert_eq!(base.radius, recipe.radius);
    let expected = theme
        .focus_ring(true)
        .expect("focus recipe")
        .inward_annulus_primitives(response.rect, recipe.radius, recipe.border.width);
    assert_eq!(run.frame.primitives[base_index + 1], expected[0]);
    assert_eq!(run.frame.primitives[base_index + 2], expected[1]);
    assert!(matches!(
        run.frame.primitives[base_index + 3],
        Primitive::Text(_)
    ));
    for primitive in &run.frame.primitives[base_index + 1..=base_index + 2] {
        let Primitive::Path(path) = primitive else {
            panic!("cell focus must be a compound path");
        };
        assert_eq!(path.elements.len(), 20);
        assert_eq!(path.stroke, None);
        assert_eq!(winding_at(&path.elements, response.rect.center()), 0);
        let bounds = path_bounds(&path.elements);
        assert!(
            [
                bounds.x,
                bounds.y,
                bounds.width,
                bounds.height,
                bounds.max_x(),
                bounds.max_y(),
            ]
            .into_iter()
            .all(f32::is_finite)
        );
        assert!(response.rect.contains_rect(bounds));
    }
    assert_eq!(
        run.frame
            .primitives
            .iter()
            .filter(|primitive| matches!(primitive, Primitive::Path(_)))
            .count(),
        2
    );
    expected
}

fn output_without_focus(mut output: VirtualTableOutput) -> VirtualTableOutput {
    for response in &mut output.selection_responses {
        response.response.state.focused = false;
    }
    output
}

fn semantics_without_focus(run: &Run) -> Vec<SemanticNode> {
    run.frame
        .semantics
        .nodes()
        .iter()
        .cloned()
        .map(|mut node| {
            node.state.focused = false;
            node
        })
        .collect()
}

fn assert_focus_only_transition(focused: &Run, unfocused: &Run) {
    assert_eq!(focused.callbacks, unfocused.callbacks);
    assert_eq!(
        output_without_focus(focused.output.clone()),
        unfocused.output
    );
    assert_eq!(
        focused
            .frame
            .primitives
            .iter()
            .filter(|primitive| !matches!(primitive, Primitive::Path(_)))
            .cloned()
            .collect::<Vec<_>>(),
        unfocused.frame.primitives
    );
    assert_eq!(
        semantics_without_focus(focused),
        unfocused.frame.semantics.nodes()
    );
    assert_eq!(focused.frame.repaint, unfocused.frame.repaint);
    assert_eq!(focused.frame.actions, unfocused.frame.actions);
    assert_eq!(
        focused.frame.platform_requests,
        unfocused.frame.platform_requests
    );
    assert_eq!(focused.frame.warnings, unfocused.frame.warnings);
}

#[test]
fn every_cell_state_matrix_case_adds_only_one_exact_owned_pair_when_focused() {
    let items = projection(3);
    let table_config = config(VirtualTableSelectionMode::Cell);
    for row in 0..3 {
        for column in 0..3 {
            let target = cell_target(row, column);
            for selected in [false, true] {
                for focused in [false, true] {
                    for interaction in [
                        CellInteraction::Idle,
                        CellInteraction::Hover,
                        CellInteraction::Press,
                    ] {
                        let mut selection = VirtualTableSelection::new();
                        let mut memory = UiMemory::new();
                        let seed = run_frame(
                            &items,
                            table_config.clone(),
                            &mut selection,
                            &mut memory,
                            UiInput::default(),
                        );
                        if selected {
                            let _ = select_cell(
                                &items,
                                table_config.clone(),
                                &mut selection,
                                &mut memory,
                                row,
                                column,
                            );
                        }
                        let cell_id = seed.root.child((
                            "virtual-table-cell",
                            row as u64 + 1,
                            [10_u64, 20, 30][column],
                        ));
                        if focused {
                            memory.focus(cell_id);
                        } else {
                            memory.clear_focus();
                        }
                        let run = run_frame(
                            &items,
                            table_config.clone(),
                            &mut selection,
                            &mut memory,
                            interaction.input(cell_point(row, column)),
                        );
                        let response = selection_response(&run, target);
                        assert_eq!(response.state.selected, selected);
                        assert_eq!(response.state.focused, focused);
                        assert_eq!(response.state.hovered, interaction.expected_hovered());
                        assert_eq!(response.state.pressed, interaction.expected_pressed());
                        assert_eq!(run.output.sort_requested, None);
                        assert_eq!(run.output.resize_requested, None);
                        if focused {
                            assert_focused_cell(&run, target);
                        } else {
                            assert_eq!(
                                run.frame
                                    .primitives
                                    .iter()
                                    .filter(|primitive| matches!(primitive, Primitive::Path(_)))
                                    .count(),
                                0
                            );
                            assert!(matches!(
                                run.frame.primitives[cell_base_index(&run, target) + 1],
                                Primitive::Text(_)
                            ));
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn focused_cell_is_exactly_the_unfocused_frame_plus_two_paths_and_focus_bits() {
    let items = projection(3);
    let table_config = config(VirtualTableSelectionMode::Cell);
    let target = cell_target(1, 1);
    let mut selection = VirtualTableSelection::new();
    let mut memory = UiMemory::new();
    let seed = select_cell(
        &items,
        table_config.clone(),
        &mut selection,
        &mut memory,
        1,
        1,
    );
    let cell_id = selection_response(&seed, target).id;
    memory.clear_focus();
    let unfocused = run_frame(
        &items,
        table_config.clone(),
        &mut selection,
        &mut memory,
        UiInput::default(),
    );
    memory.focus(cell_id);
    let focused = run_frame(
        &items,
        table_config,
        &mut selection,
        &mut memory,
        UiInput::default(),
    );
    assert_focused_cell(&focused, target);
    assert_focus_only_transition(&focused, &unfocused);
}

#[test]
fn disabled_retained_cell_focus_is_inert_non_focusable_actionless_and_ring_free() {
    let items = projection(3);
    let enabled = config(VirtualTableSelectionMode::Cell);
    let target = cell_target(0, 0);
    let mut selection = VirtualTableSelection::new();
    let mut memory = UiMemory::new();
    let selected = select_cell(&items, enabled, &mut selection, &mut memory, 0, 0);
    let cell_id = selection_response(&selected, target).id;
    assert!(memory.is_focused(cell_id));
    let disabled = run_frame(
        &items,
        VirtualTableConfig {
            disabled: true,
            ..config(VirtualTableSelectionMode::Cell)
        },
        &mut selection,
        &mut memory,
        pointer_input(cell_point(0, 0), true, false),
    );
    let response = selection_response(&disabled, target);
    assert!(response.state.focused);
    assert!(response.state.disabled);
    assert!(!response.state.pressed);
    assert_eq!(
        disabled
            .frame
            .primitives
            .iter()
            .filter(|primitive| matches!(primitive, Primitive::Path(_)))
            .count(),
        0
    );
    let semantic = disabled
        .frame
        .semantics
        .get(cell_id)
        .expect("disabled cell");
    assert_eq!(semantic.role, SemanticRole::Cell);
    assert!(semantic.state.focused);
    assert!(semantic.state.disabled);
    assert!(!semantic.focusable);
    assert!(semantic.actions.is_empty());
    assert_eq!(disabled.output.sort_requested, None);
    assert_eq!(disabled.output.resize_requested, None);
}

#[test]
fn row_mode_retains_row_focus_and_exact_cell_bases_without_any_annuli() {
    let items = projection(3);
    let table_config = config(VirtualTableSelectionMode::Row);
    let mut selection = VirtualTableSelection::new();
    let mut memory = UiMemory::new();
    let point = cell_point(1, 1);
    let _ = run_frame(
        &items,
        table_config.clone(),
        &mut selection,
        &mut memory,
        pointer_input(point, true, false),
    );
    let _ = run_frame(
        &items,
        table_config.clone(),
        &mut selection,
        &mut memory,
        pointer_input(point, false, true),
    );
    let focused = run_frame(
        &items,
        table_config.clone(),
        &mut selection,
        &mut memory,
        UiInput::default(),
    );
    let target = VirtualTableTarget::Row(id(2));
    let row_response = selection_response(&focused, target);
    assert!(row_response.state.focused);
    assert!(row_response.state.selected);
    assert_eq!(
        focused
            .frame
            .primitives
            .iter()
            .filter(|primitive| matches!(primitive, Primitive::Path(_)))
            .count(),
        0
    );
    let row_semantic = focused
        .frame
        .semantics
        .get(row_response.id)
        .expect("focused row");
    assert!(row_semantic.focusable);
    assert!(row_semantic.state.focused);
    for column in [10_u64, 20, 30] {
        let cell = focused
            .frame
            .semantics
            .get(focused.root.child(("virtual-table-cell", 2_u64, column)))
            .expect("row-owned cell");
        assert!(!cell.focusable);
        assert!(!cell.state.focused);
    }
    memory.clear_focus();
    let unfocused = run_frame(
        &items,
        table_config,
        &mut selection,
        &mut memory,
        UiInput::default(),
    );
    assert_focus_only_transition(&focused, &unfocused);
}

fn linear_channel(channel: f32) -> f32 {
    if channel <= 0.040_45 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

fn contrast_ratio(foreground: Color, background: Color) -> f32 {
    let luminance = |color: Color| {
        0.2126 * linear_channel(color.r)
            + 0.7152 * linear_channel(color.g)
            + 0.0722 * linear_channel(color.b)
    };
    let foreground = luminance(foreground);
    let background = luminance(background);
    (foreground.max(background) + 0.05) / (foreground.min(background) + 0.05)
}

fn solid(brush: Brush) -> Color {
    let Brush::Solid(color) = brush else {
        panic!("expected solid brush");
    };
    color
}

fn assert_ratio(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 0.000_01,
        "{actual} != {expected}"
    );
}

fn cell_colors(run: &Run, target: VirtualTableTarget) -> (Color, Color, Color) {
    let base_index = cell_base_index(run, target);
    let Primitive::Rect(base) = &run.frame.primitives[base_index] else {
        unreachable!()
    };
    let text = run.frame.primitives[base_index + 1..]
        .iter()
        .find_map(|primitive| match primitive {
            Primitive::Text(text) => Some(text),
            _ => None,
        })
        .expect("cell text");
    (
        solid(base.fill.expect("cell fill")),
        solid(base.stroke.expect("cell border").brush),
        solid(text.brush),
    )
}

#[test]
fn production_cell_primitives_inventory_acc005_and_grid_border_nonconformities() {
    let theme = default_dark_theme();
    let items = projection(2);
    let table_config = config(VirtualTableSelectionMode::Cell);
    let target = cell_target(0, 0);

    let idle = run_frame(
        &items,
        table_config.clone(),
        &mut VirtualTableSelection::new(),
        &mut UiMemory::new(),
        UiInput::default(),
    );
    let (idle_background, idle_border, idle_text) = cell_colors(&idle, target);
    assert_ratio(contrast_ratio(idle_text, idle_background), 16.063_878);
    assert_ratio(contrast_ratio(idle_border, idle_background), 1.237_124);

    let hovered = run_frame(
        &items,
        table_config.clone(),
        &mut VirtualTableSelection::new(),
        &mut UiMemory::new(),
        CellInteraction::Hover.input(cell_point(0, 0)),
    );
    let (hover_background, hover_border, hover_text) = cell_colors(&hovered, target);
    assert_ratio(contrast_ratio(hover_text, hover_background), 13.908_798);
    assert_ratio(contrast_ratio(hover_border, hover_background), 1.071_155);

    let mut selection = VirtualTableSelection::new();
    let mut memory = UiMemory::new();
    let selected = select_cell(
        &items,
        table_config.clone(),
        &mut selection,
        &mut memory,
        0,
        0,
    );
    assert_focused_cell(&selected, target);
    let (selected_background, selected_border, selected_text) = cell_colors(&selected, target);
    assert_ratio(
        contrast_ratio(selected_text, selected_background),
        3.533_269,
    );
    assert_ratio(
        contrast_ratio(selected_border, selected_background),
        4.502_908,
    );

    let disabled = run_frame(
        &items,
        VirtualTableConfig {
            disabled: true,
            ..table_config
        },
        &mut selection,
        &mut memory,
        UiInput::default(),
    );
    let (disabled_background, disabled_border, disabled_text) = cell_colors(&disabled, target);
    assert_ratio(
        contrast_ratio(disabled_text, disabled_background),
        3.208_475,
    );
    assert_ratio(
        contrast_ratio(disabled_border, disabled_background),
        1.157_923,
    );

    let base_index = cell_base_index(&selected, target);
    let Primitive::Path(primary) = &selected.frame.primitives[base_index + 1] else {
        panic!("primary focus path");
    };
    let Primitive::Path(separator) = &selected.frame.primitives[base_index + 2] else {
        panic!("separator focus path");
    };
    let primary = solid(primary.fill.expect("primary focus fill"));
    let separator = solid(separator.fill.expect("separator focus fill"));
    assert_ratio(contrast_ratio(primary, separator), 8.555_114);
    assert_ratio(contrast_ratio(separator, selected_background), 5.570_656);
    assert_ratio(contrast_ratio(primary, selected_background), 1.535_746);
    assert_eq!(selected_background, theme.colors.selection.background);
    assert_eq!(selected_text, theme.colors.selection.foreground);
}
