use super::evidence::{
    IMPLEMENTED_TAXONOMY_EVIDENCE, PARTIAL_TAXONOMY_EVIDENCE, S12_VIEWPORT_ACTION_ROUTING_EVIDENCE,
    S12_VIEWPORT_EVIDENCE, S12_VIEWPORT_TOOLS_EVIDENCE, S13_DIAGNOSTIC_STRIP_EVIDENCE,
    S13_FEEDBACK_STACK_EVIDENCE, S13_JOB_LIST_EVIDENCE, S13_PROGRESS_EVIDENCE,
    STAGE_10_PARTIAL_EVIDENCE, STAGE_11_PARTIAL_EVIDENCE, STAGE_13_PARTIAL_EVIDENCE,
};
use super::types::{ComponentCategory, ComponentConformanceStatus, ComponentMetadata};

use ComponentCategory::{
    Collection, Control, Display, Docking, Input, Inspector, Overlay, System, TextEditing, Viewport,
};
use ComponentConformanceStatus::{Implemented, Partial};

/// Data-only registry of Kinetik widget components and editor patterns.
pub const COMPONENT_METADATA: &[ComponentMetadata] = &[
    ComponentMetadata::new("Label", "label", Display, Implemented)
        .with_evidence(IMPLEMENTED_TAXONOMY_EVIDENCE),
    ComponentMetadata::new("Image", "image", Display, Implemented)
        .with_evidence(IMPLEMENTED_TAXONOMY_EVIDENCE),
    ComponentMetadata::new("Separator", "separator", Display, Implemented)
        .with_evidence(IMPLEMENTED_TAXONOMY_EVIDENCE),
    ComponentMetadata::new("Button", "button", Control, Implemented)
        .with_evidence(IMPLEMENTED_TAXONOMY_EVIDENCE),
    ComponentMetadata::new("IconButton", "icon-button", Control, Implemented)
        .with_evidence(IMPLEMENTED_TAXONOMY_EVIDENCE),
    ComponentMetadata::new("Checkbox", "checkbox", Input, Implemented)
        .with_evidence(IMPLEMENTED_TAXONOMY_EVIDENCE),
    ComponentMetadata::new("RadioButton", "radio-button", Input, Implemented)
        .with_evidence(IMPLEMENTED_TAXONOMY_EVIDENCE),
    ComponentMetadata::new("Toggle", "toggle", Input, Implemented)
        .with_evidence(IMPLEMENTED_TAXONOMY_EVIDENCE),
    ComponentMetadata::new("Slider", "slider", Input, Implemented)
        .with_evidence(IMPLEMENTED_TAXONOMY_EVIDENCE),
    ComponentMetadata::new("NumericInput", "numeric-input", Input, Implemented)
        .with_evidence(IMPLEMENTED_TAXONOMY_EVIDENCE),
    ComponentMetadata::new(
        "NumericScrubInput",
        "numeric-scrub-input",
        Input,
        Implemented,
    )
    .with_evidence(IMPLEMENTED_TAXONOMY_EVIDENCE),
    ComponentMetadata::new("TextField", "text-field", TextEditing, Implemented)
        .with_evidence(IMPLEMENTED_TAXONOMY_EVIDENCE),
    ComponentMetadata::new(
        "MultiLineTextField",
        "multi-line-text-field",
        TextEditing,
        Implemented,
    )
    .with_evidence(IMPLEMENTED_TAXONOMY_EVIDENCE),
    ComponentMetadata::new("SearchField", "search-field", TextEditing, Implemented)
        .with_evidence(IMPLEMENTED_TAXONOMY_EVIDENCE),
    ComponentMetadata::new("List", "list", Collection, Partial)
        .with_evidence(PARTIAL_TAXONOMY_EVIDENCE),
    ComponentMetadata::new("Grid", "grid", Collection, Partial)
        .with_evidence(PARTIAL_TAXONOMY_EVIDENCE),
    ComponentMetadata::new("Table", "table", Collection, Partial)
        .with_evidence(PARTIAL_TAXONOMY_EVIDENCE),
    ComponentMetadata::new("Tree", "tree", Collection, Partial)
        .with_evidence(PARTIAL_TAXONOMY_EVIDENCE),
    ComponentMetadata::new("Outliner", "outliner", Collection, Partial)
        .with_stage(10)
        .with_evidence(STAGE_10_PARTIAL_EVIDENCE),
    ComponentMetadata::new("AssetBrowser", "asset-browser", Collection, Partial)
        .with_stage(10)
        .with_evidence(STAGE_10_PARTIAL_EVIDENCE),
    ComponentMetadata::new("PropertyGrid", "property-grid", Inspector, Partial)
        .with_evidence(PARTIAL_TAXONOMY_EVIDENCE),
    ComponentMetadata::new(
        "PropertyAffordanceControls",
        "property-affordance-controls",
        Inspector,
        Implemented,
    )
    .with_evidence(IMPLEMENTED_TAXONOMY_EVIDENCE),
    ComponentMetadata::new("Vector2Field", "vector-two-field", Inspector, Implemented)
        .with_evidence(IMPLEMENTED_TAXONOMY_EVIDENCE),
    ComponentMetadata::new("Vector3Field", "vector-three-field", Inspector, Implemented)
        .with_evidence(IMPLEMENTED_TAXONOMY_EVIDENCE),
    ComponentMetadata::new("Vector4Field", "vector-four-field", Inspector, Implemented)
        .with_evidence(IMPLEMENTED_TAXONOMY_EVIDENCE),
    ComponentMetadata::new("ColorField", "color-field", Inspector, Implemented)
        .with_evidence(IMPLEMENTED_TAXONOMY_EVIDENCE),
    ComponentMetadata::new("SelectField", "select-field", Inspector, Implemented)
        .with_evidence(IMPLEMENTED_TAXONOMY_EVIDENCE),
    ComponentMetadata::new("AssetSlotField", "asset-slot-field", Inspector, Implemented)
        .with_evidence(IMPLEMENTED_TAXONOMY_EVIDENCE),
    ComponentMetadata::new("PathField", "path-field", Inspector, Implemented)
        .with_evidence(IMPLEMENTED_TAXONOMY_EVIDENCE),
    ComponentMetadata::new("Panel", "panel", Docking, Partial)
        .with_evidence(PARTIAL_TAXONOMY_EVIDENCE),
    ComponentMetadata::new("Frame", "frame", Docking, Partial)
        .with_evidence(PARTIAL_TAXONOMY_EVIDENCE),
    ComponentMetadata::new("Dock", "dock", Docking, Partial)
        .with_evidence(PARTIAL_TAXONOMY_EVIDENCE),
    ComponentMetadata::new("Menu", "menu", Overlay, Partial)
        .with_evidence(PARTIAL_TAXONOMY_EVIDENCE),
    ComponentMetadata::new("MenuItem", "menu-item", Overlay, Partial)
        .with_evidence(PARTIAL_TAXONOMY_EVIDENCE),
    ComponentMetadata::new("ContextMenu", "context-menu", Overlay, Partial)
        .with_evidence(PARTIAL_TAXONOMY_EVIDENCE),
    ComponentMetadata::new("Popover", "popover", Overlay, Partial)
        .with_evidence(PARTIAL_TAXONOMY_EVIDENCE),
    ComponentMetadata::new("Tooltip", "tooltip", Overlay, Partial)
        .with_evidence(PARTIAL_TAXONOMY_EVIDENCE),
    ComponentMetadata::new("CommandPalette", "command-palette", Overlay, Partial)
        .with_evidence(PARTIAL_TAXONOMY_EVIDENCE),
    ComponentMetadata::new("Viewport", "viewport", Viewport, Partial)
        .with_stage(12)
        .with_evidence(S12_VIEWPORT_EVIDENCE),
    ComponentMetadata::new("ViewportTools", "viewport-tools", Viewport, Partial)
        .with_stage(12)
        .with_evidence(S12_VIEWPORT_TOOLS_EVIDENCE),
    ComponentMetadata::new(
        "ViewportActionRouting",
        "viewport-action-routing",
        Viewport,
        Partial,
    )
    .with_stage(12)
    .with_evidence(S12_VIEWPORT_ACTION_ROUTING_EVIDENCE),
    ComponentMetadata::new("NodeGraph", "node-graph", Viewport, Partial)
        .with_evidence(PARTIAL_TAXONOMY_EVIDENCE),
    ComponentMetadata::new("Ruler", "ruler", Viewport, Partial)
        .with_stage(11)
        .with_evidence(STAGE_11_PARTIAL_EVIDENCE),
    ComponentMetadata::new("Dropdown", "dropdown", Overlay, Partial)
        .with_evidence(PARTIAL_TAXONOMY_EVIDENCE),
    ComponentMetadata::new("MenuBar", "menu-bar", Overlay, Partial)
        .with_evidence(PARTIAL_TAXONOMY_EVIDENCE),
    ComponentMetadata::new("Tabs", "tabs", Docking, Partial)
        .with_evidence(PARTIAL_TAXONOMY_EVIDENCE),
    ComponentMetadata::new("Toolbar", "toolbar", System, Partial)
        .with_evidence(PARTIAL_TAXONOMY_EVIDENCE),
    ComponentMetadata::new("StatusBar", "status-bar", System, Partial)
        .with_stage(13)
        .with_evidence(STAGE_13_PARTIAL_EVIDENCE),
    ComponentMetadata::new("Modal", "modal", Overlay, Partial)
        .with_evidence(PARTIAL_TAXONOMY_EVIDENCE),
    ComponentMetadata::new("Timeline", "timeline", Viewport, Partial)
        .with_stage(11)
        .with_evidence(STAGE_11_PARTIAL_EVIDENCE),
    ComponentMetadata::new("TransportControls", "transport-controls", Control, Partial)
        .with_stage(11)
        .with_evidence(STAGE_11_PARTIAL_EVIDENCE),
    ComponentMetadata::new("ProgressIndicator", "progress-indicator", Display, Partial)
        .with_stage(13)
        .with_evidence(S13_PROGRESS_EVIDENCE),
    ComponentMetadata::new("JobList", "job-list", System, Partial)
        .with_stage(13)
        .with_evidence(S13_JOB_LIST_EVIDENCE),
    ComponentMetadata::new("DiagnosticStrip", "diagnostic-strip", System, Partial)
        .with_stage(13)
        .with_evidence(S13_DIAGNOSTIC_STRIP_EVIDENCE),
    ComponentMetadata::new("FeedbackStack", "feedback-stack", System, Partial)
        .with_stage(13)
        .with_evidence(S13_FEEDBACK_STACK_EVIDENCE),
];
