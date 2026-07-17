//! Bounded icon fallback, geometry, and accessible-name conformance evidence.

use stern_core::{
    Alignment, ImageId, Key, KeyEvent, KeyState, KeyboardInput, Modifiers, PathElement,
    PlatformRequest, Point, PointerButtonState, PointerInput, Primitive, Rect, SemanticActionKind,
    Size, UiInput, UiMemory, WidgetId, default_dark_theme, fit_box,
};
use stern_widgets::{
    IconGraphic, IconId, IconLibrary, IconPath, Ui, WidgetOutput, icon_button_with_library,
    image_icon_button,
};

const OUTER: Rect = Rect::new(12.25, 20.5, 40.0, 32.0);
const LABEL: &str = "Open inspector";

fn icon_id(raw: u64) -> IconId {
    IconId::from_raw(raw)
}

fn widget_id() -> WidgetId {
    WidgetId::from_key("bounded-icon")
}

fn graphic(inset: f32) -> IconGraphic {
    IconGraphic::new(
        Rect::new(0.0, 0.0, 16.0, 16.0),
        vec![IconPath::filled(vec![
            PathElement::MoveTo(Point::new(inset, inset)),
            PathElement::LineTo(Point::new(16.0 - inset, inset)),
            PathElement::LineTo(Point::new(16.0 - inset, 16.0 - inset)),
            PathElement::LineTo(Point::new(inset, 16.0 - inset)),
            PathElement::Close,
        ])],
    )
}

fn render_vector(
    icons: &IconLibrary,
    icon: IconId,
    input: &UiInput,
    memory: &mut UiMemory,
    disabled: bool,
) -> WidgetOutput {
    icon_button_with_library(
        widget_id(),
        OUTER,
        icon,
        LABEL,
        icons,
        input,
        memory,
        &default_dark_theme(),
        disabled,
    )
}

fn optical_box() -> Rect {
    let theme = default_dark_theme();
    fit_box(
        OUTER,
        Size::new(theme.sizes.icon.md, theme.sizes.icon.md),
        Alignment::Center,
        Alignment::Center,
    )
}

fn pointer_input(down: bool, pressed: bool, released: bool) -> UiInput {
    UiInput {
        pointer: PointerInput {
            position: Some(OUTER.center()),
            primary: PointerButtonState::new(down, pressed, released),
            click_count: u8::from(released),
            ..PointerInput::default()
        },
        ..UiInput::default()
    }
}

fn assert_outer_contract(output: &WidgetOutput) {
    let response = output.response.expect("icon response");
    assert_eq!(response.id, widget_id());
    assert_eq!(response.rect, OUTER);
    let [semantic] = output.semantics.as_slice() else {
        panic!("one icon semantic node");
    };
    assert_eq!(semantic.id, widget_id());
    assert_eq!(semantic.bounds, OUTER);
    assert_eq!(semantic.label.as_deref(), Some(LABEL));
}

fn assert_point_in(rect: Rect, point: Point) {
    assert!(
        point.x >= rect.x
            && point.x <= rect.max_x()
            && point.y >= rect.y
            && point.y <= rect.max_y(),
        "point {point:?} escaped {rect:?}"
    );
}

fn assert_icon_tail_contained(output: &WidgetOutput, count: usize) {
    assert!(count > 0 && output.primitives.len() > count);
    let icon_primitives = &output.primitives[output.primitives.len() - count..];
    for primitive in icon_primitives {
        match primitive {
            Primitive::Path(path) => {
                assert!(!path.elements.is_empty());
                for element in &path.elements {
                    match element {
                        PathElement::MoveTo(point) | PathElement::LineTo(point) => {
                            assert_point_in(optical_box(), *point);
                        }
                        PathElement::QuadTo { ctrl, to } => {
                            assert_point_in(optical_box(), *ctrl);
                            assert_point_in(optical_box(), *to);
                        }
                        PathElement::CubicTo { ctrl1, ctrl2, to } => {
                            assert_point_in(optical_box(), *ctrl1);
                            assert_point_in(optical_box(), *ctrl2);
                            assert_point_in(optical_box(), *to);
                        }
                        PathElement::Close => {}
                    }
                }
            }
            Primitive::Line(line) => {
                assert_point_in(optical_box(), line.from);
                assert_point_in(optical_box(), line.to);
            }
            Primitive::Image(image) => assert!(optical_box().contains_rect(image.rect)),
            other => panic!("unexpected icon-tail primitive {other:?}"),
        }
    }
}

#[test]
fn registered_missing_and_unpaintable_icons_keep_exact_outer_contract_bounds() {
    let icon = icon_id(7);
    let input = UiInput::default();
    let mut memory = UiMemory::new();
    let mut icons = IconLibrary::new();

    let missing_before = render_vector(&icons, icon, &input, &mut memory, false);
    icons.register(icon, graphic(2.0));
    let registered = render_vector(&icons, icon, &input, &mut memory, false);
    icons = IconLibrary::new();
    let missing_after = render_vector(&icons, icon, &input, &mut memory, false);

    let invalid_graphics = [
        IconGraphic::new(
            Rect::new(0.0, 0.0, 0.0, 16.0),
            vec![IconPath::filled(vec![PathElement::Close])],
        ),
        IconGraphic::new(Rect::new(0.0, 0.0, 16.0, 16.0), Vec::new()),
        IconGraphic::new(
            Rect::new(0.0, 0.0, 16.0, 16.0),
            vec![IconPath {
                elements: vec![PathElement::MoveTo(Point::new(4.0, 4.0))],
                fill: false,
                stroke_width: None,
            }],
        ),
    ];
    let invalid = invalid_graphics.map(|graphic| {
        let mut icons = IconLibrary::new();
        icons.register(icon, graphic);
        render_vector(&icons, icon, &input, &mut memory, false)
    });

    let expected_response = missing_before.response.expect("missing response");
    for output in [
        &missing_before,
        &registered,
        &missing_after,
        &invalid[0],
        &invalid[1],
        &invalid[2],
    ] {
        assert_outer_contract(output);
        assert_eq!(output.response, Some(expected_response));
    }

    assert_icon_tail_contained(&registered, 1);
    for output in [
        &missing_before,
        &missing_after,
        &invalid[0],
        &invalid[1],
        &invalid[2],
    ] {
        assert_icon_tail_contained(output, 2);
    }
    assert_eq!(missing_before.primitives, missing_after.primitives);
    assert_ne!(registered.primitives, missing_before.primitives);
}

#[test]
fn icon_bounds_survive_idle_hover_press_and_disabled_states() {
    let icon = icon_id(11);
    let mut icons = IconLibrary::new();
    icons.register(icon, graphic(3.0));

    let idle = render_vector(
        &icons,
        icon,
        &UiInput::default(),
        &mut UiMemory::new(),
        false,
    );
    let hovered = render_vector(
        &icons,
        icon,
        &pointer_input(false, false, false),
        &mut UiMemory::new(),
        false,
    );
    let pressed = render_vector(
        &icons,
        icon,
        &pointer_input(true, true, false),
        &mut UiMemory::new(),
        false,
    );
    let disabled = render_vector(
        &icons,
        icon,
        &pointer_input(true, true, false),
        &mut UiMemory::new(),
        true,
    );

    for output in [&idle, &hovered, &pressed, &disabled] {
        assert_outer_contract(output);
        assert_icon_tail_contained(output, 1);
    }
    assert!(!idle.response.expect("idle response").state.hovered);
    assert!(hovered.response.expect("hover response").state.hovered);
    assert!(pressed.response.expect("pressed response").state.pressed);

    let disabled_response = disabled.response.expect("disabled response");
    assert!(disabled_response.state.disabled);
    assert!(!disabled_response.clicked);
    assert!(!disabled_response.keyboard_activated);
    let disabled_semantic = &disabled.semantics[0];
    assert!(disabled_semantic.state.disabled);
    assert!(!disabled_semantic.focusable);
    assert_eq!(disabled_semantic.label.as_deref(), Some(LABEL));
}
