//! Public-consumer baseline for the Stern integration demo.

use stern::UiState;
use stern::core::{
    ActionContext, ActionDescriptor, ActionInvocation, FrameContext, FrameOutput, PhysicalSize,
    PlatformRequest, Rect, ScaleFactor, SemanticRole, Size, TimeInfo, UiInput, ViewportInfo,
    WidgetId, default_dark_theme,
};
use stern::render::RenderResources;
use stern::text::TextEditState;

/// Canonical integration-demo title.
pub const DEMO_TITLE: &str = "Stern Integration Demo";

const EDIT_ACTION: &str = "workspace.edit";
const GRAPH_ACTION: &str = "workspace.graph";
const APPLY_ACTION: &str = "shared.apply";

/// Connected workspaces exposed by the phase-zero public consumer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoWorkspace {
    /// Document editing controls.
    Edit,
    /// Graph-oriented shared-state controls.
    Graph,
}

/// Application-owned state composed exclusively through the public `stern` facade.
pub struct DemoApp {
    ui_state: UiState,
    workspace: DemoWorkspace,
    document_name: TextEditState,
    applied_revision: u32,
}

impl DemoApp {
    /// Creates the deterministic baseline fixture.
    #[must_use]
    pub fn new() -> Self {
        Self {
            ui_state: UiState::new(),
            workspace: DemoWorkspace::Edit,
            document_name: TextEditState::new("Untitled Stern Document"),
            applied_revision: 0,
        }
    }

    /// Returns the active application workspace.
    #[must_use]
    pub const fn workspace(&self) -> DemoWorkspace {
        self.workspace
    }

    /// Returns the application-owned shared revision.
    #[must_use]
    pub const fn applied_revision(&self) -> u32 {
        self.applied_revision
    }

    /// Builds and dispatches one frame through public toolkit APIs.
    pub fn frame(&mut self, context: FrameContext) -> FrameOutput {
        let edit = ActionDescriptor::new(EDIT_ACTION, "Edit Workspace");
        let graph = ActionDescriptor::new(GRAPH_ACTION, "Graph Workspace");
        let apply = ActionDescriptor::new(APPLY_ACTION, "Apply Shared State");
        let workspace = self.workspace;
        let theme = default_dark_theme();
        let output = {
            let mut ui = self.ui_state.begin_frame(context, &theme);
            ui.push_platform_request(PlatformRequest::SetWindowTitle(DEMO_TITLE.to_owned()));
            ui.label(Rect::new(24.0, 20.0, 320.0, 24.0), DEMO_TITLE);
            let _ = ui.action_button(
                EDIT_ACTION,
                Rect::new(24.0, 56.0, 112.0, 30.0),
                &edit,
                ActionContext::Global,
            );
            let _ = ui.action_button(
                GRAPH_ACTION,
                Rect::new(148.0, 56.0, 120.0, 30.0),
                &graph,
                ActionContext::Global,
            );
            match workspace {
                DemoWorkspace::Edit => {
                    ui.label(Rect::new(24.0, 108.0, 180.0, 20.0), "Document name");
                    let _ = ui.text_field(
                        "document.name",
                        Rect::new(24.0, 136.0, 300.0, 30.0),
                        &mut self.document_name,
                        false,
                    );
                }
                DemoWorkspace::Graph => {
                    ui.label(Rect::new(24.0, 108.0, 280.0, 20.0), "Shared graph revision");
                }
            }
            let _ = ui.action_button(
                APPLY_ACTION,
                Rect::new(24.0, 188.0, 160.0, 30.0),
                &apply,
                ActionContext::Global,
            );
            ui.finish_output()
        };
        let mut actions = output.actions.clone();
        for invocation in actions.drain() {
            self.dispatch(&invocation);
        }
        output
    }

    /// Returns renderer resources for the latest public frame.
    #[must_use]
    pub fn render_resources(&self) -> RenderResources {
        self.ui_state.text_render_resources()
    }

    /// Returns the retained focused widget.
    #[must_use]
    pub fn focused(&self) -> Option<WidgetId> {
        self.ui_state.memory().focused()
    }

    fn dispatch(&mut self, invocation: &ActionInvocation) {
        match invocation.action_id.as_str() {
            EDIT_ACTION => self.workspace = DemoWorkspace::Edit,
            GRAPH_ACTION => self.workspace = DemoWorkspace::Graph,
            APPLY_ACTION => self.applied_revision = self.applied_revision.saturating_add(1),
            _ => {}
        }
    }
}

impl Default for DemoApp {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a deterministic frame context for tests and evidence capture.
#[must_use]
pub fn demo_context(input: UiInput) -> FrameContext {
    let logical = Size::new(720.0, 480.0);
    FrameContext::new(
        ViewportInfo::new(logical, PhysicalSize::new(720, 480), ScaleFactor::ONE),
        input,
        TimeInfo::default(),
    )
}

/// Reports whether output contains real component semantics.
#[must_use]
pub fn has_component_semantics(output: &FrameOutput) -> bool {
    let has_button = output
        .semantics
        .nodes()
        .iter()
        .any(|node| node.role == SemanticRole::Button);
    let has_text = output
        .semantics
        .nodes()
        .iter()
        .any(|node| node.role == SemanticRole::TextField);
    has_button && has_text
}
