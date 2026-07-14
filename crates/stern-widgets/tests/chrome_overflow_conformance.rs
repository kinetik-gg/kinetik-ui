//! Windowless conformance for deterministic chrome overflow projection.

use stern_widgets::{ChromeOverflowItem, ChromeOverflowProjection, project_chrome_overflow};

fn item(key: &'static str, width: f32) -> ChromeOverflowItem<&'static str> {
    ChromeOverflowItem::new(key, width)
}

fn keys(projection: &ChromeOverflowProjection<&'static str>) -> Vec<&'static str> {
    projection.visible().iter().map(|item| item.key).collect()
}

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= f32::EPSILON,
        "{actual} != {expected}"
    );
}

fn assert_finite_non_overlapping(projection: &ChromeOverflowProjection<&'static str>) {
    let mut end = 0.0;
    for item in projection.visible() {
        assert!(item.x.is_finite(), "{item:?}");
        assert!(item.width.is_finite(), "{item:?}");
        assert!(item.x >= 0.0, "{item:?}");
        assert!(item.width >= 0.0, "{item:?}");
        assert!(item.x >= end, "{item:?}");
        end = item.x + item.width;
        assert!(end.is_finite(), "{item:?}");
    }

    if let Some(trigger) = projection.trigger() {
        assert!(trigger.x.is_finite(), "{trigger:?}");
        assert!(trigger.width.is_finite(), "{trigger:?}");
        assert!(trigger.x >= end, "{trigger:?}");
        assert!(trigger.width >= 0.0, "{trigger:?}");
        assert!((trigger.x + trigger.width).is_finite(), "{trigger:?}");
    }
}

#[test]
fn full_and_exact_fit_keep_every_item_without_a_trigger() {
    let full = project_chrome_overflow([item("file", 24.0), item("edit", 16.0)], 48.0, 12.0);
    let exact = project_chrome_overflow([item("file", 24.0), item("edit", 16.0)], 40.0, 12.0);

    for projection in [&full, &exact] {
        assert_eq!(keys(projection), ["file", "edit"]);
        assert!(projection.overflowed().is_empty());
        assert!(!projection.has_overflow());
        assert_eq!(projection.trigger(), None);
        assert_close(projection.visible()[0].x, 0.0);
        assert_close(projection.visible()[1].x, 24.0);
        assert_finite_non_overlapping(projection);
    }
}

#[test]
fn compact_projection_reserves_trigger_and_collapses_the_tail() {
    let projection = project_chrome_overflow(
        [item("file", 30.0), item("edit", 20.0), item("view", 10.0)],
        48.0,
        12.0,
    );

    assert_eq!(keys(&projection), ["file"]);
    assert_eq!(projection.overflowed(), ["edit", "view"]);
    assert!(projection.has_overflow());
    assert_eq!(projection.trigger().map(|trigger| trigger.x), Some(30.0));
    assert_eq!(
        projection.trigger().map(|trigger| trigger.width),
        Some(12.0)
    );
    assert_finite_non_overlapping(&projection);
}

#[test]
fn invalid_widths_sanitize_and_extremely_narrow_space_stays_finite() {
    let projection = project_chrome_overflow(
        [
            item("negative", -4.0),
            item("nan", f32::NAN),
            item("wide", 8.0),
        ],
        -1.0,
        f32::INFINITY,
    );

    assert_eq!(keys(&projection), ["negative", "nan"]);
    assert_close(projection.visible()[0].width, 0.0);
    assert_close(projection.visible()[1].width, 0.0);
    assert_eq!(projection.overflowed(), ["wide"]);
    assert_eq!(
        projection
            .trigger()
            .map(|trigger| (trigger.x, trigger.width)),
        Some((0.0, 0.0))
    );
    assert_finite_non_overlapping(&projection);

    let narrow = project_chrome_overflow([item("zero", 0.0), item("positive", 6.0)], 5.0, 20.0);
    assert_eq!(keys(&narrow), ["zero"]);
    assert_eq!(narrow.overflowed(), ["positive"]);
    assert_eq!(
        narrow.trigger().map(|trigger| (trigger.x, trigger.width)),
        Some((0.0, 5.0))
    );
    assert_finite_non_overlapping(&narrow);
}

#[test]
fn non_finite_available_width_resolves_to_zero_deterministically() {
    for available_width in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let projection = project_chrome_overflow(
            [item("zero", 0.0), item("positive", 1.0)],
            available_width,
            4.0,
        );

        assert_eq!(keys(&projection), ["zero"]);
        assert_eq!(projection.overflowed(), ["positive"]);
        assert_eq!(
            projection
                .trigger()
                .map(|trigger| (trigger.x, trigger.width)),
            Some((0.0, 0.0))
        );
        assert_finite_non_overlapping(&projection);
    }
}

#[test]
fn caller_keys_and_source_order_survive_reorder_and_removal() {
    let original = project_chrome_overflow(
        [item("a", 10.0), item("b", 10.0), item("c", 10.0)],
        25.0,
        5.0,
    );
    let reordered = project_chrome_overflow(
        [item("c", 10.0), item("a", 10.0), item("b", 10.0)],
        25.0,
        5.0,
    );
    let removed = project_chrome_overflow([item("a", 10.0), item("c", 10.0)], 25.0, 5.0);

    assert_eq!(keys(&original), ["a", "b"]);
    assert_eq!(original.overflowed(), ["c"]);
    assert_eq!(keys(&reordered), ["c", "a"]);
    assert_eq!(reordered.overflowed(), ["b"]);
    assert_eq!(keys(&removed), ["a", "c"]);
    assert!(removed.overflowed().is_empty());
    assert_eq!(removed.trigger(), None);
}

#[test]
fn hidden_items_are_excluded_before_fit_resolution() {
    let projection = project_chrome_overflow(
        [
            item("file", 20.0),
            item("hidden", 500.0).with_visible(false),
            item("edit", 20.0),
        ],
        40.0,
        12.0,
    );

    assert_eq!(keys(&projection), ["file", "edit"]);
    assert!(projection.overflowed().is_empty());
    assert_eq!(projection.trigger(), None);
    assert_finite_non_overlapping(&projection);
}

#[test]
fn first_non_fitting_item_forces_the_remaining_tail_to_overflow() {
    let projection = project_chrome_overflow(
        [item("wide", 20.0), item("zero", 0.0), item("small", 1.0)],
        10.0,
        2.0,
    );

    assert!(projection.visible().is_empty());
    assert_eq!(projection.overflowed(), ["wide", "zero", "small"]);
    assert_eq!(
        projection
            .trigger()
            .map(|trigger| (trigger.x, trigger.width)),
        Some((0.0, 2.0))
    );
    assert_finite_non_overlapping(&projection);
}

#[test]
fn huge_finite_widths_keep_the_trigger_extent_finite() {
    let projection = project_chrome_overflow(
        [
            item("prefix", f32::from_bits(0x7f19_671e)),
            item("overflow", f32::MAX),
        ],
        f32::MAX,
        f32::from_bits(0x7ecd_31c3),
    );

    assert!(projection.visible().is_empty());
    assert_eq!(projection.overflowed(), ["prefix", "overflow"]);
    assert_finite_non_overlapping(&projection);
    let trigger = projection.trigger().expect("overflow trigger");
    assert!(trigger.x + trigger.width <= f32::MAX);
}
