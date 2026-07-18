use stern::core::{
    ActionContext, ActionInvocation, ActionSource, Axis, Key, KeyState, PointerOrder,
    PointerTarget, Rect, Size, TextureId, UiInput, WidgetId,
};
use stern::render::{RenderImage, RenderImageSampling, RenderResources, TextureResource};
use stern::widgets::asset_browser::{
    AssetBrowserConfig, AssetBrowserItem, AssetBrowserLayout, AssetBrowserModel,
    AssetBrowserRequest, AssetBrowserState, AssetBrowserViewMode,
};
use stern::widgets::dock::{DockScene, DockSceneConfig};
use stern::widgets::inspector::PropertyGridConfig;
use stern::widgets::{
    ChromeScene, ChromeSceneConfig, ChromeSceneIntent, ChromeSceneItemKey, CommandPaletteOverlay,
    Dock, DockNode, Frame, FrameId, FrameTab, GridColumns, GridLayout, InlineEditDraftDisposition,
    InlineEditDraftPolicy, InlineEditFocusLossPolicy, InlineEditRequest, ItemId, ListLayout, Menu,
    MenuBar, MenuBarMenu, MenuBarMenuId, MenuBarOverlayRequest, MenuOverlay, OverlayDismissal,
    OverlayId, OverlayKind, OverlayScene, OverlaySceneIntent, OverlaySceneSurface, PanZoom, Panel,
    PanelId, PopoverPlacement, PropertyGridRow, StatusBar, StatusItem, StatusItemId,
    StatusItemKind, TabStrip, Toolbar, ToolbarGroup, ToolbarGroupId, Ui, ViewportSurface,
    ViewportWidget, ViewportWidgetConfig,
};

use crate::{DemoActionRegistry, DemoWorkspace};

const ASSETS_PANEL: PanelId = PanelId::from_raw(11);
const VIEWPORT_PANEL: PanelId = PanelId::from_raw(21);
const INSPECTOR_PANEL: PanelId = PanelId::from_raw(31);
const VIEWPORT_TEXTURE: TextureId = TextureId::from_raw(1);
const TOOLBAR_GROUP: ToolbarGroupId = ToolbarGroupId::from_raw(1);
const APPLICATION_MENU: MenuBarMenuId = MenuBarMenuId::from_raw(1);
const APPLICATION_MENU_OVERLAY: OverlayId = OverlayId::from_raw(1);
const CONTEXT_MENU_OVERLAY: OverlayId = OverlayId::from_raw(2);
const COMMAND_PALETTE_OVERLAY: OverlayId = OverlayId::from_raw(3);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AssetKind {
    Raster,
    Vector,
    Adjustment,
    Text,
}

impl AssetKind {
    const fn label(self) -> &'static str {
        match self {
            Self::Raster => "Raster layer",
            Self::Vector => "Vector layer",
            Self::Adjustment => "Adjustment layer",
            Self::Text => "Text layer",
        }
    }
}

#[derive(Debug, Clone)]
struct AssetRecord {
    id: ItemId,
    name: String,
    kind: AssetKind,
    selected: bool,
}

impl AssetRecord {
    fn new(id: u64, name: &str, kind: AssetKind) -> Self {
        Self {
            id: ItemId::from_raw(id),
            name: name.to_owned(),
            kind,
            selected: id == 1,
        }
    }
}

/// Retained public Stern state for the deterministic Edit workspace fixture.
pub(crate) struct EditWorkspace {
    dock: Dock,
    assets: Vec<AssetRecord>,
    asset_browser: AssetBrowserState,
    pan_zoom: PanZoom,
    texture: TextureResource,
    overlay: Option<OverlayScene>,
}

impl EditWorkspace {
    pub(crate) fn new() -> Self {
        let assets = asset_records();
        let model = asset_browser_model(&assets);
        let mut asset_browser = AssetBrowserState::new();
        asset_browser.selection.replace(assets[0].id);
        let _ = asset_browser
            .cursor
            .activate(&model.projection(), assets[0].id);

        Self {
            dock: edit_dock(),
            assets,
            asset_browser,
            pan_zoom: PanZoom::default(),
            texture: viewport_texture(),
            overlay: None,
        }
    }

    pub(crate) const fn has_overlay(&self) -> bool {
        self.overlay.is_some()
    }

    pub(crate) fn compose(
        &mut self,
        ui: &mut Ui<'_>,
        actions: &DemoActionRegistry,
        workspace: DemoWorkspace,
        revision: u32,
        bounds: Size,
    ) {
        let layout = WorkspaceLayout::new(bounds);
        let mut menu_bar = MenuBar::from_menus([MenuBarMenu::from_actions(
            APPLICATION_MENU,
            "Workspace",
            actions.iter().cloned(),
        )]);
        let toolbar = Toolbar::from_groups([ToolbarGroup::from_actions(
            TOOLBAR_GROUP,
            "Workspace actions",
            actions.iter().cloned(),
        )]);
        let tab_strip = TabStrip::from_tabs([
            workspace_tab(101, "Edit Workspace", workspace == DemoWorkspace::Edit),
            workspace_tab(102, "Graph Workspace", workspace == DemoWorkspace::Graph),
        ]);
        let status_bar = StatusBar::from_items([StatusItem::new(
            StatusItemId::from_raw(1),
            "Revision",
            format!("Applied revision {revision}"),
            StatusItemKind::Ready,
        )]);
        let chrome = ChromeScene::new(
            chrome_config(layout, actions),
            &menu_bar,
            &toolbar,
            &tab_strip,
            &status_bar,
        );
        let dock_scene = DockScene::new(
            DockSceneConfig::new(WidgetId::from_key("edit-workspace.dock"), layout.dock),
            &self.dock,
        );

        let assets_bounds = panel_bounds(&dock_scene, ASSETS_PANEL).map(|rect| rect.inset(8.0));
        let viewport_bounds = panel_bounds(&dock_scene, VIEWPORT_PANEL).map(|rect| rect.inset(8.0));
        let asset_model = asset_browser_model(&self.assets);
        let asset_browser =
            prepare_asset_browser(ui, assets_bounds, &asset_model, &self.asset_browser);
        let viewport = viewport_bounds.map(|rect| {
            ui.prepare_viewport_widget(ViewportWidgetConfig::new(
                WidgetId::from_key("edit-workspace.viewport"),
                ViewportSurface {
                    texture: VIEWPORT_TEXTURE,
                    source_size: Size::new(1280.0, 720.0),
                    bounds: rect,
                    pan_zoom: self.pan_zoom,
                },
            ))
        });

        open_palette_if_requested(&mut self.overlay, ui.input(), actions, bounds);
        let context_target = ui.make_id("edit-workspace.shared-action-context");
        let route_context_pointer = secondary_route_active(ui.input());

        ui.resolve_pointer_targets(|plan| {
            let mut next = dock_scene.declare_pointer_targets_with_content(
                plan,
                PointerOrder::new(0),
                |plan, mut next| {
                    if let Some(asset_browser) = &asset_browser {
                        next =
                            asset_browser.declare_pointer_targets(plan, next, &self.asset_browser);
                    }
                    if let Some(viewport) = &viewport {
                        next = viewport.declare_pointer_targets(plan, next);
                    }
                    next
                },
            );
            if route_context_pointer && let Some(rect) = viewport_bounds {
                plan.target(PointerTarget::new(context_target, rect, next));
                next = PointerOrder::new(next.raw() + 1);
            }
            next = chrome.declare_pointer_targets(plan, next);
            if let Some(overlay) = &self.overlay {
                overlay.declare_pointer_targets(plan, next);
            }
        })
        .expect("Edit workspace pointer targets are valid");

        compose_workspace_panels(
            ui,
            &dock_scene,
            asset_browser.as_ref(),
            viewport.as_ref(),
            &mut self.asset_browser,
            &mut self.assets,
            &mut self.pan_zoom,
        );
        let context_requested = shared_context_requested(ui, viewport_bounds);
        let chrome_output = ui.chrome_scene(&chrome);
        route_workspace_tabs(ui, actions, &chrome_output.intents);
        self.reconcile_overlay(
            ui,
            actions,
            &mut menu_bar,
            &chrome_output.intents,
            context_requested,
            bounds,
        );
    }

    fn reconcile_overlay(
        &mut self,
        ui: &mut Ui<'_>,
        actions: &DemoActionRegistry,
        menu_bar: &mut MenuBar,
        chrome_intents: &[ChromeSceneIntent],
        context_requested: bool,
        bounds: Size,
    ) {
        let close_overlay = self.overlay.as_mut().is_some_and(|overlay| {
            ui.overlay_scene(overlay).intents.iter().any(|intent| {
                matches!(
                    intent,
                    OverlaySceneIntent::Action(_) | OverlaySceneIntent::Dismiss(_)
                )
            })
        });
        if close_overlay {
            self.overlay = None;
        }
        if self.overlay.is_none() {
            if let Some((menu, anchor)) = chrome_intents.iter().find_map(|intent| {
                let ChromeSceneIntent::OpenMenu { menu, anchor } = intent else {
                    return None;
                };
                Some((*menu, *anchor))
            }) {
                let _ = menu_bar.open(menu);
                self.overlay = application_menu_scene(menu_bar, anchor, bounds);
            } else if context_requested {
                let anchor = ui
                    .input()
                    .pointer
                    .position
                    .map_or(Rect::new(0.0, 0.0, 1.0, 1.0), |point| {
                        Rect::new(point.x, point.y, 1.0, 1.0)
                    });
                self.overlay = Some(context_menu_scene(actions, anchor, bounds));
            }
        }
    }

    pub(crate) fn register_resources(&self, resources: &mut RenderResources) {
        resources.register_texture(self.texture.clone());
    }
}

fn open_palette_if_requested(
    overlay: &mut Option<OverlayScene>,
    input: &UiInput,
    actions: &DemoActionRegistry,
    bounds: Size,
) {
    if overlay.is_none() && command_palette_requested(input) {
        *overlay = Some(command_palette_scene(actions, bounds));
    }
}

#[allow(clippy::too_many_arguments)]
fn compose_workspace_panels(
    ui: &mut Ui<'_>,
    dock_scene: &DockScene,
    asset_browser: Option<&stern::widgets::asset_browser::AssetBrowserScene<'_>>,
    viewport: Option<&ViewportWidget>,
    asset_state: &mut AssetBrowserState,
    assets: &mut [AssetRecord],
    pan_zoom: &mut PanZoom,
) {
    let _ = ui.dock_scene(dock_scene, |ui, panel| match panel.panel {
        ASSETS_PANEL => {
            if let Some(asset_browser) = asset_browser {
                let output = ui.asset_browser(
                    asset_browser,
                    asset_state,
                    |target, draft| rename_conflict(assets, target, draft),
                    |_| Vec::new(),
                );
                apply_asset_browser_requests(assets, output.requests);
                project_selection(assets, asset_state);
            }
        }
        VIEWPORT_PANEL => {
            if let Some(viewport) = viewport {
                let output = ui.viewport_widget(viewport, pan_zoom, &[]);
                *pan_zoom = output.next_pan_zoom;
            }
        }
        INSPECTOR_PANEL => {
            let selected = assets.iter().find(|asset| asset.selected);
            inspector(ui, panel.rect.inset(8.0), selected);
        }
        _ => {}
    });
}

fn asset_records() -> Vec<AssetRecord> {
    use AssetKind::{Adjustment, Raster, Text, Vector};
    [
        (1, "Backdrop", Raster),
        (2, "Character", Vector),
        (3, "Lighting", Adjustment),
        (4, "Title", Text),
        (5, "Clouds", Raster),
        (6, "Foreground", Vector),
        (7, "Grade", Adjustment),
        (8, "Subtitle", Text),
        (9, "Mountains", Raster),
        (10, "Effects", Vector),
        (11, "Bloom", Adjustment),
        (12, "Credits", Text),
        (13, "Sky", Raster),
        (14, "Props", Vector),
        (15, "Contrast", Adjustment),
        (16, "Location", Text),
        (17, "Ground", Raster),
        (18, "Particles", Vector),
        (19, "Vignette", Adjustment),
        (20, "Watermark", Text),
        (21, "Reflections", Raster),
        (22, "Guides", Vector),
        (23, "Exposure", Adjustment),
        (24, "Notes", Text),
    ]
    .into_iter()
    .map(|(id, name, kind)| AssetRecord::new(id, name, kind))
    .collect()
}

fn asset_browser_model(assets: &[AssetRecord]) -> AssetBrowserModel {
    AssetBrowserModel::new(
        assets
            .iter()
            .map(|asset| AssetBrowserItem::new(asset.id, &asset.name, asset.kind.label()))
            .collect::<Vec<_>>(),
    )
}

fn asset_browser_layout() -> AssetBrowserLayout {
    AssetBrowserLayout::new(
        AssetBrowserViewMode::List,
        GridLayout {
            columns: GridColumns::Fixed(2),
            item_size: Size::new(96.0, 72.0),
            gap: 4.0,
        },
        ListLayout::new(28.0),
    )
    .with_overscan(1)
}

fn prepare_asset_browser<'a>(
    ui: &mut Ui<'_>,
    bounds: Option<Rect>,
    model: &'a AssetBrowserModel,
    state: &AssetBrowserState,
) -> Option<stern::widgets::asset_browser::AssetBrowserScene<'a>> {
    bounds.and_then(|rect| {
        ui.prepare_asset_browser(
            "assets",
            AssetBrowserConfig::new(rect, asset_browser_layout())
                .label("Assets")
                .rename_policy(
                    InlineEditFocusLossPolicy::Commit,
                    InlineEditDraftPolicy::new(
                        InlineEditDraftDisposition::Commit,
                        InlineEditDraftDisposition::Cancel,
                    ),
                ),
            model,
            state,
        )
    })
}

fn rename_conflict(assets: &[AssetRecord], target: ItemId, draft: &str) -> Option<String> {
    let draft = draft.trim();
    if draft.is_empty() {
        return Some("Name is required".to_owned());
    }
    assets
        .iter()
        .any(|asset| asset.id != target && asset.name.eq_ignore_ascii_case(draft))
        .then(|| "Name already exists".to_owned())
}

fn apply_asset_browser_requests(assets: &mut [AssetRecord], requests: Vec<AssetBrowserRequest>) {
    for request in requests {
        let AssetBrowserRequest::Rename(InlineEditRequest::Commit(commit)) = request else {
            continue;
        };
        if let Some(asset) = assets.iter_mut().find(|asset| asset.id == commit.target) {
            commit.draft_text.trim().clone_into(&mut asset.name);
        }
    }
}

fn project_selection(assets: &mut [AssetRecord], state: &AssetBrowserState) {
    for asset in assets {
        asset.selected = state.selection.contains(asset.id);
    }
}

fn shared_context_requested(ui: &mut Ui<'_>, bounds: Option<Rect>) -> bool {
    bounds.is_some_and(|rect| {
        ui.context_menu_trigger("edit-workspace.shared-action-context", rect, false)
            .context_requested
    })
}

fn route_workspace_tabs(
    ui: &mut Ui<'_>,
    actions: &DemoActionRegistry,
    intents: &[ChromeSceneIntent],
) {
    for intent in intents {
        let ChromeSceneIntent::ActivateTab(target) = intent else {
            continue;
        };
        let action = if target.panel == PanelId::from_raw(101) {
            actions.edit_workspace()
        } else if target.panel == PanelId::from_raw(102) {
            actions.graph_workspace()
        } else {
            continue;
        };
        ui.push_action(ActionInvocation::new(
            action.id.clone(),
            ActionSource::Button,
            ActionContext::Editor,
        ));
    }
}

fn command_palette_requested(input: &UiInput) -> bool {
    input.keyboard.events.iter().any(|event| {
        event.state == KeyState::Pressed
            && !event.repeat
            && event.modifiers.ctrl
            && event.modifiers.shift
            && matches!(&event.key, Key::Character(value) if value.eq_ignore_ascii_case("p"))
    })
}

fn secondary_route_active(input: &UiInput) -> bool {
    let secondary = input.pointer.secondary;
    secondary.down || secondary.pressed || secondary.released
}

fn viewport_rect(bounds: Size) -> Rect {
    Rect::new(0.0, 0.0, bounds.width.max(0.0), bounds.height.max(0.0))
}

fn application_menu_scene(menu_bar: &MenuBar, anchor: Rect, bounds: Size) -> Option<OverlayScene> {
    let overlay = menu_bar.active_overlay(MenuBarOverlayRequest {
        overlay_id: APPLICATION_MENU_OVERLAY,
        kind: OverlayKind::Menu,
        anchor,
        size: Size::new(320.0, 96.0),
        placement: PopoverPlacement::Below,
        offset: 2.0,
        fit_viewport: true,
        viewport: viewport_rect(bounds),
        dismissal: OverlayDismissal::OutsideClickOrEscape,
        source: ActionSource::Menu,
        context: ActionContext::Editor,
    })?;
    let mut scene = OverlayScene::new();
    scene.push(OverlaySceneSurface::menu("Workspace commands", overlay));
    Some(scene)
}

fn context_menu_scene(actions: &DemoActionRegistry, anchor: Rect, bounds: Size) -> OverlayScene {
    let overlay = MenuOverlay::anchored(
        CONTEXT_MENU_OVERLAY,
        OverlayKind::ContextMenu,
        Menu::from_actions([actions.apply_shared_state().clone()]),
        anchor,
        Size::new(320.0, 40.0),
        PopoverPlacement::Below,
        2.0,
        true,
        viewport_rect(bounds),
        OverlayDismissal::OutsideClickOrEscape,
        ActionSource::Menu,
        ActionContext::Editor,
    );
    let mut scene = OverlayScene::new();
    scene.push(OverlaySceneSurface::menu("Viewport commands", overlay));
    scene
}

fn command_palette_scene(actions: &DemoActionRegistry, bounds: Size) -> OverlayScene {
    let viewport = viewport_rect(bounds);
    let anchor = Rect::new(viewport.width * 0.5, 24.0, 1.0, 1.0);
    let overlay = CommandPaletteOverlay::anchored_from_actions(
        COMMAND_PALETTE_OVERLAY,
        &[actions.apply_shared_state().clone()],
        anchor,
        Size::new(360.0, 96.0),
        PopoverPlacement::Below,
        4.0,
        true,
        viewport,
        OverlayDismissal::OutsideClickOrEscape,
        ActionContext::Editor,
    );
    let mut scene = OverlayScene::new();
    scene.push(OverlaySceneSurface::command_palette(
        "Shared command palette",
        overlay,
    ));
    scene
}

#[derive(Debug, Clone, Copy)]
struct WorkspaceLayout {
    menu: Rect,
    toolbar: Rect,
    tabs: Rect,
    dock: Rect,
    status: Rect,
}

impl WorkspaceLayout {
    fn new(size: Size) -> Self {
        let width = size.width.max(0.0);
        let height = size.height.max(0.0);
        let dock_y = 88.0_f32.min(height);
        let status_y = (height - 24.0).max(dock_y);
        Self {
            menu: Rect::new(0.0, 0.0, width, 28.0_f32.min(height)),
            toolbar: Rect::new(0.0, 28.0, width, 32.0_f32.min((height - 28.0).max(0.0))),
            tabs: Rect::new(0.0, 60.0, width, 28.0_f32.min((height - 60.0).max(0.0))),
            dock: Rect::new(0.0, dock_y, width, (status_y - dock_y).max(0.0)),
            status: Rect::new(0.0, status_y, width, (height - status_y).max(0.0)),
        }
    }
}

fn chrome_config(layout: WorkspaceLayout, actions: &DemoActionRegistry) -> ChromeSceneConfig {
    let mut widths = vec![
        (ChromeSceneItemKey::Menu(MenuBarMenuId::from_raw(1)), 96.0),
        (ChromeSceneItemKey::Tab(PanelId::from_raw(101)), 132.0),
        (ChromeSceneItemKey::Tab(PanelId::from_raw(102)), 140.0),
        (ChromeSceneItemKey::Status(StatusItemId::from_raw(1)), 152.0),
    ];
    widths.extend(actions.iter().map(|action| {
        (
            ChromeSceneItemKey::Toolbar {
                group: TOOLBAR_GROUP,
                action: action.id.clone(),
            },
            144.0,
        )
    }));
    ChromeSceneConfig::new(
        WidgetId::from_key("edit-workspace.chrome"),
        layout.menu,
        layout.toolbar,
        layout.tabs,
        layout.status,
        ActionContext::Editor,
    )
    .with_widths(widths)
}

fn workspace_tab(panel: u64, title: &str, active: bool) -> FrameTab {
    FrameTab {
        panel: PanelId::from_raw(panel),
        title: title.to_owned(),
        active,
        close_visible: false,
        draggable: false,
    }
}

fn edit_dock() -> Dock {
    let assets = dock_frame(1, ASSETS_PANEL, "Assets");
    let viewport = dock_frame(2, VIEWPORT_PANEL, "Viewport");
    let inspector = dock_frame(3, INSPECTOR_PANEL, "Inspector");
    let right = split(Axis::Horizontal, 0.72, viewport, inspector);
    let mut dock = Dock::new(split(Axis::Horizontal, 0.22, assets, right));
    let _ = dock.set_active_frame(FrameId::from_raw(2));
    dock
}

fn dock_frame(id: u64, panel_id: PanelId, title: &str) -> DockNode {
    let mut frame = Frame::new(FrameId::from_raw(id), vec![Panel::new(panel_id, title)]);
    let _ = frame.set_panel_dismissible(panel_id, false);
    DockNode::Frame(frame)
}

fn split(axis: Axis, ratio: f32, first: DockNode, second: DockNode) -> DockNode {
    DockNode::Split {
        axis,
        ratio,
        min_first: 120.0,
        min_second: 120.0,
        first: Box::new(first),
        second: Box::new(second),
    }
}

fn panel_bounds(scene: &DockScene, panel: PanelId) -> Option<Rect> {
    scene
        .layout()
        .frames
        .iter()
        .find_map(|frame| frame.panel.as_ref().filter(|item| item.panel == panel))
        .map(|panel| panel.rect)
}

fn inspector(ui: &mut Ui<'_>, bounds: Rect, selected: Option<&AssetRecord>) {
    let rows = [
        PropertyGridRow::section(ItemId::from_raw(100), "Selection"),
        PropertyGridRow::property(ItemId::from_raw(101), "Name", 0).with_read_only(true),
        PropertyGridRow::property(ItemId::from_raw(102), "Kind", 0).with_read_only(true),
    ];
    let values = selected.map_or(("No selection", "Unavailable"), |asset| {
        (asset.name.as_str(), asset.kind.label())
    });
    let _ = ui.property_grid(
        "selected-asset",
        bounds,
        &rows,
        PropertyGridConfig::default(),
        |ui, cell| {
            let value = if cell.row.id == ItemId::from_raw(101) {
                values.0
            } else {
                values.1
            };
            ui.label_keyed(("value", cell.row.id.raw()), cell.value_rect, value);
        },
    );
}

fn viewport_texture() -> TextureResource {
    let pixels = RenderImage::rgba8(
        1280,
        720,
        include_bytes!("../assets/viewport-1280x720.rgba").to_vec(),
    )
    .expect("bundled viewport RGBA dimensions are exact");
    TextureResource {
        id: VIEWPORT_TEXTURE,
        size: Size::new(1280.0, 720.0),
        sampling: RenderImageSampling::HighQuality,
        snapshot: Some(pixels),
    }
}
