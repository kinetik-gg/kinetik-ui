//! Explicit application-owned timeline journey through the public Stern facade.

use stern::core::{ActionContext, ActionDescriptor, ActionInvocation, ActionSource};
use stern_demo::{DemoActionRegistry, DemoApplicationModel, DemoScenario, DemoTransportState};

#[test]
fn timeline_scenario_owns_stable_time_clip_keyframes_and_transport() {
    let default_model = DemoApplicationModel::new();
    let default_actions = DemoActionRegistry::new();
    assert_eq!(default_model.scenario(), DemoScenario::Default);
    assert!(default_model.timeline().keyframes().is_empty());
    assert!(!default_actions.transport_play_pause().state.visible);
    assert!(!default_actions.transport_stop().state.visible);

    let actions = DemoActionRegistry::for_scenario(DemoScenario::TimelineJourney);
    let mut model = DemoApplicationModel::for_scenario(DemoScenario::TimelineJourney);
    let timeline = model.timeline();
    assert_eq!(timeline.frame_rate().numerator, 30);
    assert_eq!(timeline.frame_rate().denominator, 1);
    assert_eq!(timeline.frame_range(), (0, 240));
    assert_eq!(timeline.clip_id(), 1);
    assert_eq!(timeline.clip_label(), "Hero clip");
    assert_eq!(timeline.clip_frames(), (30, 90));
    assert_eq!(
        timeline
            .keyframes()
            .iter()
            .map(|keyframe| (keyframe.id(), keyframe.frame(), keyframe.label()))
            .collect::<Vec<_>>(),
        [
            (101, 36, "Position A"),
            (102, 60, "Position B"),
            (103, 84, "Position C"),
        ]
    );
    assert_eq!(model.timeline().position().frame(), 24);
    assert_eq!(model.timeline().position().time().seconds(), 0.8);
    assert_eq!(model.transport_state(), DemoTransportState::Stopped);
    assert!(actions.transport_play_pause().state.visible);
    assert!(actions.transport_stop().state.visible);

    assert!(model.execute(&invocation(actions.transport_play_pause())));
    assert_eq!(model.transport_state(), DemoTransportState::Playing);
    assert!(model.execute(&invocation(actions.transport_play_pause())));
    assert_eq!(model.transport_state(), DemoTransportState::Paused);
    assert!(model.execute(&invocation(actions.transport_play_pause())));
    assert_eq!(model.transport_state(), DemoTransportState::Playing);
    assert!(model.execute(&invocation(actions.transport_stop())));
    assert_eq!(model.transport_state(), DemoTransportState::Stopped);
}

fn invocation(action: &ActionDescriptor) -> ActionInvocation {
    ActionInvocation::new(
        action.id.clone(),
        ActionSource::Button,
        ActionContext::Editor,
    )
}
