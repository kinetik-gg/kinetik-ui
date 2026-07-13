//! Stable collection cursor navigation and reconciliation conformance tests.

use kinetik_ui_widgets::{
    CollectionCursor, CollectionCursorMove, CollectionProjection, ItemId, Selection, TreeExpansion,
    TreeItem, TreeModel,
};

fn id(raw: u64) -> ItemId {
    ItemId::from_raw(raw)
}

fn projection(raw_ids: &[u64]) -> CollectionProjection {
    CollectionProjection::from_source_ids(&raw_ids.iter().copied().map(id).collect::<Vec<_>>())
}

#[test]
fn empty_cursor_navigation_chooses_directional_boundary_and_never_wraps() {
    let items = projection(&[10, 20, 30]);
    let mut cursor = CollectionCursor::new();

    let first = cursor
        .navigate(&items, CollectionCursorMove::Next)
        .expect("first target");
    assert_eq!((first.id, first.projected_index), (id(10), 0));
    assert_eq!(
        cursor.navigate(&items, CollectionCursorMove::Previous),
        Some(first)
    );

    assert_eq!(
        cursor.navigate(&items, CollectionCursorMove::Last),
        Some(kinetik_ui_widgets::CollectionCursorTarget {
            id: id(30),
            projected_index: 2,
        })
    );
    assert_eq!(
        cursor.navigate(&items, CollectionCursorMove::Next),
        Some(kinetik_ui_widgets::CollectionCursorTarget {
            id: id(30),
            projected_index: 2,
        })
    );

    cursor.clear();
    let last = cursor
        .navigate(&items, CollectionCursorMove::Previous)
        .expect("last target");
    assert_eq!((last.id, last.projected_index), (id(30), 2));
    assert_eq!(
        cursor.navigate(&items, CollectionCursorMove::First),
        Some(first)
    );
}

#[test]
fn page_navigation_uses_caller_supplied_row_stride_and_is_bounded() {
    let items = projection(&(0..10).collect::<Vec<_>>());
    let mut cursor = CollectionCursor::new();

    assert_eq!(
        cursor
            .navigate(&items, CollectionCursorMove::PageNext { rows: 4 })
            .expect("initial target")
            .projected_index,
        0
    );
    assert_eq!(
        cursor
            .navigate(&items, CollectionCursorMove::PageNext { rows: 4 })
            .expect("page target")
            .projected_index,
        4
    );
    assert_eq!(
        cursor
            .navigate(&items, CollectionCursorMove::PageNext { rows: 99 })
            .expect("bounded end")
            .projected_index,
        9
    );
    assert_eq!(
        cursor
            .navigate(&items, CollectionCursorMove::PagePrevious { rows: 3 })
            .expect("previous page")
            .projected_index,
        6
    );
    assert_eq!(
        cursor
            .navigate(&items, CollectionCursorMove::PagePrevious { rows: 99 })
            .expect("bounded start")
            .projected_index,
        0
    );
}

#[test]
fn reconciliation_preserves_active_identity_across_reorder() {
    let first = projection(&[1, 2, 3]);
    let reordered = projection(&[3, 1, 2]);
    let mut cursor = CollectionCursor::new();

    cursor.activate(&first, id(2)).expect("activate item");
    let target = cursor.reconcile(&reordered).expect("reconciled target");

    assert_eq!(target.id, id(2));
    assert_eq!(target.projected_index, 2);
    assert_eq!(cursor.active(), Some(id(2)));
    assert_eq!(cursor.last_projected_index(), Some(2));
}

#[test]
fn reconciliation_repairs_removed_item_to_same_slot_then_preceding_tail() {
    let first = projection(&[1, 2, 3, 4]);
    let same_slot = projection(&[1, 3, 4]);
    let short_tail = projection(&[1]);
    let mut cursor = CollectionCursor::new();

    cursor.activate(&first, id(2)).expect("activate item");
    let repaired = cursor.reconcile(&same_slot).expect("same slot repair");
    assert_eq!((repaired.id, repaired.projected_index), (id(3), 1));

    cursor.activate(&first, id(4)).expect("activate tail");
    let repaired = cursor.reconcile(&short_tail).expect("tail repair");
    assert_eq!((repaired.id, repaired.projected_index), (id(1), 0));
}

#[test]
fn empty_projection_clears_retained_cursor() {
    let items = projection(&[1]);
    let mut cursor = CollectionCursor::new();
    cursor.activate(&items, id(1)).expect("activate item");

    assert_eq!(cursor.reconcile(&CollectionProjection::empty()), None);
    assert_eq!(cursor.active(), None);
    assert_eq!(cursor.last_projected_index(), None);
}

#[test]
fn ten_thousand_row_navigation_remains_exact() {
    let raw_ids = (0..10_000).collect::<Vec<_>>();
    let items = projection(&raw_ids);
    let mut cursor = CollectionCursor::new();

    cursor
        .navigate(&items, CollectionCursorMove::First)
        .expect("first target");
    for _ in 0..999 {
        cursor
            .navigate(&items, CollectionCursorMove::PageNext { rows: 10 })
            .expect("page target");
    }

    assert_eq!(cursor.active(), Some(id(9_990)));
    assert_eq!(cursor.last_projected_index(), Some(9_990));
    let end = cursor
        .navigate(&items, CollectionCursorMove::PageNext { rows: 10 })
        .expect("bounded end");
    assert_eq!((end.id, end.projected_index), (id(9_999), 9_999));
}

#[test]
fn navigation_target_composes_with_existing_range_selection() {
    let items = projection(&[1, 2, 3, 4]);
    let visible_ids = items.visible_ids();
    let mut cursor = CollectionCursor::new();
    let mut selection = Selection::new();

    let anchor = cursor.activate(&items, id(2)).expect("anchor target");
    selection.replace(anchor.id);
    let end = cursor
        .navigate(&items, CollectionCursorMove::PageNext { rows: 2 })
        .expect("range end");

    assert!(selection.select_range(&visible_ids, end.id));
    assert_eq!(selection.selected(), vec![id(2), id(3), id(4)]);
    assert_eq!(selection.active, Some(id(4)));
}

#[test]
fn flattened_tree_ids_use_the_same_vertical_cursor_contract() {
    let tree = TreeModel::new(vec![
        TreeItem {
            id: id(1),
            parent: None,
            has_children: true,
        },
        TreeItem {
            id: id(2),
            parent: Some(id(1)),
            has_children: false,
        },
        TreeItem {
            id: id(3),
            parent: None,
            has_children: false,
        },
    ]);
    let mut expansion = TreeExpansion::new();
    expansion.expand(id(1));
    let visible = tree.visible_item_ids(&expansion);
    let items = CollectionProjection::from_source_ids(&visible);
    let mut cursor = CollectionCursor::new();

    assert_eq!(
        cursor
            .navigate(&items, CollectionCursorMove::Next)
            .expect("root")
            .id,
        id(1)
    );
    assert_eq!(
        cursor
            .navigate(&items, CollectionCursorMove::Next)
            .expect("child")
            .id,
        id(2)
    );
    assert_eq!(
        cursor
            .navigate(&items, CollectionCursorMove::Next)
            .expect("next root")
            .id,
        id(3)
    );
}
