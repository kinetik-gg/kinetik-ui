//! Causal `DomainDrag` action and first-claim conformance.

use kinetik_ui_core::{
    ClipId, DomainDragGesturePhase, Modifiers, MouseButton, Point, PointerButtonState,
    PointerOrder, PointerTarget, Primitive, Rect, TextInputEvent, Transform, UiInput, UiInputEvent,
    UiMemory, UiTestHarness, Vec2, WidgetId, draggable, draggable_transformed, drop_target,
};

const FULL: Rect = Rect::new(0.0, 0.0, 160.0, 80.0);
const MISS: Rect = Rect::new(300.0, 0.0, 40.0, 40.0);
const CTRL: Modifiers = Modifiers {
    ctrl: true,
    alt: false,
    shift: false,
    super_key: false,
};
const SHIFT: Modifiers = Modifiers {
    ctrl: false,
    alt: false,
    shift: true,
    super_key: false,
};

fn release_outcome_at(end: Point) -> kinetik_ui_core::CapturedDomainDragGesture {
    let mut harness = UiTestHarness::new();
    harness.set_pointer_position(Point::new(10.0, 10.0));
    harness.pointer_press(MouseButton::Primary);
    harness.set_pointer_position(end);
    harness.pointer_release(MouseButton::Primary);
    harness
        .run_frame(|ui| {
            let id = ui.id("drag");
            ui.captured_domain_drag_gesture(id, FULL, false)
        })
        .0
}

fn release_actions(
    gesture: &kinetik_ui_core::CapturedDomainDragGesture,
) -> Vec<(Option<usize>, bool)> {
    gesture
        .actions
        .iter()
        .filter(|action| action.phase == DomainDragGesturePhase::Release)
        .map(|action| (action.ordinal, action.release_clicked))
        .collect()
}

fn start_crossed_drag(harness: &mut UiTestHarness) {
    harness.set_pointer_position(Point::new(10.0, 10.0));
    harness.pointer_press(MouseButton::Primary);
    let pressed = harness
        .run_frame(|ui| {
            let id = ui.id("source");
            ui.captured_domain_drag_gesture(id, FULL, false)
        })
        .0;
    assert_eq!(pressed.actions[0].phase, DomainDragGesturePhase::Press);

    harness.set_pointer_position(Point::new(14.0, 10.0));
    let crossed = harness
        .run_frame(|ui| {
            let id = ui.id("source");
            ui.captured_domain_drag_gesture(id, FULL, false)
        })
        .0;
    assert!(crossed.response.dragged);
    assert_eq!(crossed.response.drag_delta, Vec2::new(4.0, 0.0));
}

#[test]
fn release_actions_pin_below_exact_and_above_threshold_outcomes() {
    let below = release_outcome_at(Point::new(13.0, 10.0));
    assert!(below.response.clicked);
    assert_eq!(release_actions(&below), vec![(Some(3), true)]);

    let exact = release_outcome_at(Point::new(14.0, 10.0));
    assert!(!exact.response.clicked);
    assert!(exact.response.dragged);
    assert_eq!(release_actions(&exact), vec![(Some(3), false)]);

    let above = release_outcome_at(Point::new(18.0, 10.0));
    assert!(!above.response.clicked);
    assert!(above.response.dragged);
    assert_eq!(release_actions(&above), vec![(Some(3), false)]);
}

#[test]
fn each_same_frame_release_carries_its_own_click_result() {
    for crossed_first in [false, true] {
        let mut harness = UiTestHarness::new();
        let transactions = if crossed_first {
            [
                (Point::new(10.0, 10.0), Point::new(15.0, 10.0)),
                (Point::new(30.0, 10.0), Point::new(32.0, 10.0)),
            ]
        } else {
            [
                (Point::new(10.0, 10.0), Point::new(12.0, 10.0)),
                (Point::new(30.0, 10.0), Point::new(35.0, 10.0)),
            ]
        };
        for (start, end) in transactions {
            harness.set_pointer_position(start);
            harness.pointer_press(MouseButton::Primary);
            harness.set_pointer_position(end);
            harness.pointer_release(MouseButton::Primary);
        }

        let gesture = harness
            .run_frame(|ui| {
                let id = ui.id("drag");
                ui.captured_domain_drag_gesture(id, FULL, false)
            })
            .0;
        let outcomes = release_actions(&gesture)
            .into_iter()
            .map(|(_, clicked)| clicked)
            .collect::<Vec<_>>();
        assert_eq!(
            outcomes,
            if crossed_first {
                vec![false, true]
            } else {
                vec![true, false]
            }
        );
        assert!(gesture.response.clicked);
        assert!(gesture.response.dragged);
    }
}

#[test]
fn outside_and_missing_position_releases_never_claim_a_click() {
    let outside = release_outcome_at(Point::new(200.0, 10.0));
    assert!(!outside.response.clicked);
    assert_eq!(release_actions(&outside), vec![(Some(3), false)]);

    let mut missing = UiTestHarness::new();
    missing.set_pointer_position(Point::new(10.0, 10.0));
    missing.pointer_press(MouseButton::Primary);
    missing.input_mut().push_event(UiInputEvent::PointerButton {
        button: MouseButton::Primary,
        down: false,
        click_count: 1,
        position: None,
    });
    let gesture = missing
        .run_frame(|ui| {
            let id = ui.id("drag");
            ui.captured_domain_drag_gesture(id, FULL, false)
        })
        .0;
    assert!(!gesture.response.clicked);
    assert_eq!(release_actions(&gesture), vec![(Some(2), false)]);
}

#[test]
fn spatial_gaps_preserve_root_ordinals_local_positions_and_modifiers() {
    let mut harness = UiTestHarness::new();
    harness.set_pointer_position(Point::new(400.0, 10.0));
    harness.set_modifiers(CTRL);
    harness
        .input_mut()
        .push_event(UiInputEvent::Text(TextInputEvent::Commit("gap".to_owned())));
    harness.set_pointer_position(Point::new(20.0, 20.0));
    harness.pointer_press(MouseButton::Primary);
    harness.set_modifiers(SHIFT);
    harness.set_pointer_position(Point::new(24.0, 20.0));
    harness.pointer_release(MouseButton::Primary);

    let transform = Transform::scale(Vec2::new(2.0, 2.0));
    let clip = ClipId::from_raw(401);
    let gesture = harness
        .run_frame(|ui| {
            let id = ui.id("drag");
            ui.push_primitive(Primitive::TransformBegin(transform));
            ui.push_primitive(Primitive::ClipBegin {
                id: clip,
                rect: FULL,
            });
            let gesture = ui.captured_domain_drag_gesture(id, FULL, false);
            ui.push_primitive(Primitive::ClipEnd { id: clip });
            ui.push_primitive(Primitive::TransformEnd);
            gesture
        })
        .0;

    assert_eq!(
        gesture
            .actions
            .iter()
            .map(|action| (action.ordinal, action.modifiers))
            .collect::<Vec<_>>(),
        vec![(Some(4), CTRL), (Some(6), SHIFT), (Some(7), SHIFT)]
    );
    assert_eq!(gesture.actions[0].position, Some(Point::new(10.0, 10.0)));
    assert_eq!(gesture.actions[1].position, Some(Point::new(12.0, 10.0)));
    assert!(gesture.actions[2].release_clicked);
}

#[test]
fn legacy_actions_use_no_ordinal_and_snapshot_modifiers() {
    let mut harness = UiTestHarness::new();
    let input = harness.input_mut();
    input.keyboard.modifiers = CTRL;
    input.pointer.position = Some(Point::new(10.0, 10.0));
    input.pointer.primary = PointerButtonState::new(true, true, false);
    input.pointer.click_count = 1;
    assert!(input.events.is_empty());

    let gesture = harness
        .run_frame(|ui| {
            let id = ui.id("drag");
            ui.captured_domain_drag_gesture(id, FULL, false)
        })
        .0;
    assert_eq!(gesture.actions.len(), 1);
    assert_eq!(gesture.actions[0].phase, DomainDragGesturePhase::Press);
    assert_eq!(gesture.actions[0].ordinal, None);
    assert_eq!(gesture.actions[0].modifiers, CTRL);
    assert!(!gesture.actions[0].release_clicked);
}

#[test]
fn focus_loss_emits_one_non_clicking_cancel_with_event_time_modifiers() {
    let mut harness = UiTestHarness::new();
    harness.set_pointer_position(Point::new(10.0, 10.0));
    harness.pointer_press(MouseButton::Primary);
    let _ = harness.run_frame(|ui| {
        let id = ui.id("drag");
        ui.captured_domain_drag_gesture(id, FULL, false)
    });

    harness.set_modifiers(CTRL);
    harness.set_pointer_position(Point::new(12.0, 10.0));
    harness.set_window_focused(false);
    let gesture = harness
        .run_frame(|ui| {
            let id = ui.id("drag");
            ui.captured_domain_drag_gesture(id, FULL, false)
        })
        .0;
    let cancels = gesture
        .actions
        .iter()
        .filter(|action| action.phase == DomainDragGesturePhase::Cancel)
        .collect::<Vec<_>>();
    assert_eq!(cancels.len(), 1);
    assert_eq!(cancels[0].modifiers, CTRL);
    assert!(!cancels[0].release_clicked);
    assert_eq!(harness.memory().pointer_capture(), None);
    assert_eq!(harness.memory().drag_source(), None);
}

#[test]
fn captured_duplicates_return_the_exact_response_without_memory_mutation() {
    let mut harness = UiTestHarness::new();
    harness.set_pointer_position(Point::new(10.0, 10.0));
    harness.pointer_press(MouseButton::Primary);
    harness.set_pointer_position(Point::new(14.0, 10.0));

    let ((first, second, unchanged), _) = harness.run_frame(|ui| {
        let id = ui.id("drag");
        let first = ui.captured_domain_drag_gesture(id, FULL, false);
        let after_first = ui.memory().clone();
        let second = ui.captured_domain_drag_gesture(id, MISS, true);
        let unchanged = after_first == *ui.memory();
        (first, second, unchanged)
    });
    assert!(!first.actions.is_empty());
    assert_eq!(second.response, first.response);
    assert!(second.actions.is_empty());
    assert!(unchanged);
}

#[test]
fn ordinary_captured_and_transformed_calls_share_one_exact_response() {
    let mut ordinary_first = UiTestHarness::new();
    ordinary_first.set_pointer_position(Point::new(10.0, 10.0));
    ordinary_first.pointer_press(MouseButton::Primary);
    ordinary_first.set_pointer_position(Point::new(14.0, 10.0));
    let ((ordinary, captured, unchanged), _) = ordinary_first.run_frame(|ui| {
        let id = ui.id("drag");
        let ordinary = {
            let (input, memory) = ui.input_and_memory_mut();
            draggable_transformed(id, FULL, Transform::IDENTITY, input, memory, false)
        };
        let after_first = ui.memory().clone();
        let captured = ui.captured_domain_drag_gesture(id, MISS, true);
        (ordinary, captured, after_first == *ui.memory())
    });
    assert_eq!(captured.response, ordinary);
    assert!(captured.actions.is_empty());
    assert!(unchanged);

    let mut captured_first = UiTestHarness::new();
    captured_first.set_pointer_position(Point::new(10.0, 10.0));
    captured_first.pointer_press(MouseButton::Primary);
    captured_first.set_pointer_position(Point::new(14.0, 10.0));
    let ((captured, ordinary, unchanged), _) = captured_first.run_frame(|ui| {
        let id = ui.id("drag");
        let captured = ui.captured_domain_drag_gesture(id, FULL, false);
        let after_first = ui.memory().clone();
        let ordinary = {
            let (input, memory) = ui.input_and_memory_mut();
            draggable_transformed(id, MISS, Transform::IDENTITY, input, memory, true)
        };
        (captured, ordinary, after_first == *ui.memory())
    });
    assert_eq!(ordinary, captured.response);
    assert!(!captured.actions.is_empty());
    assert!(unchanged);
}

#[test]
fn disabled_first_and_next_frame_reset_are_deterministic() {
    let mut harness = UiTestHarness::new();
    harness.set_pointer_position(Point::new(10.0, 10.0));
    harness.pointer_press(MouseButton::Primary);
    let (first, _) = harness.run_frame(|ui| {
        let id = ui.id("drag");
        let disabled = {
            let (input, memory) = ui.input_and_memory_mut();
            draggable(id, FULL, input, memory, true)
        };
        let captured = ui.captured_domain_drag_gesture(id, FULL, false);
        (disabled, captured)
    });
    assert!(first.0.state.disabled);
    assert_eq!(first.1.response, first.0);
    assert!(first.1.actions.is_empty());

    harness.pointer_press(MouseButton::Primary);
    let next = harness
        .run_frame(|ui| {
            let id = ui.id("drag");
            ui.captured_domain_drag_gesture(id, FULL, false)
        })
        .0;
    assert!(!next.response.state.disabled);
    assert_eq!(next.actions[0].phase, DomainDragGesturePhase::Press);
}

#[test]
fn unframed_calls_stay_uncached_and_runtime_end_closes_the_cache() {
    let id = WidgetId::from_key("drag");
    let input = UiInput::default();
    let mut memory = UiMemory::new();
    let first = draggable(id, FULL, &input, &mut memory, true);
    let second = draggable(id, MISS, &input, &mut memory, false);
    assert!(first.state.disabled);
    assert!(!second.state.disabled);
    assert_eq!(second.rect, MISS);

    let mut harness = UiTestHarness::new();
    let _ = harness.run_frame(|ui| {
        let id = ui.id("drag");
        ui.captured_domain_drag_gesture(id, FULL, true)
    });
    let pending = harness.input().clone();
    let after_end = draggable(id, MISS, &pending, harness.memory_mut(), false);
    assert!(!after_end.state.disabled);
    assert_eq!(after_end.rect, MISS);
}

#[test]
fn claims_are_per_owner_and_independent_from_selection() {
    let mut harness = UiTestHarness::new();
    harness.set_pointer_position(Point::new(10.0, 10.0));
    harness.pointer_press(MouseButton::Primary);
    let ((missed, hit), _) = harness.run_frame(|ui| {
        let missed_id = ui.id("missed");
        let hit_id = ui.id("hit");
        let missed = ui.captured_domain_drag_gesture(missed_id, MISS, false);
        let hit = ui.captured_domain_drag_gesture(hit_id, FULL, false);
        (missed, hit)
    });
    assert!(missed.actions.is_empty());
    assert_eq!(hit.actions[0].phase, DomainDragGesturePhase::Press);

    let mut independent = UiTestHarness::new();
    independent.set_pointer_position(Point::new(10.0, 10.0));
    independent.pointer_press(MouseButton::Primary);
    let (domain, _) = independent.run_frame(|ui| {
        let id = ui.id("shared");
        let selection = ui.captured_selection_gesture(id, MISS, false);
        assert!(selection.actions.is_empty());
        ui.captured_domain_drag_gesture(id, FULL, false)
    });
    assert_eq!(domain.actions[0].phase, DomainDragGesturePhase::Press);
}

#[test]
fn captured_actions_do_not_change_planned_or_unplanned_drop_authority() {
    let mut planned = UiTestHarness::new();
    start_crossed_drag(&mut planned);
    planned.set_modifiers(CTRL);
    planned
        .input_mut()
        .push_event(UiInputEvent::Text(TextInputEvent::Commit("gap".to_owned())));
    planned.pointer_release(MouseButton::Primary);
    let ((gesture, drop), _) = planned.run_frame(|ui| {
        let source = ui.id("source");
        let target = ui.id("target");
        ui.resolve_pointer_targets(|plan| {
            plan.target(
                PointerTarget::new(source, FULL, PointerOrder::new(20)).domain_drag_source(),
            );
            plan.target(
                PointerTarget::new(target, FULL, PointerOrder::new(30))
                    .ordinary_owner(None)
                    .drop_owner(target),
            );
        })
        .expect("valid planned captured DomainDrag");
        let drop = {
            let (input, memory) = ui.input_and_memory_mut();
            drop_target(target, FULL, input, memory, false)
        };
        let gesture = ui.captured_domain_drag_gesture(source, FULL, false);
        (gesture, drop)
    });
    assert!(drop.dropped);
    assert_eq!(drop.source, Some(gesture.response.id));
    assert_eq!(release_actions(&gesture), vec![(Some(2), false)]);
    assert_eq!(gesture.actions.last().unwrap().modifiers, CTRL);

    let mut unplanned = UiTestHarness::new();
    start_crossed_drag(&mut unplanned);
    unplanned.pointer_release(MouseButton::Primary);
    let ((gesture, drop), _) = unplanned.run_frame(|ui| {
        let source = ui.id("source");
        let target = ui.id("target");
        let drop = {
            let (input, memory) = ui.input_and_memory_mut();
            drop_target(target, FULL, input, memory, false)
        };
        let gesture = ui.captured_domain_drag_gesture(source, FULL, false);
        (gesture, drop)
    });
    assert!(!drop.dropped);
    assert_eq!(drop.source, None);
    assert_eq!(release_actions(&gesture), vec![(Some(0), false)]);
}
