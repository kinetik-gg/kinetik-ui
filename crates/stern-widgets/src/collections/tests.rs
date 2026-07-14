use super::{
    GridColumns, GridLayout, ItemId, ListLayout, Selection, SortDirection, TableColumn,
    TableLayout, TableSort, TreeExpansion, TreeItem, TreeLayout, TreeModel, TreeModelError,
    VirtualRangeRequest, clamp_virtual_scroll_offset, virtual_range,
};
use stern_core::{Rect, Size};

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
    assert!(rect.width >= 0.0, "rect width must be bounded: {rect:?}");
    assert!(rect.height >= 0.0, "rect height must be bounded: {rect:?}");
}

#[test]
fn list_layout_computes_row_rectangles() {
    let rows = ListLayout::new(20.0).row_rects(Rect::new(0.0, 0.0, 100.0, 200.0), 10, 2..5);

    assert_eq!(rows.len(), 3);
    assert!((rows[0].rect.y - 40.0).abs() < f32::EPSILON);
}

#[test]
fn list_layout_exposes_scroll_extent_and_visible_rows() {
    let list = ListLayout::new(10.0);

    assert_approx(list.content_height(100), 1000.0);
    assert_approx(list.max_scroll_offset(100, 30.0), 970.0);
    assert_approx(list.clamp_scroll_offset(100, 30.0, 5000.0), 970.0);
    assert_eq!(list.visible_range(100, 5000.0, 30.0, 0), 97..100);

    let rows = list.visible_row_rects(Rect::new(0.0, 100.0, 200.0, 30.0), 100, 5000.0, 0);
    assert_eq!(rows[0].index, 97);
    assert!((rows[0].rect.y - 100.0).abs() < f32::EPSILON);
}

#[test]
fn list_layout_rejects_invalid_row_height() {
    let list = ListLayout::new(f32::NAN);

    assert_approx(list.content_height(100), 0.0);
    assert_eq!(list.visible_range(100, 0.0, 30.0, 0), 0..0);
    assert!(
        list.row_rects(Rect::new(0.0, 0.0, 200.0, 30.0), 100, 0..3)
            .is_empty()
    );
}

#[test]
fn list_layout_sanitizes_visible_rect_inputs() {
    let list = ListLayout::new(10.0);

    let direct = list.row_rect(Rect::new(f32::NAN, f32::INFINITY, f32::INFINITY, 30.0), 2);
    assert_rect_finite(direct.expect("valid row height"));

    let rows = list.visible_row_rects(
        Rect::new(f32::NAN, f32::NEG_INFINITY, f32::INFINITY, 30.0),
        8,
        f32::INFINITY,
        0,
    );
    assert_eq!(rows.len(), 4);
    assert_eq!(rows[0].index, 0);
    for row in rows {
        assert_rect_finite(row.rect);
    }
}

#[test]
fn grid_layout_supports_fixed_columns() {
    let grid = GridLayout {
        columns: GridColumns::Fixed(2),
        item_size: Size::new(10.0, 10.0),
        gap: 2.0,
    };
    let items = grid.item_rects(Rect::new(0.0, 0.0, 100.0, 100.0), 4, 0..4);

    assert!((items[2].rect.y - 12.0).abs() < f32::EPSILON);
}

#[test]
fn grid_layout_supports_adaptive_columns() {
    let grid = GridLayout {
        columns: GridColumns::Adaptive { min_width: 20.0 },
        item_size: Size::new(20.0, 20.0),
        gap: 5.0,
    };

    assert_eq!(grid.column_count(Rect::new(0.0, 0.0, 75.0, 100.0)), 3);
}

#[test]
fn grid_layout_sanitizes_bad_adaptive_inputs() {
    let grid = GridLayout {
        columns: GridColumns::Adaptive {
            min_width: f32::NAN,
        },
        item_size: Size::new(20.0, 20.0),
        gap: -5.0,
    };

    assert_eq!(grid.column_count(Rect::new(0.0, 0.0, 75.0, 100.0)), 3);

    let invalid_grid = GridLayout {
        columns: GridColumns::Fixed(2),
        item_size: Size::new(0.0, 20.0),
        gap: 4.0,
    };
    assert!(
        invalid_grid
            .item_rects(Rect::new(0.0, 0.0, 100.0, 100.0), 4, 0..4)
            .is_empty()
    );
}

#[test]
fn table_layout_computes_header_and_cell_rectangles() {
    let table = TableLayout {
        columns: vec![
            TableColumn {
                id: ItemId::from_raw(1),
                header: "Name".to_owned(),
                width: 100.0,
            },
            TableColumn {
                id: ItemId::from_raw(2),
                header: "Kind".to_owned(),
                width: 50.0,
            },
        ],
        header_height: 24.0,
        row_height: 18.0,
        sort: Some(TableSort {
            column: ItemId::from_raw(1),
            direction: SortDirection::Ascending,
        }),
    };

    assert_eq!(
        table.header_rects(Rect::new(0.0, 0.0, 200.0, 200.0)).len(),
        2
    );
    assert_eq!(
        table
            .cell_rects(Rect::new(0.0, 0.0, 200.0, 200.0), 2, 0..2)
            .len(),
        4
    );
}

#[test]
fn table_layout_exposes_content_size_scroll_and_cell_metadata() {
    let table = TableLayout {
        columns: vec![
            TableColumn {
                id: ItemId::from_raw(1),
                header: "Name".to_owned(),
                width: 100.0,
            },
            TableColumn {
                id: ItemId::from_raw(2),
                header: "State".to_owned(),
                width: f32::NAN,
            },
        ],
        header_height: 30.0,
        row_height: 20.0,
        sort: None,
    };

    assert_approx(table.total_width(), 100.0);
    let content_size = table.content_size(3);
    assert_approx(content_size.width, 100.0);
    assert_approx(content_size.height, 90.0);
    assert_approx(table.max_scroll_offset(100, 70.0), 1960.0);

    let headers = table.header_cells(Rect::new(10.0, 20.0, 200.0, 70.0));
    assert_eq!(headers[1].column_id, ItemId::from_raw(2));
    assert_approx(headers[1].rect.width, 0.0);

    let cells = table.visible_body_cells(Rect::new(10.0, 20.0, 200.0, 70.0), 100, 5000.0, 0);
    assert_eq!(cells[0].row, 98);
    assert_eq!(cells[0].column, 0);
    assert_eq!(cells[0].column_id, ItemId::from_raw(1));
    assert!((cells[0].rect.y - 50.0).abs() < f32::EPSILON);
}

#[test]
fn table_layout_rejects_invalid_row_height() {
    let table = TableLayout {
        columns: vec![TableColumn {
            id: ItemId::from_raw(1),
            header: "Name".to_owned(),
            width: 100.0,
        }],
        header_height: 24.0,
        row_height: 0.0,
        sort: None,
    };

    assert_approx(table.body_height(10), 0.0);
    assert_eq!(table.visible_row_range(10, 0.0, 100.0, 0), 0..0);
    assert!(
        table
            .body_cells(Rect::new(0.0, 0.0, 200.0, 100.0), 10, 0..3)
            .is_empty()
    );
}

#[test]
fn table_layout_sanitizes_visible_ranges_and_rects() {
    let table = TableLayout {
        columns: vec![
            TableColumn {
                id: ItemId::from_raw(1),
                header: "Name".to_owned(),
                width: 100.0,
            },
            TableColumn {
                id: ItemId::from_raw(2),
                header: "Bad".to_owned(),
                width: f32::INFINITY,
            },
        ],
        header_height: f32::NAN,
        row_height: 20.0,
        sort: None,
    };

    let headers = table.header_cells(Rect::new(f32::NAN, f32::INFINITY, 200.0, 80.0));
    assert_eq!(headers.len(), 2);
    for header in headers {
        assert_rect_finite(header.rect);
    }

    let cells = table.visible_body_cells(
        Rect::new(f32::NAN, f32::NEG_INFINITY, f32::INFINITY, 60.0),
        10,
        f32::INFINITY,
        0,
    );
    assert_eq!(cells.len(), 8);
    assert_eq!(cells[0].row, 0);
    assert_eq!(cells[0].index, 0);
    for cell in cells {
        assert_rect_finite(cell.rect);
    }
}

fn tree_model() -> TreeModel {
    TreeModel::new(vec![
        TreeItem {
            id: ItemId::from_raw(1),
            parent: None,
            has_children: false,
        },
        TreeItem {
            id: ItemId::from_raw(2),
            parent: Some(ItemId::from_raw(1)),
            has_children: false,
        },
        TreeItem {
            id: ItemId::from_raw(3),
            parent: Some(ItemId::from_raw(2)),
            has_children: false,
        },
        TreeItem {
            id: ItemId::from_raw(4),
            parent: None,
            has_children: true,
        },
    ])
}

#[test]
fn tree_model_validates_structure() {
    assert!(tree_model().validate().is_ok());

    let duplicate = TreeModel::new(vec![
        TreeItem {
            id: ItemId::from_raw(1),
            parent: None,
            has_children: false,
        },
        TreeItem {
            id: ItemId::from_raw(1),
            parent: None,
            has_children: false,
        },
    ]);
    assert_eq!(
        duplicate.validate(),
        Err(TreeModelError::DuplicateItemId {
            id: ItemId::from_raw(1)
        })
    );

    let unknown = TreeModel::new(vec![TreeItem {
        id: ItemId::from_raw(2),
        parent: Some(ItemId::from_raw(99)),
        has_children: false,
    }]);
    assert_eq!(
        unknown.validate(),
        Err(TreeModelError::UnknownParent {
            id: ItemId::from_raw(2),
            parent: ItemId::from_raw(99),
        })
    );

    let self_parent = TreeModel::new(vec![TreeItem {
        id: ItemId::from_raw(7),
        parent: Some(ItemId::from_raw(7)),
        has_children: false,
    }]);
    assert_eq!(
        self_parent.validate(),
        Err(TreeModelError::SelfParent {
            id: ItemId::from_raw(7)
        })
    );

    let cycle = TreeModel::new(vec![
        TreeItem {
            id: ItemId::from_raw(1),
            parent: Some(ItemId::from_raw(2)),
            has_children: false,
        },
        TreeItem {
            id: ItemId::from_raw(2),
            parent: Some(ItemId::from_raw(1)),
            has_children: false,
        },
    ]);
    assert_eq!(
        cycle.validate(),
        Err(TreeModelError::Cycle {
            id: ItemId::from_raw(1)
        })
    );
}

#[test]
fn invalid_tree_models_have_empty_visible_rows() {
    let invalid = TreeModel::new(vec![TreeItem {
        id: ItemId::from_raw(1),
        parent: Some(ItemId::from_raw(99)),
        has_children: false,
    }]);

    assert!(invalid.visible_rows(&TreeExpansion::new()).is_empty());
    assert_eq!(invalid.visible_row_count(&TreeExpansion::new()), 0);
    assert!(
        invalid
            .visible_rows_in_range(&TreeExpansion::new(), 0..1)
            .is_empty()
    );
}

#[test]
fn tree_model_flattens_visible_rows_by_expansion() {
    let tree = tree_model();
    let mut expansion = TreeExpansion::new();

    let rows = tree.visible_rows(&expansion);
    assert_eq!(
        rows.iter().map(|row| row.id).collect::<Vec<_>>(),
        vec![ItemId::from_raw(1), ItemId::from_raw(4)]
    );
    assert!(rows[0].has_children);
    assert!(!rows[0].expanded);
    assert!(rows[1].has_children);

    assert!(expansion.expand(ItemId::from_raw(1)));
    let rows = tree.visible_rows(&expansion);
    assert_eq!(
        rows.iter()
            .map(|row| (row.id, row.depth))
            .collect::<Vec<_>>(),
        vec![
            (ItemId::from_raw(1), 0),
            (ItemId::from_raw(2), 1),
            (ItemId::from_raw(4), 0),
        ]
    );

    assert!(expansion.expand(ItemId::from_raw(2)));
    let rows = tree.visible_rows(&expansion);
    assert_eq!(
        rows.iter()
            .map(|row| (row.id, row.depth))
            .collect::<Vec<_>>(),
        vec![
            (ItemId::from_raw(1), 0),
            (ItemId::from_raw(2), 1),
            (ItemId::from_raw(3), 2),
            (ItemId::from_raw(4), 0),
        ]
    );
}

#[test]
fn tree_model_collects_visible_rows_in_range_with_global_indices() {
    let tree = tree_model();
    let mut expansion = TreeExpansion::new();
    expansion.expand(ItemId::from_raw(1));
    expansion.expand(ItemId::from_raw(2));

    assert_eq!(tree.visible_row_count(&expansion), 4);
    let rows = tree.visible_rows_in_range(&expansion, 1..3);

    assert_eq!(
        rows.iter()
            .map(|row| (row.row, row.id, row.depth))
            .collect::<Vec<_>>(),
        vec![(1, ItemId::from_raw(2), 1), (2, ItemId::from_raw(3), 2)]
    );
}

#[test]
fn tree_expansion_collapses_descendants() {
    let tree = tree_model();
    let mut expansion = TreeExpansion::new();
    expansion.expand(ItemId::from_raw(1));
    expansion.expand(ItemId::from_raw(2));

    assert_eq!(
        tree.descendant_ids(ItemId::from_raw(1)),
        vec![ItemId::from_raw(2), ItemId::from_raw(3)]
    );
    assert!(expansion.collapse_descendants(&tree, ItemId::from_raw(1)));
    assert!(expansion.expanded().is_empty());
}

#[test]
fn tree_expansion_toggle_clear_and_visible_rows_are_deterministic() {
    let tree = tree_model();
    let mut expansion = TreeExpansion::new();

    assert!(expansion.toggle(ItemId::from_raw(2)));
    assert_eq!(expansion.expanded(), vec![ItemId::from_raw(2)]);
    assert!(
        tree.visible_rows(&expansion)
            .iter()
            .all(|row| row.id != ItemId::from_raw(3))
    );

    assert!(expansion.toggle(ItemId::from_raw(1)));
    let rows = tree.visible_rows(&expansion);
    assert_eq!(
        rows.iter().map(|row| row.id).collect::<Vec<_>>(),
        vec![
            ItemId::from_raw(1),
            ItemId::from_raw(2),
            ItemId::from_raw(3),
            ItemId::from_raw(4)
        ]
    );

    assert!(!expansion.toggle(ItemId::from_raw(2)));
    let rows = tree.visible_rows(&expansion);
    assert_eq!(
        rows.iter().map(|row| row.id).collect::<Vec<_>>(),
        vec![
            ItemId::from_raw(1),
            ItemId::from_raw(2),
            ItemId::from_raw(4)
        ]
    );

    expansion.clear();
    assert!(expansion.expanded().is_empty());
    assert_eq!(
        tree.visible_rows(&expansion)
            .iter()
            .map(|row| row.id)
            .collect::<Vec<_>>(),
        vec![ItemId::from_raw(1), ItemId::from_raw(4)]
    );
}

#[test]
fn tree_layout_virtualizes_indented_visible_rows() {
    let tree = tree_model();
    let mut expansion = TreeExpansion::new();
    expansion.expand(ItemId::from_raw(1));
    expansion.expand(ItemId::from_raw(2));
    let rows = tree.visible_rows(&expansion);
    let layout = TreeLayout::new(20.0, 12.0);

    assert_approx(layout.content_height(rows.len()), 80.0);
    assert_approx(layout.max_scroll_offset(rows.len(), 40.0), 40.0);
    assert_approx(layout.clamp_scroll_offset(rows.len(), 40.0, 500.0), 40.0);
    assert_eq!(layout.visible_range(rows.len(), 20.0, 40.0, 0), 1..4);

    let rects = layout.visible_row_rects(Rect::new(10.0, 100.0, 200.0, 40.0), &rows, 20.0, 0);
    assert_eq!(rects.len(), 3);
    assert_eq!(rects[0].row.id, ItemId::from_raw(2));
    assert_approx(rects[0].rect.y, 100.0);
    assert_approx(rects[0].content_rect.x, 22.0);
    assert_approx(rects[1].content_rect.x, 34.0);
}

#[test]
fn tree_layout_rejects_invalid_row_height_and_sanitizes_indent() {
    let layout = TreeLayout::new(f32::NAN, f32::NAN);
    let rows = tree_model().visible_rows(&TreeExpansion::new());

    assert_approx(layout.content_height(rows.len()), 0.0);
    assert_eq!(layout.visible_range(rows.len(), 0.0, 100.0, 0), 0..0);
    assert!(
        layout
            .visible_row_rects(Rect::new(0.0, 0.0, 100.0, 100.0), &rows, 0.0, 0)
            .is_empty()
    );

    let layout = TreeLayout::new(20.0, -12.0);
    let rects = layout.visible_row_rects(Rect::new(10.0, 20.0, 100.0, 40.0), &rows, 0.0, 0);
    assert_eq!(rects[0].rect, rects[0].content_rect);
}

#[test]
fn tree_layout_sanitizes_visible_row_rects() {
    let tree = tree_model();
    let mut expansion = TreeExpansion::new();
    expansion.expand(ItemId::from_raw(1));
    let rows = tree.visible_rows(&expansion);
    let layout = TreeLayout::new(20.0, 12.0);

    let rects = layout.visible_row_rects(
        Rect::new(f32::NAN, f32::INFINITY, f32::INFINITY, 40.0),
        &rows,
        f32::NEG_INFINITY,
        usize::MAX,
    );
    assert_eq!(rects.len(), rows.len());
    for rect in rects {
        assert_rect_finite(rect.rect);
        assert_rect_finite(rect.content_rect);
    }
}

#[test]
fn tree_layout_positions_range_rows_by_global_row_index() {
    let tree = tree_model();
    let mut expansion = TreeExpansion::new();
    expansion.expand(ItemId::from_raw(1));
    expansion.expand(ItemId::from_raw(2));
    let total_rows = tree.visible_row_count(&expansion);
    let rows = tree.visible_rows_in_range(&expansion, 1..3);
    let layout = TreeLayout::new(20.0, 12.0);

    let rects = layout.visible_row_rects_in_range(
        Rect::new(10.0, 100.0, 200.0, 40.0),
        total_rows,
        &rows,
        20.0,
        0,
    );

    assert_eq!(rects.len(), 2);
    assert_eq!(rects[0].row.row, 1);
    assert_approx(rects[0].rect.y, 100.0);
    assert_approx(rects[1].rect.y, 120.0);
    assert_approx(rects[1].content_rect.x, 34.0);
}

#[test]
fn virtual_range_applies_overscan_and_bounds() {
    let range = virtual_range(VirtualRangeRequest {
        item_count: 100,
        scroll_offset: 50.0,
        viewport_extent: 40.0,
        item_extent: 10.0,
        overscan: 2,
    });

    assert_eq!(range, 3..12);
}

#[test]
fn virtual_range_clamps_overscrolled_offsets() {
    let range = virtual_range(VirtualRangeRequest {
        item_count: 100,
        scroll_offset: 5000.0,
        viewport_extent: 40.0,
        item_extent: 10.0,
        overscan: 0,
    });

    assert_eq!(range, 96..100);
    assert_approx(clamp_virtual_scroll_offset(5000.0, 100, 10.0, 40.0), 960.0);
}

#[test]
fn virtual_range_clamps_negative_and_extreme_overscan() {
    assert_eq!(
        virtual_range(VirtualRangeRequest {
            item_count: 6,
            scroll_offset: -200.0,
            viewport_extent: 20.0,
            item_extent: 10.0,
            overscan: usize::MAX,
        }),
        0..6
    );
    assert_eq!(
        virtual_range(VirtualRangeRequest {
            item_count: 6,
            scroll_offset: f32::INFINITY,
            viewport_extent: 20.0,
            item_extent: 10.0,
            overscan: 0,
        }),
        0..3
    );
}

#[test]
fn virtual_range_handles_empty_inputs() {
    assert_eq!(
        virtual_range(VirtualRangeRequest {
            item_count: 0,
            scroll_offset: 0.0,
            viewport_extent: 100.0,
            item_extent: 20.0,
            overscan: 1,
        }),
        0..0
    );
    assert_eq!(
        virtual_range(VirtualRangeRequest {
            item_count: 10,
            scroll_offset: 0.0,
            viewport_extent: f32::NAN,
            item_extent: 20.0,
            overscan: 1,
        }),
        0..0
    );
    assert_eq!(
        virtual_range(VirtualRangeRequest {
            item_count: 10,
            scroll_offset: 0.0,
            viewport_extent: 100.0,
            item_extent: f32::NAN,
            overscan: 1,
        }),
        0..0
    );
}

mod selection;
