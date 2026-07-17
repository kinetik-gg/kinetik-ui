//! Public scroll-area extent, clamping, staging, and ownership conformance.

use stern_core::{Rect, Size, UiInput, UiMemory, Vec2, WidgetId, default_dark_theme};
use stern_widgets::Ui;

#[test]
fn scroll_area_exposes_exact_viewport_extent_offset_and_maximum() {
    let viewport = Rect::new(10.0, 20.0, 80.0, 50.0);
    let content = Size::new(200.0, 140.0);
    let retained = Vec2::new(35.0, 25.0);
    let area_id = WidgetId::from_key("root").child("extent-area");
    let input = UiInput::default();
    let mut memory = UiMemory::new();
    memory.set_scroll_offset(area_id, retained);
    let theme = default_dark_theme();
    let mut ui = Ui::new(&input, &mut memory, &theme);

    let area = ui.scroll_area("extent-area", viewport, content, false, |_ui, offset| {
        (offset, String::from("logical extent retained"), 7_usize)
    });
    let frame = ui.finish_output();

    assert_eq!(area.scroll.response.id, area_id);
    assert_eq!(area.scroll.response.rect, viewport);
    assert_eq!(area.scroll.offset, retained);
    assert_eq!(area.scroll.delta, Vec2::ZERO);
    assert_eq!(area.scroll.max_offset, Vec2::new(120.0, 90.0));
    assert_eq!(area.inner.0, retained);
    assert_eq!(area.inner.1, "logical extent retained");
    assert_eq!(area.inner.2, 7);
    assert_eq!(memory.scroll_offset(area_id), retained);
    assert_eq!(
        frame
            .semantics
            .get(area_id)
            .expect("scroll semantics")
            .bounds,
        viewport
    );
    assert!(frame.warnings.is_empty());
}
