//! Builds one deterministic UI frame through the application-facing facade.

use std::time::Duration;

use kinetik_ui::prelude::*;

fn main() {
    let theme = default_dark_theme();
    let viewport = ViewportInfo::new(
        Size::new(960.0, 540.0),
        PhysicalSize::new(1920, 1080),
        ScaleFactor::new(2.0),
    );
    let context = FrameContext::new(
        viewport,
        UiInput::default(),
        TimeInfo::new(Duration::ZERO, Duration::from_millis(16), 0),
    );

    let mut state = UiState::new();
    let mut query = TextEditState::new("media");
    let mut amount = 0.45;
    let run_action = ActionDescriptor::new("run", "Run");

    let mut ui = state.begin_frame(context, &theme);
    ui.panel(Rect::new(24.0, 24.0, 360.0, 184.0));
    ui.label(Rect::new(40.0, 40.0, 240.0, 24.0), "Kinetik UI");
    let run = ui
        .action_button(
            "run",
            Rect::new(40.0, 72.0, 96.0, 30.0),
            &run_action,
            ActionContext::Global,
        )
        .expect("run action is visible");
    let scrub = ui.slider(
        "amount",
        Rect::new(40.0, 120.0, 220.0, 16.0),
        &mut amount,
        0.0..=1.0,
        false,
    );
    let search = ui.search_field(
        "search",
        Rect::new(40.0, 152.0, 240.0, 28.0),
        &mut query,
        false,
    );
    let output = ui.finish_output();

    assert!(!run.clicked);
    assert_eq!(scrub.rect, Rect::new(40.0, 120.0, 220.0, 16.0));
    assert_eq!(search.query, "media");
    assert!(!output.primitives.is_empty());
    assert!(output.semantics.validate().is_ok());
    assert!(output.warnings.is_empty());
    assert!(!state.text_layouts().is_empty());
}
