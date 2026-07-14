use super::{ItemId, Selection};

#[test]
fn selection_supports_replace_toggle_clear() {
    let mut selection = Selection::new();
    let one = ItemId::from_raw(1);
    let two = ItemId::from_raw(2);

    selection.replace(one);
    assert!(selection.contains(one));
    assert_eq!(selection.active, Some(one));
    assert_eq!(selection.selected(), vec![one]);
    selection.toggle(two);
    assert_eq!(selection.selected(), vec![one, two]);
    assert_eq!(selection.active, Some(two));
    selection.toggle(one);
    assert!(!selection.contains(one));
    selection.clear();
    assert!(selection.selected().is_empty());
    assert_eq!(selection.active, None);
}

#[test]
fn selection_supports_ranges_from_anchor() {
    let items = [
        ItemId::from_raw(1),
        ItemId::from_raw(2),
        ItemId::from_raw(3),
        ItemId::from_raw(4),
    ];
    let mut selection = Selection::new();

    selection.replace(ItemId::from_raw(2));
    assert!(selection.select_range(&items, ItemId::from_raw(4)));

    assert_eq!(
        selection.selected(),
        vec![
            ItemId::from_raw(2),
            ItemId::from_raw(3),
            ItemId::from_raw(4)
        ]
    );
}

#[test]
fn selection_range_failure_preserves_deterministic_state() {
    let items = [
        ItemId::from_raw(5),
        ItemId::from_raw(3),
        ItemId::from_raw(9),
        ItemId::from_raw(1),
    ];
    let mut selection = Selection::new();

    selection.replace(ItemId::from_raw(3));
    selection.toggle(ItemId::from_raw(1));
    assert_eq!(
        selection.selected(),
        vec![ItemId::from_raw(1), ItemId::from_raw(3)]
    );
    assert!(!selection.select_range(&items, ItemId::from_raw(99)));
    assert_eq!(selection.active, Some(ItemId::from_raw(1)));
    assert_eq!(
        selection.selected(),
        vec![ItemId::from_raw(1), ItemId::from_raw(3)]
    );
}
