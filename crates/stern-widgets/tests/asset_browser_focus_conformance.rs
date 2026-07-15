//! Asset-browser inward focus ownership and composition conformance tests.

#![allow(clippy::float_cmp)]

use std::time::Duration;

use stern_core::{
    ComponentState, FrameContext, ImageId, PathElement, PhysicalSize, Point, PointerButtonState,
    PointerInput, PointerOrder, Primitive, Rect, ScaleFactor, SemanticNode, Size, TimeInfo,
    UiInput, UiMemory, Vec2, ViewportInfo, WidgetId, default_dark_theme,
};
use stern_widgets::asset_browser::{
    AssetBrowserConfig, AssetBrowserItem, AssetBrowserItemRect, AssetBrowserLayout,
    AssetBrowserModel, AssetBrowserOutput, AssetBrowserSort, AssetBrowserSortKey,
    AssetBrowserState, AssetBrowserViewMode, AssetIconFallback,
};
use stern_widgets::{GridColumns, GridLayout, ItemId, ListLayout, SortDirection, Ui};

const BOUNDS: Rect = Rect::new(10.25, 20.5, 240.0, 112.0);

fn id(raw: u64) -> ItemId {
    ItemId::from_raw(raw)
}

fn asset(raw: u64, name: impl Into<String>, kind: impl Into<String>) -> AssetBrowserItem {
    AssetBrowserItem::new(id(raw), name, kind)
}

fn layout(view_mode: AssetBrowserViewMode) -> AssetBrowserLayout {
    AssetBrowserLayout::new(
        view_mode,
        GridLayout {
            columns: GridColumns::Fixed(3),
            item_size: Size::new(72.0, 72.0),
            gap: 4.0,
        },
        ListLayout::new(28.0),
    )
    .with_overscan(1)
}

fn config(view_mode: AssetBrowserViewMode) -> AssetBrowserConfig {
    AssetBrowserConfig::new(BOUNDS, layout(view_mode))
        .label("Project assets")
        .selection_mode(stern_widgets::asset_browser::AssetBrowserSelectionMode::Multiple)
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

fn pointer_input(point: Point, down: bool, pressed: bool, released: bool) -> UiInput {
    UiInput {
        pointer: PointerInput {
            position: Some(point),
            primary: PointerButtonState::new(down, pressed, released),
            click_count: u8::from(released),
            ..PointerInput::default()
        },
        ..UiInput::default()
    }
}

#[derive(Debug)]
struct Run {
    root: WidgetId,
    items: Vec<AssetBrowserItemRect>,
    projected: Vec<ItemId>,
    output: AssetBrowserOutput,
    frame: stern_core::FrameOutput,
}

fn run_frame(
    model: &AssetBrowserModel,
    config: AssetBrowserConfig,
    state: &mut AssetBrowserState,
    memory: &mut UiMemory,
    input: UiInput,
) -> Run {
    let theme = default_dark_theme();
    let mut ui = Ui::begin_frame(context(input), memory, &theme);
    let scene = ui
        .prepare_asset_browser("focus-assets", config, model, state)
        .expect("valid asset browser scene");
    let root = scene.widget_id();
    let items = scene.layout().items.clone();
    let projected = scene.projection().visible_ids();
    ui.resolve_pointer_targets(|plan| {
        scene.declare_pointer_targets(plan, PointerOrder::new(100), state);
    })
    .expect("valid shared pointer plan");
    let output = ui.asset_browser(&scene, state, |_target, _draft| None, |_target| Vec::new());
    let frame = ui.finish_output();
    Run {
        root,
        items,
        projected,
        output,
        frame,
    }
}

fn click(
    point: Point,
    model: &AssetBrowserModel,
    config: AssetBrowserConfig,
    state: &mut AssetBrowserState,
    memory: &mut UiMemory,
) -> Run {
    let _ = run_frame(
        model,
        config.clone(),
        state,
        memory,
        pointer_input(point, true, true, false),
    );
    run_frame(
        model,
        config,
        state,
        memory,
        pointer_input(point, false, false, true),
    )
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

fn item_response(run: &Run, target: ItemId) -> stern_core::Response {
    run.output
        .responses
        .iter()
        .find(|response| response.item == target)
        .unwrap_or_else(|| panic!("missing response for item {}", target.raw()))
        .response
}

fn item_rect(run: &Run, target: ItemId) -> &AssetBrowserItemRect {
    run.items
        .iter()
        .find(|item| item.item.id == target)
        .unwrap_or_else(|| panic!("missing geometry for item {}", target.raw()))
}

fn assert_item_focus(run: &Run, target: ItemId) -> [Primitive; 2] {
    let theme = default_dark_theme();
    let item = item_rect(run, target);
    let response = item_response(run, target);
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
    let base_index = run
        .frame
        .primitives
        .iter()
        .position(|primitive| matches!(primitive, Primitive::Rect(base) if base.rect == item.rect))
        .expect("asset item base");
    let Primitive::Rect(base) = &run.frame.primitives[base_index] else {
        unreachable!()
    };
    assert_eq!(base.fill, Some(recipe.background));
    assert_eq!(base.stroke, Some(recipe.border));
    assert_eq!(base.radius, recipe.radius);

    let expected = theme
        .focus_ring(true)
        .expect("focus recipe")
        .inward_annulus_primitives(item.rect, recipe.radius, recipe.border.width);
    assert_eq!(run.frame.primitives[base_index + 1], expected[0]);
    assert_eq!(run.frame.primitives[base_index + 2], expected[1]);
    for primitive in &run.frame.primitives[base_index + 1..=base_index + 2] {
        let Primitive::Path(path) = primitive else {
            panic!("asset focus must remain a compound path");
        };
        assert_eq!(path.elements.len(), 20);
        assert_eq!(path.stroke, None);
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
        assert!(item.rect.contains_rect(bounds));
    }

    assert!(matches!(
        run.frame.primitives[base_index + 3],
        Primitive::Rect(preview) if preview.rect == item.preview_rect
    ));
    let content_index = base_index + 4;
    if item.item.thumbnail.is_some() {
        assert!(matches!(
            run.frame.primitives[content_index],
            Primitive::Image(image) if image.rect == item.preview_rect
        ));
    } else {
        assert!(matches!(
            run.frame.primitives[content_index],
            Primitive::Text(ref text) if text.text == item.item.fallback.label
        ));
    }
    assert!(matches!(
        run.frame.primitives[content_index + 1],
        Primitive::Text(ref text) if text.text == item.item.name
    ));
    assert!(matches!(
        run.frame.primitives[content_index + 2],
        Primitive::Text(ref text) if text.text == item.item.kind
    ));

    [
        run.frame.primitives[base_index + 1].clone(),
        run.frame.primitives[base_index + 2].clone(),
    ]
}

fn primitives_without_focus_paths(run: &Run) -> Vec<Primitive> {
    run.frame
        .primitives
        .iter()
        .filter(|primitive| !matches!(primitive, Primitive::Path(_)))
        .cloned()
        .collect()
}

fn output_without_focus(mut output: AssetBrowserOutput) -> AssetBrowserOutput {
    for response in &mut output.responses {
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
    assert_eq!(focused.items, unfocused.items);
    assert_eq!(focused.projected, unfocused.projected);
    assert_eq!(focused.frame.repaint, unfocused.frame.repaint);
    assert_eq!(
        output_without_focus(focused.output.clone()),
        unfocused.output
    );
    assert_eq!(
        primitives_without_focus_paths(focused),
        unfocused.frame.primitives
    );
    assert_eq!(
        semantics_without_focus(focused),
        unfocused.frame.semantics.nodes()
    );
    assert_eq!(
        focused
            .frame
            .primitives
            .iter()
            .filter(|primitive| matches!(primitive, Primitive::Path(_)))
            .count(),
        2
    );
    assert!(
        focused
            .frame
            .primitives
            .iter()
            .all(|primitive| !matches!(primitive, Primitive::TransformBegin { .. }))
    );
}

#[test]
fn grid_and_list_thumbnail_fallback_selected_pairs_add_only_exact_owned_annuli() {
    let model = AssetBrowserModel::new(vec![
        asset(1, "Thumbnail", "image").with_thumbnail(ImageId::from_raw(77)),
        asset(2, "Fallback", "material").with_fallback(AssetIconFallback::new("material", "MAT")),
    ]);

    for view_mode in [AssetBrowserViewMode::Grid, AssetBrowserViewMode::List] {
        let seed = run_frame(
            &model,
            config(view_mode),
            &mut AssetBrowserState::new(),
            &mut UiMemory::new(),
            UiInput::default(),
        );
        for target in [id(1), id(2)] {
            for selected in [false, true] {
                let mut unfocused_state = AssetBrowserState::new();
                if selected {
                    unfocused_state.selection.replace(target);
                }
                let unfocused = run_frame(
                    &model,
                    config(view_mode),
                    &mut unfocused_state,
                    &mut UiMemory::new(),
                    UiInput::default(),
                );

                let mut focused_state = AssetBrowserState::new();
                if selected {
                    focused_state.selection.replace(target);
                }
                let mut focused_memory = UiMemory::new();
                focused_memory.focus(seed.root.child(("asset-browser-item", target.raw())));
                let focused = run_frame(
                    &model,
                    config(view_mode),
                    &mut focused_state,
                    &mut focused_memory,
                    UiInput::default(),
                );

                assert_focus_only_transition(&focused, &unfocused);
                assert_eq!(item_response(&focused, target).state.selected, selected);
                assert_item_focus(&focused, target);
                let semantic = focused
                    .frame
                    .semantics
                    .get(seed.root.child(("asset-browser-item", target.raw())))
                    .expect("focused asset semantic");
                assert!(semantic.state.focused);
                assert_eq!(semantic.state.selected, selected);
                assert_eq!(semantic.bounds, item_rect(&focused, target).rect);
            }
        }
    }
}

#[test]
fn hover_press_selection_and_focus_combinations_preserve_the_exact_annuli() {
    let model = AssetBrowserModel::new(vec![asset(1, "State target", "mesh")]);

    for view_mode in [AssetBrowserViewMode::Grid, AssetBrowserViewMode::List] {
        let seed = run_frame(
            &model,
            config(view_mode),
            &mut AssetBrowserState::new(),
            &mut UiMemory::new(),
            UiInput::default(),
        );
        let point = seed.items[0].rect.center();
        let target_widget = seed.root.child(("asset-browser-item", 1_u64));
        let mut baseline = None;
        for (hovered, pressed, selected) in [
            (false, false, false),
            (true, false, false),
            (false, true, false),
            (false, false, true),
            (true, false, true),
            (false, true, true),
        ] {
            let input = if pressed {
                pointer_input(point, true, true, false)
            } else if hovered {
                pointer_input(point, false, false, false)
            } else {
                UiInput::default()
            };
            let mut unfocused_state = AssetBrowserState::new();
            if selected {
                unfocused_state.selection.replace(id(1));
            }
            let unfocused = run_frame(
                &model,
                config(view_mode),
                &mut unfocused_state,
                &mut UiMemory::new(),
                input.clone(),
            );

            let mut focused_state = AssetBrowserState::new();
            if selected {
                focused_state.selection.replace(id(1));
            }
            let mut focused_memory = UiMemory::new();
            focused_memory.focus(target_widget);
            let focused = run_frame(
                &model,
                config(view_mode),
                &mut focused_state,
                &mut focused_memory,
                input,
            );

            assert_focus_only_transition(&focused, &unfocused);
            let response = item_response(&focused, id(1));
            assert_eq!(response.state.hovered, hovered || pressed);
            assert_eq!(response.state.pressed, pressed);
            assert_eq!(response.state.selected, selected);
            let annuli = assert_item_focus(&focused, id(1));
            if let Some(baseline) = &baseline {
                assert_eq!(&annuli, baseline);
            } else {
                baseline = Some(annuli);
            }
        }
    }
}

#[test]
fn stable_id_focus_cursor_and_selection_survive_filter_sort_reorder_scroll_and_view_changes() {
    let original = AssetBrowserModel::new(vec![
        asset(1, "Zeta", "scene"),
        asset(2, "Alpha", "mesh"),
        asset(3, "Beta", "material"),
        asset(4, "Gamma", "image"),
        asset(5, "Delta", "mesh"),
        asset(6, "Epsilon", "audio"),
    ]);
    let sorted = config(AssetBrowserViewMode::List).sort(Some(AssetBrowserSort::new(
        AssetBrowserSortKey::Name,
        SortDirection::Ascending,
    )));
    let mut state = AssetBrowserState::new();
    let mut memory = UiMemory::new();
    let seed = run_frame(
        &original,
        sorted.clone(),
        &mut state,
        &mut memory,
        UiInput::default(),
    );
    assert_eq!(
        seed.projected,
        vec![id(2), id(3), id(5), id(6), id(4), id(1)]
    );
    let target_point = item_rect(&seed, id(3)).rect.center();
    let selected = click(target_point, &original, sorted, &mut state, &mut memory);
    let target_widget = selected.root.child(("asset-browser-item", 3_u64));
    assert_eq!(state.cursor.active(), Some(id(3)));
    assert_eq!(state.selection.selected(), vec![id(3)]);
    assert!(memory.is_focused(target_widget));
    assert_item_focus(&selected, id(3));

    let filtered_grid = run_frame(
        &original,
        config(AssetBrowserViewMode::Grid)
            .query("a")
            .sort(Some(AssetBrowserSort::new(
                AssetBrowserSortKey::Name,
                SortDirection::Descending,
            ))),
        &mut state,
        &mut memory,
        UiInput::default(),
    );
    assert_eq!(
        filtered_grid.projected,
        vec![id(1), id(4), id(6), id(5), id(3), id(2)]
    );
    assert_eq!(state.cursor.active(), Some(id(3)));
    assert_eq!(state.selection.selected(), vec![id(3)]);
    assert!(memory.is_focused(target_widget));
    assert!(item_response(&filtered_grid, id(3)).state.selected);
    assert_item_focus(&filtered_grid, id(3));

    let reordered = AssetBrowserModel::new(vec![
        asset(6, "Epsilon", "audio"),
        asset(5, "Delta", "mesh"),
        asset(4, "Gamma", "image"),
        asset(3, "Beta", "material"),
        asset(2, "Alpha", "mesh"),
        asset(1, "Zeta", "scene"),
    ]);
    memory.set_scroll_offset(selected.root, Vec2::new(0.0, 28.5));
    let scrolled_list = run_frame(
        &reordered,
        config(AssetBrowserViewMode::List),
        &mut state,
        &mut memory,
        UiInput::default(),
    );
    let target_rect = item_rect(&scrolled_list, id(3)).rect;
    assert_eq!(target_rect.y.to_bits(), 76.0_f32.to_bits());
    assert_eq!(state.cursor.active(), Some(id(3)));
    assert_eq!(state.selection.selected(), vec![id(3)]);
    assert!(memory.is_focused(target_widget));
    assert!(item_response(&scrolled_list, id(3)).state.selected);
    assert_item_focus(&scrolled_list, id(3));
    assert_eq!(scrolled_list.output.visible_range, 1..6);
    assert_eq!(scrolled_list.output.materialized_range, 0..6);
}
