//! DCC-style editor showcase surface.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::too_many_lines
)]

mod showcase;
#[cfg(test)]
mod tests;

use std::time::Duration;

use kinetik_ui::core::{
    ActionContext, ActionDescriptor, ActionIcon, ActionId, ActionInvocation, ActionQueue,
    ActionSource, Axis, Brush, Color, CornerRadius, CursorShape, ImagePrimitive, Key, KeyState,
    Modifiers, PlatformRequest, Point, Primitive, Rect, RectPrimitive, RepaintRequest, Response,
    Shortcut, Size, Stroke, TextPrimitive, TextureId, Theme, Vec2, WidgetId,
};
use kinetik_ui::render::{
    ImageAtlasRegion, ImageResource, RenderImage, RenderImageSampling, RenderResources,
    TextureResource,
};
use kinetik_ui::text::TextEditState;
use kinetik_ui::widgets::{
    AssetSlotAsset, AssetSlotConfig, DiagnosticSource, DiagnosticStrip, DiagnosticStripItem,
    DiagnosticStripItemId, DiagnosticStripSeverity, Dock, DockChromeStyle, DockInteractionPolicy,
    DockNode, DockSplitterContextActionKind, DropdownItem, DropdownItemId, DropdownModel,
    EdgeDescriptor, EdgeId, FeedbackAction, FeedbackDismiss, FeedbackId, FeedbackItem,
    FeedbackKind, FeedbackStack, Frame, FrameId, GraphPoint, GraphRect, GraphVector, ItemId,
    JobCancel, JobList, JobPhase, JobProgress, JobRow, JobRowId, ListLayout, Menu, MenuBar,
    MenuBarMenu, MenuBarMenuId, MenuBarOverlayRequest, MenuItem, MenuOverlay, ModalAction,
    ModalActionRole, ModalDialog, ModalDialogOverlay, NodeDescriptor, NodeFrameDescriptor,
    NodeFrameId, NodeGraphDescriptor, NodeGraphEdgeRoutePoint, NodeGraphEmissionError,
    NodeGraphPanZoom, NodeGraphSelection, NodeGraphSelectionTarget, NodeGraphStaticOutput,
    NodeGraphStaticView, NodeGraphViewport, NodeGroupDescriptor, NodeGroupId, NodeId,
    NumericScrubInputConfig, OverlayDismissal, OverlayId, OverlayKind, OverlayStack, PanZoom,
    Panel, PanelId, PanelInstanceId, PanelInstancePolicy, PanelInstanceSnapshot,
    PanelOpenActionMetadata, PanelOpenDecision, PanelRegistry, PanelTypeCategory,
    PanelTypeDescriptor, PanelTypeId, PanelWorkspaceContext, PathFieldConfig, PopoverPlacement,
    PortDescriptor, PortDirection, PortEndpoint, PortId, PortTypeId, PropertyGridLayout,
    PropertyGridRow, PropertyGridRowStatus, RerouteDescriptor, RerouteId, SelectFieldConfig,
    StatusBar, StatusItem, StatusItemId, StatusItemKind, StatusProgress, TableColumn, TableLayout,
    Toolbar, ToolbarGroup, ToolbarGroupId, ToolbarItem, ToolbarItemPresentation, TreeExpansion, Ui,
    VectorScrubInputConfig, ViewportFit, WorkspaceSnapshot, classify_numeric_input_draft,
    icon_button_semantics, resolve_dock_splitter_context_actions_with_policy, solve_dock_layout,
    solve_dock_splitters_with_style,
};
#[cfg(test)]
use kinetik_ui::widgets::{FrameTab, TabStrip, frame_tabs};

include!("editor/workflow.rs");
include!("editor/root_state.rs");
include!("editor/resources.rs");
include!("editor/models.rs");
include!("editor/fixtures_paint.rs");
