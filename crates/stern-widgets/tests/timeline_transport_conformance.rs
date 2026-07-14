//! Windowless timeline transport action contract conformance tests.

use stern_core::{
    ActionContext, ActionDescriptor, ActionId, ActionSource, Rect, SemanticActionKind,
    SemanticRole, WidgetId,
};
use stern_widgets::{
    Menu, TimelineId, TimelineRange, TimelineTime, TimelineTransportContext, Toolbar, ToolbarGroup,
    ToolbarGroupId, TransportControlDescriptor, TransportControlId, TransportControlIntent,
    TransportControlKind, TransportControlSemanticRect, TransportControls,
    transport_control_semantics, transport_control_widget_id, transport_controls_semantics,
};

fn action(id: &str, label: &str) -> ActionDescriptor {
    ActionDescriptor::new(id, label)
}

fn hidden_action(id: &str) -> ActionDescriptor {
    let mut action = ActionDescriptor::new(id, "Hidden");
    action.state.visible = false;
    action
}

fn disabled_action(id: &str, label: &str) -> ActionDescriptor {
    let mut action = ActionDescriptor::new(id, label);
    action.state.enabled = false;
    action
}

fn control(
    raw: u64,
    intent: TransportControlIntent,
    action: ActionDescriptor,
) -> TransportControlDescriptor {
    TransportControlDescriptor::new(TransportControlId::from_raw(raw), intent, action)
}

#[test]
fn transport_visible_controls_preserve_descriptor_order_and_omit_hidden_actions() {
    let controls = TransportControls::from_controls([
        control(
            1,
            TransportControlIntent::JumpToStart,
            action("jump.start", "Start"),
        ),
        control(
            2,
            TransportControlIntent::PlayPause,
            hidden_action("play.pause.hidden"),
        ),
        control(3, TransportControlIntent::Stop, action("stop", "Stop")),
        control(
            4,
            TransportControlIntent::StepForward,
            disabled_action("step.forward", "Step Forward"),
        ),
    ]);

    let visible = controls.visible_controls();

    assert_eq!(visible.len(), 3);
    assert_eq!(visible[0].id, TransportControlId::from_raw(1));
    assert_eq!(visible[0].action_id(), &ActionId::new("jump.start"));
    assert_eq!(visible[1].id, TransportControlId::from_raw(3));
    assert_eq!(visible[2].id, TransportControlId::from_raw(4));
    assert!(!visible[2].enabled());
}

#[test]
fn transport_disabled_and_hidden_actions_do_not_emit_requests() {
    let controls = TransportControls::from_controls([
        control(
            1,
            TransportControlIntent::PlayPause,
            action("play.pause", "Play"),
        ),
        control(
            2,
            TransportControlIntent::Stop,
            disabled_action("stop", "Stop"),
        ),
        control(
            3,
            TransportControlIntent::StepBackward,
            hidden_action("step.back.hidden"),
        ),
    ]);

    let enabled = controls
        .request_for_visible(0, ActionSource::Button, None)
        .expect("enabled request");

    assert_eq!(enabled.action_id, ActionId::new("play.pause"));
    assert_eq!(enabled.source, ActionSource::Button);
    assert_eq!(
        enabled.control_kind,
        TransportControlIntent::PlayPause.default_control_kind()
    );
    assert_eq!(
        enabled.action_invocation(ActionContext::Global).action_id,
        ActionId::new("play.pause")
    );
    assert_eq!(controls.visible_controls().len(), 2);
    assert_eq!(
        controls.request_for_visible(1, ActionSource::Button, None),
        None
    );
    assert_eq!(
        controls.request_for_control(TransportControlId::from_raw(3), ActionSource::Button, None),
        None
    );
}

#[test]
fn transport_checked_toggle_state_is_presentation_metadata_only() {
    let mut loop_action = action("transport.loop", "Loop");
    loop_action.state.checked = Some(true);
    let control = control(7, TransportControlIntent::LoopToggle, loop_action);

    assert_eq!(control.control_kind, TransportControlKind::Toggle);
    assert_eq!(control.checked(), Some(true));
    assert!(control.action.state.is_checked());

    let request = control
        .request(ActionSource::Button, None)
        .expect("toggle request");
    assert_eq!(request.action_id, ActionId::new("transport.loop"));
    assert_eq!(request.intent, TransportControlIntent::LoopToggle);
    assert_eq!(request.control_kind, TransportControlKind::Toggle);
}

#[test]
fn transport_request_metadata_preserves_action_source_kind_and_timeline_context() {
    let control = control(
        9,
        TransportControlIntent::RangePlaybackToggle,
        action("preview.range", "Range Playback"),
    )
    .with_control_kind(TransportControlKind::Toggle);
    let context = TimelineTransportContext::new(TimelineId::from_raw(42))
        .with_playhead_time(TimelineTime::from_seconds(2.5))
        .with_selection_range(TimelineRange::seconds(8.0, 3.0));

    let request = control
        .request(ActionSource::Programmatic, Some(context))
        .expect("range request");

    assert_eq!(request.action_id, ActionId::new("preview.range"));
    assert_eq!(request.source, ActionSource::Programmatic);
    assert_eq!(request.control_kind, TransportControlKind::Toggle);
    assert_eq!(
        request.timeline_context.expect("timeline context"),
        TimelineTransportContext::new(TimelineId::from_raw(42))
            .with_playhead_time(TimelineTime::from_seconds(2.5))
            .with_selection_range(TimelineRange::seconds(3.0, 8.0))
    );
}

#[test]
fn transport_semantics_expose_button_toggle_state_and_action_identity() {
    let play = control(
        1,
        TransportControlIntent::PlayPause,
        action("play.pause", "Play"),
    );
    let mut loop_action = action("loop", "Loop");
    loop_action.tooltip = Some("Repeat playback".to_owned());
    loop_action.state.checked = Some(true);
    let looping = control(2, TransportControlIntent::LoopToggle, loop_action);
    let disabled = control(
        3,
        TransportControlIntent::Stop,
        disabled_action("stop", "Stop"),
    );
    let root = WidgetId::from_key("transport");

    let play_node = transport_control_semantics(root, Rect::new(0.0, 0.0, 24.0, 24.0), &play)
        .expect("play semantics");
    let loop_node = transport_control_semantics(root, Rect::new(28.0, 0.0, 24.0, 24.0), &looping)
        .expect("loop semantics");
    let disabled_node =
        transport_control_semantics(root, Rect::new(56.0, 0.0, 24.0, 24.0), &disabled)
            .expect("disabled semantics");

    assert_eq!(play_node.role, SemanticRole::Button);
    assert!(play_node.focusable);
    assert!(play_node.actions.iter().any(|action| {
        action.kind == SemanticActionKind::Invoke
            && action.action_id == Some(ActionId::new("play.pause"))
    }));
    assert_eq!(loop_node.role, SemanticRole::Toggle);
    assert_eq!(loop_node.state.checked, Some(true));
    assert!(loop_node.state.selected);
    assert_eq!(loop_node.description.as_deref(), Some("Repeat playback"));
    assert!(disabled_node.state.disabled);
    assert!(!disabled_node.focusable);
    assert!(disabled_node.actions.is_empty());
}

#[test]
fn transport_semantic_group_uses_visible_control_order_and_stable_ids() {
    let controls = TransportControls::from_controls([
        control(
            1,
            TransportControlIntent::JumpToStart,
            action("start", "Start"),
        ),
        control(2, TransportControlIntent::Stop, hidden_action("hidden")),
        control(3, TransportControlIntent::JumpToEnd, action("end", "End")),
    ]);
    let root = WidgetId::from_key("transport-root");

    let semantics = transport_controls_semantics(
        root,
        Rect::new(0.0, 0.0, 96.0, 24.0),
        "Transport",
        &controls,
        [
            TransportControlSemanticRect::new(
                TransportControlId::from_raw(1),
                Rect::new(0.0, 0.0, 24.0, 24.0),
            ),
            TransportControlSemanticRect::new(
                TransportControlId::from_raw(3),
                Rect::new(28.0, 0.0, 24.0, 24.0),
            ),
        ],
    );

    assert_eq!(
        semantics[0].role,
        SemanticRole::Custom("transport-controls".to_owned())
    );
    assert_eq!(
        semantics[0].children,
        vec![
            transport_control_widget_id(root, TransportControlId::from_raw(1)),
            transport_control_widget_id(root, TransportControlId::from_raw(3)),
        ]
    );
    assert_eq!(semantics.len(), 3);
    assert_eq!(semantics[1].label.as_deref(), Some("Start"));
    assert_eq!(semantics[2].label.as_deref(), Some("End"));
}

#[test]
fn transport_descriptors_reuse_action_surface_contracts_without_command_duplication() {
    let mut loop_action = action("transport.loop", "Loop");
    loop_action.state.checked = Some(true);
    let stop = disabled_action("transport.stop", "Stop");
    let transport = TransportControls::from_controls([
        control(1, TransportControlIntent::LoopToggle, loop_action.clone()),
        control(2, TransportControlIntent::Stop, stop.clone()),
    ]);
    let toolbar = Toolbar::from_groups([ToolbarGroup::from_actions(
        ToolbarGroupId::from_raw(1),
        "Transport",
        [loop_action.clone(), stop.clone()],
    )]);
    let menu = Menu::from_actions([loop_action, stop]);

    let visible_transport = transport.visible_controls();
    let visible_toolbar = toolbar.visible_groups()[0].visible_items();
    let visible_menu = menu.visible_items();

    assert_eq!(
        visible_transport[0].action_id(),
        visible_toolbar[0].action_id()
    );
    assert_eq!(visible_transport[0].checked(), visible_toolbar[0].checked());
    assert_eq!(visible_transport[0].checked(), Some(true));
    assert_eq!(visible_menu.len(), 2);
    assert_eq!(
        transport.request_for_visible(0, ActionSource::Button, None),
        Some(stern_widgets::TransportActionRequest::new(
            ActionId::new("transport.loop"),
            TransportControlIntent::LoopToggle,
            ActionSource::Button,
            TransportControlKind::Toggle,
            None,
        ))
    );
    assert_eq!(
        transport.request_for_visible(1, ActionSource::Button, None),
        None
    );
}
