//! Public fixed-height virtual-list composition conformance tests.

use std::time::Duration;

use stern_core::{
    FrameContext, Key, KeyEvent, KeyState, KeyboardInput, Modifiers, PhysicalSize, Point,
    PointerButtonState, PointerInput, PointerOrder, PointerTarget, Primitive, Rect, RepaintRequest,
    Response, ScaleFactor, SemanticRole, Size, TimeInfo, UiInput, UiMemory, Vec2, ViewportInfo,
    WidgetId, default_dark_theme,
};
use stern_widgets::{
    CollectionCursor, CollectionProjection, ItemId, Selection, Ui, VirtualListConfig,
    VirtualListOutput, VirtualListRow, VirtualListSelectionMode,
};

const BOUNDS: Rect = Rect::new(0.0, 0.0, 120.0, 60.0);
const LOWER: Rect = Rect::new(0.0, 0.0, 160.0, 100.0);

fn id(raw: u64) -> ItemId {
    ItemId::from_raw(raw)
}

fn projection(raw_ids: &[u64]) -> CollectionProjection {
    CollectionProjection::from_source_ids(&raw_ids.iter().copied().map(id).collect::<Vec<_>>())
}

fn config() -> VirtualListConfig {
    VirtualListConfig::new(BOUNDS, 20.0)
        .label("Assets")
        .overscan(1)
        .selection_mode(VirtualListSelectionMode::Multiple)
}

fn context(input: UiInput) -> FrameContext {
    FrameContext::new(
        ViewportInfo::new(
            Size::new(320.0, 240.0),
            PhysicalSize::new(320, 240),
            ScaleFactor::ONE,
        ),
        input,
        TimeInfo::new(Duration::from_millis(500), Duration::from_millis(16), 1),
    )
}

fn pointer_input(x: f32, y: f32, pressed: bool, released: bool, modifiers: Modifiers) -> UiInput {
    UiInput {
        pointer: PointerInput {
            position: Some(Point::new(x, y)),
            primary: PointerButtonState::new(pressed, pressed, released),
            ..PointerInput::default()
        },
        keyboard: KeyboardInput {
            modifiers,
            events: Vec::new(),
        },
        ..UiInput::default()
    }
}

fn wheel_input(delta_y: f32) -> UiInput {
    UiInput {
        pointer: PointerInput {
            position: Some(Point::new(10.0, 10.0)),
            wheel_delta: Vec2::new(0.0, delta_y),
            ..PointerInput::default()
        },
        ..UiInput::default()
    }
}

fn key_input(key: Key, modifiers: Modifiers, repeat: bool) -> UiInput {
    UiInput {
        keyboard: KeyboardInput {
            modifiers,
            events: vec![KeyEvent::new(key, KeyState::Pressed, modifiers, repeat)],
        },
        ..UiInput::default()
    }
}

struct Run {
    list_id: WidgetId,
    lower: Option<Response>,
    output: VirtualListOutput,
    callbacks: Vec<ItemId>,
    frame: stern_core::FrameOutput,
}

fn run_frame(
    projection: &CollectionProjection,
    config: VirtualListConfig,
    cursor: &mut CollectionCursor,
    selection: &mut Selection,
    memory: &mut UiMemory,
    input: UiInput,
    lower: bool,
) -> Run {
    let theme = default_dark_theme();
    let mut ui = Ui::begin_frame(context(input), memory, &theme);
    let list = ui
        .prepare_virtual_list("list", config, projection)
        .expect("valid list");
    let list_id = list.widget_id();
    let lower_id = ui.make_id("lower");
    ui.resolve_pointer_targets(|plan| {
        if lower {
            plan.target(PointerTarget::new(lower_id, LOWER, PointerOrder::new(10)));
        }
        list.declare_pointer_targets(plan, PointerOrder::new(100));
    })
    .expect("valid shared pointer plan");
    let lower_response = lower.then(|| ui.pressable("lower", LOWER, false));
    let mut callbacks = Vec::new();
    let output = ui.virtual_list(&list, cursor, selection, |item| {
        callbacks.push(item.id);
        VirtualListRow::new(format!("Row {}", item.id.raw()))
    });
    let frame = ui.finish_output();
    Run {
        list_id,
        lower: lower_response,
        output,
        callbacks,
        frame,
    }
}

#[allow(clippy::cast_precision_loss)]
fn click_row(
    row: usize,
    modifiers: Modifiers,
    projection: &CollectionProjection,
    cursor: &mut CollectionCursor,
    selection: &mut Selection,
    memory: &mut UiMemory,
) -> Run {
    let y = row as f32 * 20.0 + 10.0;
    let _ = run_frame(
        projection,
        config(),
        cursor,
        selection,
        memory,
        pointer_input(10.0, y, true, false, modifiers),
        false,
    );
    run_frame(
        projection,
        config(),
        cursor,
        selection,
        memory,
        pointer_input(10.0, y, false, true, modifiers),
        false,
    )
}

#[test]
fn ten_thousand_rows_materialize_only_the_bounded_window() {
    let items = projection(&(0..10_000).collect::<Vec<_>>());
    let mut cursor = CollectionCursor::new();
    let mut selection = Selection::new();
    let mut memory = UiMemory::new();

    let run = run_frame(
        &items,
        config(),
        &mut cursor,
        &mut selection,
        &mut memory,
        UiInput::default(),
        false,
    );

    assert_eq!(run.output.window.visible_range, 0..3);
    assert_eq!(run.output.window.materialized_range, 0..5);
    assert_eq!(run.callbacks, vec![id(0), id(1), id(2), id(3), id(4)]);
    assert_eq!(run.output.responses.len(), 5);
    assert_eq!(
        run.frame
            .primitives
            .iter()
            .filter(|primitive| matches!(primitive, Primitive::Text(_)))
            .count(),
        5
    );
    let root = run.frame.semantics.get(run.list_id).expect("list root");
    assert_eq!(root.role, SemanticRole::List);
    assert_eq!(root.children.len(), 3);
}

#[test]
fn wheel_scroll_changes_the_next_frame_window_without_moving_current_geometry() {
    let items = projection(&(0..20).collect::<Vec<_>>());
    let mut cursor = CollectionCursor::new();
    let mut selection = Selection::new();
    let mut memory = UiMemory::new();

    let current = run_frame(
        &items,
        config(),
        &mut cursor,
        &mut selection,
        &mut memory,
        wheel_input(-40.0),
        false,
    );
    assert_eq!(current.output.window.visible_range, 0..3);
    assert_eq!(current.output.scroll.offset.y.to_bits(), 40.0_f32.to_bits());
    assert_eq!(current.callbacks[0], id(0));

    let next = run_frame(
        &items,
        config(),
        &mut cursor,
        &mut selection,
        &mut memory,
        UiInput::default(),
        false,
    );
    assert_eq!(next.output.window.visible_range, 2..5);
    assert_eq!(next.callbacks[0], id(1));
}

#[test]
fn focused_idle_frames_do_not_repaint_or_undo_manual_wheel_scroll() {
    let items = projection(&(0..20).collect::<Vec<_>>());
    let mut cursor = CollectionCursor::new();
    let mut selection = Selection::new();
    let mut memory = UiMemory::new();

    click_row(
        0,
        Modifiers::default(),
        &items,
        &mut cursor,
        &mut selection,
        &mut memory,
    );
    let idle = run_frame(
        &items,
        config(),
        &mut cursor,
        &mut selection,
        &mut memory,
        UiInput::default(),
        false,
    );
    assert_eq!(idle.frame.repaint, RepaintRequest::None);

    let wheel = run_frame(
        &items,
        config(),
        &mut cursor,
        &mut selection,
        &mut memory,
        wheel_input(-40.0),
        false,
    );
    assert_eq!(wheel.frame.repaint, RepaintRequest::NextFrame);
    let scrolled = run_frame(
        &items,
        config(),
        &mut cursor,
        &mut selection,
        &mut memory,
        UiInput::default(),
        false,
    );
    assert_eq!(scrolled.output.window.visible_range, 2..5);
    assert_eq!(scrolled.frame.repaint, RepaintRequest::None);
    assert!(memory.is_focused(scrolled.list_id.child(("virtual-list-row", 0_u64))));
}

#[test]
fn list_surface_blocks_lower_input_and_click_selects_with_ordered_semantics() {
    let items = projection(&[1, 2]);
    let mut cursor = CollectionCursor::new();
    let mut selection = Selection::new();
    let mut memory = UiMemory::new();

    let _ = run_frame(
        &items,
        config(),
        &mut cursor,
        &mut selection,
        &mut memory,
        pointer_input(10.0, 10.0, true, false, Modifiers::default()),
        true,
    );
    let released = run_frame(
        &items,
        config(),
        &mut cursor,
        &mut selection,
        &mut memory,
        pointer_input(10.0, 10.0, false, true, Modifiers::default()),
        true,
    );

    assert!(released.lower.is_some_and(|response| !response.clicked));
    assert_eq!(selection.selected(), vec![id(1)]);
    assert_eq!(cursor.active(), Some(id(1)));
    assert!(released.output.selection_changed);
    let root_position = released
        .frame
        .semantics
        .nodes()
        .iter()
        .position(|node| node.id == released.list_id)
        .expect("root position");
    let row_id = released.list_id.child(("virtual-list-row", 1_u64));
    let row_position = released
        .frame
        .semantics
        .nodes()
        .iter()
        .position(|node| node.id == row_id)
        .expect("row position");
    assert!(root_position < row_position);
    let row = released.frame.semantics.get(row_id).expect("row semantics");
    assert!(row.state.selected);
    assert!(row.state.focused);

    let _ = run_frame(
        &items,
        config(),
        &mut cursor,
        &mut selection,
        &mut memory,
        pointer_input(10.0, 50.0, true, false, Modifiers::default()),
        true,
    );
    let empty_release = run_frame(
        &items,
        config(),
        &mut cursor,
        &mut selection,
        &mut memory,
        pointer_input(10.0, 50.0, false, true, Modifiers::default()),
        true,
    );
    assert!(
        empty_release
            .lower
            .is_some_and(|response| !response.clicked)
    );
}

#[test]
fn multiple_selection_supports_toggle_and_range_modifiers() {
    let items = projection(&[1, 2, 3, 4]);
    let mut cursor = CollectionCursor::new();
    let mut selection = Selection::new();
    let mut memory = UiMemory::new();

    click_row(
        0,
        Modifiers::default(),
        &items,
        &mut cursor,
        &mut selection,
        &mut memory,
    );
    click_row(
        2,
        Modifiers::new(false, true, false, false),
        &items,
        &mut cursor,
        &mut selection,
        &mut memory,
    );
    assert_eq!(selection.selected(), vec![id(1), id(3)]);

    click_row(
        0,
        Modifiers::default(),
        &items,
        &mut cursor,
        &mut selection,
        &mut memory,
    );
    click_row(
        2,
        Modifiers::new(true, false, false, false),
        &items,
        &mut cursor,
        &mut selection,
        &mut memory,
    );
    assert_eq!(selection.selected(), vec![id(1), id(2), id(3)]);
}

#[test]
fn keyboard_navigation_selects_focuses_and_reveals_the_target() {
    let items = projection(&(0..20).collect::<Vec<_>>());
    let mut cursor = CollectionCursor::new();
    let mut selection = Selection::new();
    let mut memory = UiMemory::new();
    click_row(
        0,
        Modifiers::default(),
        &items,
        &mut cursor,
        &mut selection,
        &mut memory,
    );

    let navigated = run_frame(
        &items,
        config(),
        &mut cursor,
        &mut selection,
        &mut memory,
        key_input(Key::PageDown, Modifiers::default(), false),
        false,
    );
    assert_eq!(cursor.active(), Some(id(3)));
    assert_eq!(selection.selected(), vec![id(3)]);
    assert_eq!(
        navigated
            .output
            .cursor_target
            .map(|target| target.projected_index),
        Some(3)
    );

    let revealed = run_frame(
        &items,
        config(),
        &mut cursor,
        &mut selection,
        &mut memory,
        UiInput::default(),
        false,
    );
    assert_eq!(
        revealed.output.window.clamped_scroll_offset.to_bits(),
        20.0_f32.to_bits()
    );
    let focused = revealed.list_id.child(("virtual-list-row", 3_u64));
    assert!(memory.is_focused(focused));
}

#[test]
fn enter_and_space_activate_once_and_reject_repeat() {
    let items = projection(&[1, 2, 3]);
    let mut cursor = CollectionCursor::new();
    let mut selection = Selection::new();
    let mut memory = UiMemory::new();
    click_row(
        1,
        Modifiers::default(),
        &items,
        &mut cursor,
        &mut selection,
        &mut memory,
    );

    let enter = run_frame(
        &items,
        config(),
        &mut cursor,
        &mut selection,
        &mut memory,
        key_input(Key::Enter, Modifiers::default(), false),
        false,
    );
    assert_eq!(enter.output.activated, Some(id(2)));
    assert_eq!(
        enter
            .output
            .responses
            .iter()
            .filter(|item| item.response.keyboard_activated)
            .count(),
        1
    );

    let repeated = run_frame(
        &items,
        config(),
        &mut cursor,
        &mut selection,
        &mut memory,
        key_input(Key::Space, Modifiers::default(), true),
        false,
    );
    assert_eq!(repeated.output.activated, None);
    assert!(
        repeated
            .output
            .responses
            .iter()
            .all(|item| !item.response.keyboard_activated)
    );
}

#[test]
fn stable_ids_and_focus_repair_survive_reorder_and_removal() {
    let first = projection(&[1, 2, 3]);
    let reordered = projection(&[3, 2, 1]);
    let removed = projection(&[1, 3]);
    let mut cursor = CollectionCursor::new();
    let mut selection = Selection::new();
    let mut memory = UiMemory::new();
    let clicked = click_row(
        1,
        Modifiers::default(),
        &first,
        &mut cursor,
        &mut selection,
        &mut memory,
    );
    let stable_id = clicked.list_id.child(("virtual-list-row", 2_u64));

    let reordered_run = run_frame(
        &reordered,
        config(),
        &mut cursor,
        &mut selection,
        &mut memory,
        UiInput::default(),
        false,
    );
    assert_eq!(
        reordered_run.list_id.child(("virtual-list-row", 2_u64)),
        stable_id
    );
    assert!(memory.is_focused(stable_id));

    let removed_run = run_frame(
        &removed,
        config(),
        &mut cursor,
        &mut selection,
        &mut memory,
        UiInput::default(),
        false,
    );
    assert_eq!(cursor.active(), Some(id(3)));
    let repaired = removed_run.list_id.child(("virtual-list-row", 3_u64));
    assert!(memory.is_focused(repaired));
    assert!(!memory.is_focused(stable_id));
    assert!(removed_run.frame.semantics.get(stable_id).is_none());
}

#[test]
fn invalid_geometry_is_rejected_and_disabled_or_empty_lists_are_inert() {
    let items = projection(&[1, 2]);
    let theme = default_dark_theme();
    let mut invalid_memory = UiMemory::new();
    let ui = Ui::begin_frame(context(UiInput::default()), &mut invalid_memory, &theme);
    assert!(
        ui.prepare_virtual_list(
            "invalid",
            VirtualListConfig::new(Rect::new(f32::NAN, 0.0, 100.0, 20.0), 20.0),
            &items,
        )
        .is_none()
    );
    assert!(
        ui.prepare_virtual_list(
            "empty-bounds",
            VirtualListConfig::new(Rect::ZERO, 20.0),
            &items,
        )
        .is_none()
    );
    assert!(
        ui.prepare_virtual_list(
            "invalid-row",
            VirtualListConfig::new(BOUNDS, f32::INFINITY),
            &items,
        )
        .is_none()
    );

    let empty = CollectionProjection::empty();
    let mut cursor = CollectionCursor::new();
    let mut selection = Selection::new();
    let mut empty_memory = UiMemory::new();
    let empty_run = run_frame(
        &empty,
        config(),
        &mut cursor,
        &mut selection,
        &mut empty_memory,
        UiInput::default(),
        false,
    );
    assert!(empty_run.callbacks.is_empty());
    assert!(empty_run.output.responses.is_empty());
    assert_eq!(
        empty_run
            .frame
            .semantics
            .get(empty_run.list_id)
            .expect("empty list semantics")
            .role,
        SemanticRole::List
    );

    let mut disabled_memory = UiMemory::new();
    let disabled_config = config().disabled(true);
    let _ = run_frame(
        &items,
        disabled_config.clone(),
        &mut cursor,
        &mut selection,
        &mut disabled_memory,
        pointer_input(10.0, 10.0, true, false, Modifiers::default()),
        false,
    );
    let disabled = run_frame(
        &items,
        disabled_config,
        &mut cursor,
        &mut selection,
        &mut disabled_memory,
        pointer_input(10.0, 10.0, false, true, Modifiers::default()),
        false,
    );
    assert!(selection.selected().is_empty());
    assert!(
        disabled
            .output
            .responses
            .iter()
            .all(|item| { item.response.state.disabled && !item.response.clicked })
    );
}
