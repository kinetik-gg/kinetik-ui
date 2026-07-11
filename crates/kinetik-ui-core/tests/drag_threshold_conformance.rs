//! Ordered drag-threshold and captured-selection gesture conformance.

use kinetik_ui_core::{
    ClipId, InputWheelDelta, Key, KeyEvent, KeyState, Modifiers, MouseButton, Point, Primitive,
    Rect, SelectionGesturePhase, Size, TextInputEvent, UiInputEvent, UiTestHarness, Vec2,
    draggable, drop_target, pressable, scrollable, tooltip_trigger,
};

const FULL: Rect = Rect::new(0.0, 0.0, 160.0, 80.0);

fn run_drag(harness: &mut UiTestHarness) -> kinetik_ui_core::Response {
    harness
        .run_frame(|ui| {
            let id = ui.id("drag");
            let (input, memory) = ui.input_and_memory_mut();
            draggable(id, FULL, input, memory, false)
        })
        .0
}

fn run_press(harness: &mut UiTestHarness) -> kinetik_ui_core::Response {
    harness
        .run_frame(|ui| {
            let id = ui.id("press");
            let (input, memory) = ui.input_and_memory_mut();
            pressable(id, FULL, input, memory, false)
        })
        .0
}

#[test]
fn below_threshold_release_clicks_and_exact_threshold_suppresses_click() {
    let mut below = UiTestHarness::new();
    below.set_pointer_position(Point::new(10.0, 10.0));
    below.pointer_press(MouseButton::Primary);
    let pressed = run_drag(&mut below);
    below.set_pointer_position(Point::new(13.0, 10.0));
    let moved = run_drag(&mut below);
    assert!(!moved.dragged);
    assert_eq!(below.memory().drag_source(), None);
    below.pointer_release(MouseButton::Primary);
    let released = run_drag(&mut below);
    assert!(released.clicked);
    assert!(!released.double_clicked);
    assert_eq!(below.memory().released_drag_source(), None);
    assert_eq!(below.memory().pointer_capture(), None);

    let mut exact = UiTestHarness::new();
    exact.set_pointer_position(Point::new(10.0, 10.0));
    exact.pointer_press(MouseButton::Primary);
    let _ = run_drag(&mut exact);
    exact.set_pointer_position(Point::new(14.0, 10.0));
    let crossing = run_drag(&mut exact);
    assert!(crossing.dragged);
    assert_eq!(crossing.drag_delta, Vec2::new(4.0, 0.0));
    assert_eq!(exact.memory().drag_source(), Some(pressed.id));
    exact.pointer_release(MouseButton::Primary);
    let released = run_drag(&mut exact);
    assert!(!released.clicked);
    assert_eq!(exact.memory().released_drag_source(), Some(released.id));
}

#[test]
fn crossing_reports_full_displacement_then_subsequent_delta_and_never_unlatches() {
    let mut harness = UiTestHarness::new();
    harness.set_pointer_position(Point::new(10.0, 10.0));
    harness.pointer_press(MouseButton::Primary);
    let _ = run_drag(&mut harness);

    harness.set_pointer_position(Point::new(12.0, 10.0));
    assert!(!run_drag(&mut harness).dragged);

    harness.set_pointer_position(Point::new(14.0, 10.0));
    let crossing = run_drag(&mut harness);
    assert!(crossing.dragged);
    assert_eq!(crossing.drag_delta, Vec2::new(4.0, 0.0));

    harness.set_pointer_position(Point::new(17.0, 10.0));
    let later = run_drag(&mut harness);
    assert!(later.dragged);
    assert_eq!(later.drag_delta, Vec2::new(3.0, 0.0));

    harness.set_pointer_position(Point::new(11.0, 10.0));
    let moved_back = run_drag(&mut harness);
    assert!(moved_back.dragged);
    assert_eq!(moved_back.drag_delta, Vec2::new(-6.0, 0.0));

    harness.pointer_release(MouseButton::Primary);
    let released = run_drag(&mut harness);
    assert!(!released.clicked);
    assert_eq!(harness.memory().released_drag_source(), Some(released.id));
}

#[test]
fn same_frame_crossing_release_reports_inside_motion_but_outside_only_cleans_up() {
    let mut inside = UiTestHarness::new();
    inside.set_pointer_position(Point::new(10.0, 10.0));
    inside.pointer_press(MouseButton::Primary);
    inside.set_pointer_position(Point::new(14.0, 10.0));
    inside.pointer_release(MouseButton::Primary);
    let response = run_drag(&mut inside);
    assert!(response.dragged);
    assert_eq!(response.drag_delta, Vec2::new(4.0, 0.0));
    assert!(!response.clicked);
    assert_eq!(inside.memory().released_drag_source(), Some(response.id));

    let mut outside = UiTestHarness::new();
    outside.set_pointer_position(Point::new(10.0, 10.0));
    outside.pointer_press(MouseButton::Primary);
    outside.set_pointer_position(Point::new(200.0, 10.0));
    outside.pointer_release(MouseButton::Primary);
    let response = run_drag(&mut outside);
    assert!(!response.dragged);
    assert_eq!(response.drag_delta, Vec2::ZERO);
    assert!(!response.clicked);
    assert_eq!(outside.memory().pointer_capture(), None);
    assert_eq!(outside.memory().released_drag_source(), Some(response.id));
}

#[test]
fn release_event_position_can_cross_threshold_without_a_move_event() {
    let mut harness = UiTestHarness::new();
    harness.set_pointer_position(Point::new(10.0, 10.0));
    harness.pointer_press(MouseButton::Primary);
    harness.input_mut().push_event(UiInputEvent::PointerButton {
        button: MouseButton::Primary,
        down: false,
        click_count: 1,
        position: Some(Point::new(14.0, 10.0)),
    });

    let response = run_drag(&mut harness);
    assert!(response.dragged);
    assert_eq!(response.drag_delta, Vec2::new(4.0, 0.0));
    assert!(!response.clicked);
    assert_eq!(harness.memory().released_drag_source(), Some(response.id));
}

#[test]
fn pressable_suppresses_threshold_release_without_becoming_a_drag_source() {
    let mut harness = UiTestHarness::new();
    harness.set_pointer_position(Point::new(10.0, 10.0));
    harness.pointer_press(MouseButton::Primary);
    let _ = run_press(&mut harness);
    harness.set_pointer_position(Point::new(14.0, 10.0));
    let moved = run_press(&mut harness);
    assert!(!moved.dragged);
    assert_eq!(harness.memory().drag_source(), None);
    harness.pointer_release(MouseButton::Primary);
    let released = run_press(&mut harness);
    assert!(!released.clicked);
    assert_eq!(harness.memory().released_drag_source(), None);
}

#[test]
fn below_threshold_canonical_double_click_preserves_live_click_count() {
    let mut harness = UiTestHarness::new();
    harness.set_pointer_position(Point::new(10.0, 10.0));
    harness.pointer_press(MouseButton::Primary);
    harness.set_click_count(2);
    let _ = run_press(&mut harness);
    harness.set_pointer_position(Point::new(12.0, 10.0));
    let _ = run_press(&mut harness);
    harness.pointer_release(MouseButton::Primary);
    harness.set_click_count(2);
    let response = run_press(&mut harness);
    assert!(response.clicked);
    assert!(response.double_clicked);
}

#[test]
fn captured_selection_preserves_root_ordinals_without_domain_drag_or_replay() {
    let mut harness = UiTestHarness::new();
    harness.set_pointer_position(Point::new(10.0, 10.0));
    harness.pointer_press(MouseButton::Primary);
    harness
        .input_mut()
        .push_event(UiInputEvent::Text(TextInputEvent::Commit("x".to_owned())));
    harness.set_pointer_position(Point::new(18.0, 10.0));
    harness
        .input_mut()
        .push_event(UiInputEvent::Key(KeyEvent::new(
            Key::ArrowRight,
            KeyState::Pressed,
            Modifiers::default(),
            false,
        )));
    harness.pointer_release(MouseButton::Primary);

    let ((first, second), _) = harness.run_frame(|ui| {
        let id = ui.id("selection");
        let first = ui.captured_selection_gesture(id, FULL, false);
        let second = ui.captured_selection_gesture(id, FULL, false);
        (first, second)
    });

    assert_eq!(
        first
            .actions
            .iter()
            .map(|action| (action.ordinal, action.phase))
            .collect::<Vec<_>>(),
        vec![
            (Some(1), SelectionGesturePhase::Press),
            (Some(3), SelectionGesturePhase::Move),
            (Some(5), SelectionGesturePhase::Release),
        ]
    );
    assert_eq!(first.actions[1].delta, Vec2::new(8.0, 0.0));
    assert!(second.actions.is_empty());
    assert_eq!(harness.memory().drag_source(), None);
    assert_eq!(harness.memory().released_drag_source(), None);
}

#[test]
fn spatial_filtering_keeps_original_action_ordinals_with_gaps() {
    let mut harness = UiTestHarness::new();
    harness.set_pointer_position(Point::new(200.0, 10.0));
    harness
        .input_mut()
        .push_event(UiInputEvent::Text(TextInputEvent::Commit(
            "before".to_owned(),
        )));
    harness.set_pointer_position(Point::new(10.0, 10.0));
    harness.pointer_press(MouseButton::Primary);
    harness
        .input_mut()
        .push_event(UiInputEvent::Key(KeyEvent::new(
            Key::ArrowLeft,
            KeyState::Pressed,
            Modifiers::default(),
            false,
        )));
    harness.set_pointer_position(Point::new(12.0, 10.0));
    harness.pointer_release(MouseButton::Primary);

    let clip = ClipId::from_raw(91);
    let (gesture, _) = harness.run_frame(|ui| {
        let id = ui.id("selection");
        ui.push_primitive(Primitive::ClipBegin {
            id: clip,
            rect: FULL,
        });
        let gesture = ui.captured_selection_gesture(id, FULL, false);
        ui.push_primitive(Primitive::ClipEnd { id: clip });
        gesture
    });

    assert_eq!(
        gesture
            .actions
            .iter()
            .map(|action| action.ordinal)
            .collect::<Vec<_>>(),
        vec![Some(3), Some(5), Some(6)]
    );
    assert!(gesture.response.clicked);
}

#[test]
fn ordered_text_claim_exposes_root_ordinals_without_pointer_reparsing() {
    let mut harness = UiTestHarness::new();
    harness.set_pointer_position(Point::new(200.0, 10.0));
    harness.input_mut().push_event(UiInputEvent::Wheel {
        delta: InputWheelDelta::Pixels(Vec2::new(0.0, 1.0)),
        position: Some(Point::new(200.0, 10.0)),
    });
    harness.input_mut().push_event(UiInputEvent::PointerButton {
        button: MouseButton::Primary,
        down: true,
        click_count: 1,
        position: Some(Point::new(10.0, 10.0)),
    });
    harness
        .input_mut()
        .push_event(UiInputEvent::Text(TextInputEvent::Commit(
            "typed".to_owned(),
        )));

    let clip = ClipId::from_raw(94);
    let ((gesture, editing), _) = harness.run_frame(|ui| {
        let id = ui.id("selection");
        ui.memory_mut().focus(id);
        ui.memory_mut().set_text_input_owner(id);
        ui.push_primitive(Primitive::ClipBegin {
            id: clip,
            rect: FULL,
        });
        let gesture = ui.captured_selection_gesture(id, FULL, false);
        let editing = ui
            .claim_ordered_text_input_events(id)
            .expect("valid root stream")
            .expect("focused owner claim");
        ui.push_primitive(Primitive::ClipEnd { id: clip });
        (gesture, editing)
    });

    assert_eq!(gesture.actions[0].ordinal, Some(2));
    assert_eq!(editing.len(), 1);
    assert_eq!(editing[0].ordinal, Some(3));
    assert!(matches!(
        &editing[0].event,
        UiInputEvent::Text(TextInputEvent::Commit(text)) if text == "typed"
    ));
}

#[test]
fn canonical_release_outside_clip_is_cancel_only_with_original_ordinal() {
    let clip = ClipId::from_raw(92);
    let clip_rect = Rect::new(0.0, 0.0, 20.0, 20.0);
    let mut harness = UiTestHarness::new();
    harness.set_pointer_position(Point::new(10.0, 10.0));
    harness.pointer_press(MouseButton::Primary);
    let pressed = harness
        .run_frame(|ui| {
            let id = ui.id("selection");
            ui.push_primitive(Primitive::ClipBegin {
                id: clip,
                rect: clip_rect,
            });
            let gesture = ui.captured_selection_gesture(id, FULL, false);
            ui.push_primitive(Primitive::ClipEnd { id: clip });
            gesture
        })
        .0;
    assert_eq!(
        harness.memory().pointer_capture(),
        Some(pressed.response.id)
    );

    harness.set_pointer_position(Point::new(50.0, 10.0));
    harness.pointer_release(MouseButton::Primary);
    let cancelled = harness
        .run_frame(|ui| {
            let id = ui.id("selection");
            ui.push_primitive(Primitive::ClipBegin {
                id: clip,
                rect: clip_rect,
            });
            let gesture = ui.captured_selection_gesture(id, FULL, false);
            ui.push_primitive(Primitive::ClipEnd { id: clip });
            gesture
        })
        .0;

    assert_eq!(cancelled.actions.len(), 1);
    assert_eq!(cancelled.actions[0].ordinal, Some(1));
    assert_eq!(cancelled.actions[0].phase, SelectionGesturePhase::Cancel);
    assert!(!cancelled.response.clicked);
    assert!(!cancelled.response.dragged);
    assert_eq!(harness.memory().pointer_capture(), None);

    let mut ordinary = UiTestHarness::new();
    ordinary.set_pointer_position(Point::new(10.0, 10.0));
    ordinary.pointer_press(MouseButton::Primary);
    let _ = ordinary.run_frame(|ui| {
        let id = ui.id("press");
        ui.push_primitive(Primitive::ClipBegin {
            id: clip,
            rect: clip_rect,
        });
        let (input, memory) = ui.input_and_memory_mut();
        let response = pressable(id, FULL, input, memory, false);
        ui.push_primitive(Primitive::ClipEnd { id: clip });
        response
    });
    ordinary.set_pointer_position(Point::new(50.0, 10.0));
    ordinary.pointer_release(MouseButton::Primary);
    let response = ordinary
        .run_frame(|ui| {
            let id = ui.id("press");
            ui.push_primitive(Primitive::ClipBegin {
                id: clip,
                rect: clip_rect,
            });
            let (input, memory) = ui.input_and_memory_mut();
            let response = pressable(id, FULL, input, memory, false);
            ui.push_primitive(Primitive::ClipEnd { id: clip });
            response
        })
        .0;
    assert!(!response.clicked);
    assert_eq!(ordinary.memory().pointer_capture(), None);
}

#[test]
fn canonical_secondary_release_outside_clip_is_cleanup_only() {
    let clip = ClipId::from_raw(95);
    let clip_rect = Rect::new(0.0, 0.0, 20.0, 20.0);
    let mut harness = UiTestHarness::new();
    harness.set_pointer_position(Point::new(10.0, 10.0));
    harness.pointer_press(MouseButton::Secondary);
    let _ = harness.run_frame(|ui| {
        let id = ui.id("press");
        ui.push_primitive(Primitive::ClipBegin {
            id: clip,
            rect: clip_rect,
        });
        let (input, memory) = ui.input_and_memory_mut();
        let response = pressable(id, FULL, input, memory, false);
        ui.push_primitive(Primitive::ClipEnd { id: clip });
        response
    });
    harness.set_pointer_position(Point::new(50.0, 10.0));
    harness.pointer_release(MouseButton::Secondary);
    let response = harness
        .run_frame(|ui| {
            let id = ui.id("press");
            ui.push_primitive(Primitive::ClipBegin {
                id: clip,
                rect: clip_rect,
            });
            let (input, memory) = ui.input_and_memory_mut();
            let response = pressable(id, FULL, input, memory, false);
            ui.push_primitive(Primitive::ClipEnd { id: clip });
            response
        })
        .0;
    assert!(!response.secondary_clicked);
    assert_eq!(harness.memory().secondary_pressed(), None);
}

#[test]
fn release_all_emits_one_original_ordinal_cancel_and_clears_selection_capture() {
    let mut harness = UiTestHarness::new();
    harness.set_pointer_position(Point::new(10.0, 10.0));
    harness.pointer_press(MouseButton::Primary);
    let pressed = harness
        .run_frame(|ui| {
            let id = ui.id("selection");
            ui.captured_selection_gesture(id, FULL, false)
        })
        .0;
    assert_eq!(
        harness.memory().pointer_capture(),
        Some(pressed.response.id)
    );

    harness.input_mut().release_pointer_buttons();
    let cancelled = harness
        .run_frame(|ui| {
            let id = ui.id("selection");
            ui.captured_selection_gesture(id, FULL, false)
        })
        .0;
    assert_eq!(cancelled.actions.len(), 1);
    assert_eq!(cancelled.actions[0].ordinal, Some(0));
    assert_eq!(cancelled.actions[0].phase, SelectionGesturePhase::Cancel);
    assert_eq!(harness.memory().pointer_capture(), None);
    assert_eq!(harness.memory().drag_source(), None);

    let clip = ClipId::from_raw(93);
    let clip_rect = Rect::new(0.0, 0.0, 20.0, 20.0);
    let mut clipped = UiTestHarness::new();
    clipped.set_pointer_position(Point::new(10.0, 10.0));
    clipped.pointer_press(MouseButton::Primary);
    let _ = clipped.run_frame(|ui| {
        let id = ui.id("selection");
        ui.push_primitive(Primitive::ClipBegin {
            id: clip,
            rect: clip_rect,
        });
        let gesture = ui.captured_selection_gesture(id, FULL, false);
        ui.push_primitive(Primitive::ClipEnd { id: clip });
        gesture
    });
    clipped.set_pointer_position(Point::new(50.0, 10.0));
    clipped.input_mut().release_pointer_buttons();
    let cancelled = clipped
        .run_frame(|ui| {
            let id = ui.id("selection");
            ui.push_primitive(Primitive::ClipBegin {
                id: clip,
                rect: clip_rect,
            });
            let gesture = ui.captured_selection_gesture(id, FULL, false);
            ui.push_primitive(Primitive::ClipEnd { id: clip });
            gesture
        })
        .0;
    assert_eq!(cancelled.actions.len(), 1);
    assert_eq!(cancelled.actions[0].ordinal, Some(1));
    assert_eq!(cancelled.actions[0].phase, SelectionGesturePhase::Cancel);
}

#[test]
fn ordered_move_before_release_all_is_not_discarded_by_frame_cleanup() {
    let mut harness = UiTestHarness::new();
    harness.set_pointer_position(Point::new(10.0, 10.0));
    harness.pointer_press(MouseButton::Primary);
    let _ = harness.run_frame(|ui| {
        let id = ui.id("selection");
        ui.captured_selection_gesture(id, FULL, false)
    });

    harness.set_pointer_position(Point::new(12.0, 10.0));
    harness.input_mut().release_pointer_buttons();
    let gesture = harness
        .run_frame(|ui| {
            let id = ui.id("selection");
            ui.captured_selection_gesture(id, FULL, false)
        })
        .0;
    assert_eq!(
        gesture
            .actions
            .iter()
            .map(|action| (action.ordinal, action.phase))
            .collect::<Vec<_>>(),
        vec![
            (Some(0), SelectionGesturePhase::Move),
            (Some(1), SelectionGesturePhase::Cancel),
        ]
    );
    assert_eq!(harness.memory().pointer_capture(), None);
}

#[test]
fn secondary_owner_clears_when_the_participating_widget_becomes_disabled() {
    let mut harness = UiTestHarness::new();
    harness.set_pointer_position(Point::new(10.0, 10.0));
    harness.pointer_press(MouseButton::Secondary);
    let owner = run_press(&mut harness).id;
    assert_eq!(harness.memory().secondary_pressed(), Some(owner));

    let response = harness
        .run_frame(|ui| {
            let id = ui.id("press");
            let (input, memory) = ui.input_and_memory_mut();
            pressable(id, FULL, input, memory, true)
        })
        .0;
    assert!(response.state.disabled);
    assert_eq!(harness.memory().secondary_pressed(), None);
}

#[test]
fn selection_mode_change_cannot_publish_a_retained_domain_drop() {
    let mut harness = UiTestHarness::new();
    harness.set_pointer_position(Point::new(10.0, 10.0));
    harness.pointer_press(MouseButton::Primary);
    let _ = run_drag(&mut harness);
    harness.set_pointer_position(Point::new(14.0, 10.0));
    let crossed = run_drag(&mut harness);
    assert_eq!(harness.memory().drag_source(), Some(crossed.id));

    harness.pointer_release(MouseButton::Primary);
    let gesture = harness
        .run_frame(|ui| {
            let id = ui.id("drag");
            ui.captured_selection_gesture(id, FULL, false)
        })
        .0;
    assert_eq!(gesture.actions[0].phase, SelectionGesturePhase::Release);
    assert_eq!(harness.memory().drag_source(), None);
    assert_eq!(harness.memory().released_drag_source(), None);
}

#[test]
fn conflicted_selection_release_is_cancel_only() {
    let mut harness = UiTestHarness::new();
    harness.set_pointer_position(Point::new(10.0, 10.0));
    harness.pointer_press(MouseButton::Primary);
    let _ = harness.run_frame(|ui| {
        let id = ui.id("selection");
        ui.captured_selection_gesture(id, FULL, false)
    });

    harness.pointer_release(MouseButton::Primary);
    harness.input_mut().pointer.delta = Vec2::new(99.0, 0.0);
    let gesture = harness
        .run_frame(|ui| {
            let id = ui.id("selection");
            ui.captured_selection_gesture(id, FULL, false)
        })
        .0;
    assert_eq!(gesture.actions.len(), 1);
    assert_eq!(gesture.actions[0].phase, SelectionGesturePhase::Cancel);
    assert!(!gesture.response.clicked);
    assert_eq!(harness.memory().pointer_capture(), None);
}

#[test]
fn conflicting_snapshot_only_focus_loss_cannot_invent_an_ordered_cancel() {
    let mut harness = UiTestHarness::new();
    harness.set_pointer_position(Point::new(10.0, 10.0));
    harness.pointer_press(MouseButton::Primary);
    let owner = run_press(&mut harness).id;
    harness
        .input_mut()
        .push_event(UiInputEvent::Key(KeyEvent::new(
            Key::ArrowLeft,
            KeyState::Pressed,
            Modifiers::default(),
            false,
        )));
    harness.input_mut().window_focused = false;

    let response = run_press(&mut harness);
    assert!(!response.clicked);
    assert_eq!(harness.memory().pointer_capture(), Some(owner));
    assert!(!harness.memory().pointer_interaction_cancelled());
}

#[test]
fn root_conflict_blocks_tooltip_and_scroll_hover_without_discarding_canonical_wheel() {
    let mut harness = UiTestHarness::new();
    harness.set_pointer_position(Point::new(10.0, 10.0));
    harness.wheel_pixels(Vec2::new(0.0, -20.0));
    harness.input_mut().pointer.delta = Vec2::new(99.0, 0.0);

    let ((tooltip, scroll), output) = harness.run_frame(|ui| {
        let tooltip_id = ui.id("tooltip");
        let scroll_id = ui.id("scroll");
        let (input, memory) = ui.input_and_memory_mut();
        let tooltip = tooltip_trigger(tooltip_id, FULL, input, memory, false);
        let (input, memory) = ui.input_and_memory_mut();
        let scroll = scrollable(
            scroll_id,
            FULL,
            Size::new(320.0, 320.0),
            input,
            memory,
            false,
        );
        (tooltip, scroll)
    });
    assert!(!tooltip.state.hovered);
    assert!(!tooltip.tooltip_requested);
    assert!(!scroll.response.state.hovered);
    assert_eq!(scroll.delta, Vec2::new(0.0, 20.0));
    assert_eq!(output.warnings.len(), 1);
}

#[test]
fn drop_is_ineligible_below_threshold_and_order_independent_after_crossing() {
    let mut below = UiTestHarness::new();
    below.set_pointer_position(Point::new(10.0, 10.0));
    below.pointer_press(MouseButton::Primary);
    let _ = run_drag(&mut below);
    below.set_pointer_position(Point::new(13.0, 10.0));
    let _ = run_drag(&mut below);
    below.pointer_release(MouseButton::Primary);
    let ((drop, source), _) = below.run_frame(|ui| {
        let source = ui.id("drag");
        let target = ui.id("drop");
        let (input, memory) = ui.input_and_memory_mut();
        let drop = drop_target(target, FULL, input, memory, false);
        let (input, memory) = ui.input_and_memory_mut();
        let source = draggable(source, FULL, input, memory, false);
        (drop, source)
    });
    assert_eq!(drop.source, None);
    assert!(!drop.dropped);
    assert!(source.clicked);

    let mut crossed = UiTestHarness::new();
    crossed.set_pointer_position(Point::new(10.0, 10.0));
    crossed.pointer_press(MouseButton::Primary);
    let _ = run_drag(&mut crossed);
    crossed.set_pointer_position(Point::new(14.0, 10.0));
    let _ = run_drag(&mut crossed);
    crossed.pointer_release(MouseButton::Primary);
    let ((drop, source), _) = crossed.run_frame(|ui| {
        let source = ui.id("drag");
        let target = ui.id("drop");
        let (input, memory) = ui.input_and_memory_mut();
        let drop = drop_target(target, FULL, input, memory, false);
        let (input, memory) = ui.input_and_memory_mut();
        let source = draggable(source, FULL, input, memory, false);
        (drop, source)
    });
    assert_eq!(drop.source, Some(source.id));
    assert!(drop.dropped);
    assert!(!source.clicked);
}

#[test]
fn drop_uses_canonical_release_geometry_and_rejects_missing_event_position() {
    let source = kinetik_ui_core::WidgetId::from_key("source");
    let target = kinetik_ui_core::WidgetId::from_key("target");

    let mut missing = UiTestHarness::new();
    missing.memory_mut().capture_pointer(source);
    missing.memory_mut().activate(source);
    missing.memory_mut().press(source);
    missing.memory_mut().start_drag(source);
    missing.input_mut().pointer.position = Some(Point::new(10.0, 10.0));
    missing.input_mut().push_event(UiInputEvent::PointerButton {
        button: MouseButton::Primary,
        down: false,
        click_count: 1,
        position: None,
    });
    let drop = missing
        .run_frame(|ui| {
            ui.register_id(source);
            ui.register_id(target);
            let (input, memory) = ui.input_and_memory_mut();
            drop_target(target, FULL, input, memory, false)
        })
        .0;
    assert_eq!(drop.source, None);
    assert!(!drop.dropped);

    let mut ordered = UiTestHarness::new();
    ordered.memory_mut().capture_pointer(source);
    ordered.memory_mut().activate(source);
    ordered.memory_mut().press(source);
    ordered.memory_mut().start_drag(source);
    ordered.input_mut().pointer.position = Some(Point::new(10.0, 10.0));
    ordered.input_mut().pointer.primary.down = true;
    ordered.input_mut().push_event(UiInputEvent::PointerButton {
        button: MouseButton::Primary,
        down: false,
        click_count: 1,
        position: Some(Point::new(10.0, 10.0)),
    });
    ordered.input_mut().push_event(UiInputEvent::PointerMoved {
        position: Point::new(200.0, 10.0),
        delta: Vec2::new(190.0, 0.0),
    });
    let drop = ordered
        .run_frame(|ui| {
            ui.register_id(source);
            ui.register_id(target);
            let (input, memory) = ui.input_and_memory_mut();
            drop_target(target, FULL, input, memory, false)
        })
        .0;
    assert_eq!(drop.source, Some(source));
    assert!(drop.dropped);
    assert!(drop.response.state.hovered);
}

#[test]
fn conflicted_release_cleans_existing_capture_without_click_drag_or_drop() {
    let mut harness = UiTestHarness::new();
    harness.set_pointer_position(Point::new(10.0, 10.0));
    harness.pointer_press(MouseButton::Primary);
    let pressed = run_drag(&mut harness);
    assert_eq!(harness.memory().pointer_capture(), Some(pressed.id));

    harness.pointer_release(MouseButton::Primary);
    harness.input_mut().pointer.delta = Vec2::new(99.0, 0.0);
    let ((response, drop), output) = harness.run_frame(|ui| {
        let source = ui.id("drag");
        let target = ui.id("drop");
        let (input, memory) = ui.input_and_memory_mut();
        let response = draggable(source, FULL, input, memory, false);
        let (input, memory) = ui.input_and_memory_mut();
        let drop = drop_target(target, FULL, input, memory, false);
        (response, drop)
    });

    assert!(!response.clicked);
    assert!(!response.dragged);
    assert_eq!(drop.source, None);
    assert!(!drop.dropped);
    assert_eq!(harness.memory().pointer_capture(), None);
    assert_eq!(harness.memory().released_drag_source(), None);
    assert_eq!(output.warnings.len(), 1);
}

#[test]
fn plain_pointer_capture_release_cleans_without_synthesizing_a_click() {
    let mut harness = UiTestHarness::new();
    let owner = kinetik_ui_core::WidgetId::from_key("plain-capture");
    harness.memory_mut().capture_pointer(owner);
    harness.set_pointer_position(Point::new(10.0, 10.0));
    harness.pointer_release(MouseButton::Primary);

    let response = harness
        .run_frame(|ui| {
            ui.register_id(owner);
            let (input, memory) = ui.input_and_memory_mut();
            pressable(owner, FULL, input, memory, false)
        })
        .0;
    assert!(!response.clicked);
    assert_eq!(harness.memory().pointer_capture(), None);
}

#[test]
fn canonical_button_without_event_position_never_uses_the_final_snapshot_position() {
    let mut missing_press = UiTestHarness::new();
    missing_press.set_pointer_position(Point::new(10.0, 10.0));
    missing_press
        .input_mut()
        .push_event(UiInputEvent::PointerButton {
            button: MouseButton::Primary,
            down: true,
            click_count: 1,
            position: None,
        });
    let response = run_press(&mut missing_press);
    assert!(!response.state.active);
    assert_eq!(missing_press.memory().pointer_capture(), None);

    let mut missing_release = UiTestHarness::new();
    missing_release.set_pointer_position(Point::new(10.0, 10.0));
    missing_release.pointer_press(MouseButton::Primary);
    let pressed = run_press(&mut missing_release);
    assert_eq!(missing_release.memory().pointer_capture(), Some(pressed.id));
    missing_release
        .input_mut()
        .push_event(UiInputEvent::PointerButton {
            button: MouseButton::Primary,
            down: false,
            click_count: 1,
            position: None,
        });
    let response = run_press(&mut missing_release);
    assert!(!response.clicked);
    assert_eq!(missing_release.memory().pointer_capture(), None);
}
