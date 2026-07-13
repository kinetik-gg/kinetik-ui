use kinetik_ui::widgets::{asset_browser, collection_actions, dock, inline_edit, outliner, viewport};

const WORKFLOW_ASSET_ID_BASE: u64 = 1_000;
const WORKFLOW_VIEWPORT_TARGET: viewport::ViewportSelectionTargetId =
    viewport::ViewportSelectionTargetId::from_raw(1);

#[derive(Debug, Clone, PartialEq)]
struct EditorProjectSnapshot {
    revision: u64,
    object_names: Vec<String>,
    selected_object: Option<ItemId>,
    roughness: f32,
    asset_query: String,
    dragged_asset: Option<ItemId>,
    assigned_asset: Option<ItemId>,
    viewport_selection_rect: Rect,
    workspace: WorkspaceSnapshot,
}

fn workflow_object_names() -> Vec<String> {
    (1..=11)
        .map(|raw| scene_label(item_id(raw)).to_owned())
        .collect()
}

const fn workflow_asset_id(index: usize) -> ItemId {
    ItemId::from_raw(WORKFLOW_ASSET_ID_BASE + index as u64)
}

fn workflow_outliner_model(object_names: &[String]) -> outliner::OutlinerModel {
    let topology = [
        (1, None, true),
        (2, Some(1), true),
        (3, Some(2), false),
        (4, Some(2), false),
        (5, Some(2), false),
        (6, Some(1), true),
        (7, Some(6), false),
        (8, Some(6), false),
        (9, Some(1), false),
        (10, Some(1), false),
        (11, Some(1), false),
    ];
    outliner::OutlinerModel::new(
        topology
            .into_iter()
            .enumerate()
            .map(|(index, (raw, parent, has_children))| {
                let item = outliner::OutlinerItem::new(
                    item_id(raw),
                    object_names
                        .get(index)
                        .cloned()
                        .unwrap_or_else(|| format!("Object {raw}")),
                )
                .with_has_children(has_children);
                parent.map_or(item.clone(), |parent| item.with_parent(item_id(parent)))
            })
            .collect::<Vec<_>>(),
    )
}

fn workflow_asset_model() -> asset_browser::AssetBrowserModel {
    asset_browser::AssetBrowserModel::new(
        ASSETS
            .iter()
            .enumerate()
            .map(|(index, asset)| {
                asset_browser::AssetBrowserItem::new(
                    workflow_asset_id(index),
                    asset.name,
                    asset.kind,
                )
                .with_tags([asset.kind])
            })
            .collect::<Vec<_>>(),
    )
}

struct EditorWorkflowScenes<'model> {
    dock: dock::DockScene,
    outliner: Option<outliner::OutlinerScene<'model>>,
    outliner_bounds: Option<Rect>,
    assets: Option<asset_browser::AssetBrowserScene<'model>>,
    asset_search_bounds: Option<Rect>,
    asset_search_target: Option<(WidgetId, Rect)>,
    asset_bounds: Option<Rect>,
    roughness_target: Option<(WidgetId, Rect)>,
    viewport: Option<viewport::ViewportWidget>,
    viewport_tools: Option<viewport::ViewportToolScene>,
}

impl EditorShowcase {
    fn prepare_workflow_scenes<'model>(
        &self,
        ui: &Ui<'_>,
        viewport_rect: Rect,
        outliner_model: &'model outliner::OutlinerModel,
        asset_model: &'model asset_browser::AssetBrowserModel,
    ) -> EditorWorkflowScenes<'model> {
        let dock_bounds = editor_workspace_rect(ui.theme(), viewport_rect);
        let dock = dock::DockScene::new(
            dock::DockSceneConfig::new(ui.make_id("editor.workflow.dock"), dock_bounds)
                .with_tab_height(26.0)
                .with_chrome_style(editor_dock_chrome_style())
                .with_drop_preview(self.dock_controller.drop_preview()),
            &self.dock,
        );

        let panel_bounds = |panel_id| {
            dock.layout()
                .frames
                .iter()
                .filter_map(|frame| frame.panel.as_ref())
                .find(|panel| panel.panel == panel_id)
                .map(|panel| panel.rect)
        };

        let outliner_bounds = panel_bounds(PANEL_SCENE).map(|bounds| bounds.inset(6.0));
        let outliner = outliner_bounds.and_then(|bounds| {
            ui.prepare_outliner(
                "editor.workflow.outliner",
                outliner::OutlinerConfig::new(bounds, 24.0, 16.0).label("Scene outliner"),
                outliner_model,
                &self.outliner_state,
            )
        });

        let asset_panel_scene = dock
            .layout()
            .frames
            .iter()
            .filter_map(|frame| frame.panel.as_ref())
            .find(|panel| panel.panel == PANEL_ASSETS);
        let asset_panel = asset_panel_scene.map(|panel| panel.rect);
        let asset_search_bounds = asset_panel.map(|bounds| {
            Rect::new(
                bounds.x + 6.0,
                bounds.y + 6.0,
                (bounds.width - 12.0).max(0.0),
                26.0,
            )
        });
        let asset_bounds = asset_panel.map(|bounds| {
            Rect::new(
                bounds.x + 6.0,
                bounds.y + 38.0,
                (bounds.width - 12.0).max(0.0),
                (bounds.height - 44.0).max(0.0),
            )
        });
        let assets = asset_bounds.and_then(|bounds| {
            let layout = asset_browser::AssetBrowserLayout::new(
                asset_browser::AssetBrowserViewMode::Grid,
                kinetik_ui::widgets::collections::GridLayout {
                    columns: kinetik_ui::widgets::collections::GridColumns::Adaptive {
                        min_width: 92.0,
                    },
                    item_size: Size::new(88.0, 74.0),
                    gap: 6.0,
                },
                kinetik_ui::widgets::collections::ListLayout::new(28.0),
            )
            .with_overscan(1);
            ui.prepare_asset_browser(
                "editor.workflow.assets",
                asset_browser::AssetBrowserConfig::new(bounds, layout)
                    .query(&self.asset_filter.text)
                    .label("Project assets"),
                asset_model,
                &self.asset_browser_state,
            )
        });

        let asset_search_target = asset_panel_scene.zip(asset_search_bounds).map(
            |(panel, bounds)| {
                (
                    ui.make_id(("dock-panel-content", panel.id.raw()))
                        .child("editor.workflow.asset-search"),
                    bounds,
                )
            },
        );

        let roughness_target = dock
            .layout()
            .frames
            .iter()
            .filter_map(|frame| frame.panel.as_ref())
            .find(|panel| panel.panel == PANEL_INSPECTOR)
            .and_then(|panel| self.prepare_roughness_pointer_target(ui, panel.id, panel.rect));

        let (viewport, viewport_tools) = panel_bounds(PANEL_VIEWPORT).map_or((None, None), |body| {
            let bounds = body.inset(6.0);
            let surface = viewport::ViewportSurface {
                texture: VIEWPORT_TEXTURE,
                source_size: VIEWPORT_SIZE,
                bounds,
                pan_zoom: self.viewport_pan_zoom,
            };
            let viewport = ui.prepare_viewport_widget(
                viewport::ViewportWidgetConfig::new(
                    ui.make_id("editor.workflow.viewport"),
                    surface,
                )
                .with_label("Project viewport"),
            );
            let target = viewport::ViewportSelectionTargetDescriptor::new(
                WORKFLOW_VIEWPORT_TARGET,
                self.viewport_selection_rect,
            )
            .with_handles(viewport::ViewportTransformHandleSet::move_only())
            .with_label(
                self.outliner_state
                    .selection
                    .active
                    .and_then(|selected| {
                        usize::try_from(selected.raw())
                            .ok()
                            .and_then(|raw| raw.checked_sub(1))
                            .and_then(|index| self.object_names.get(index))
                    })
                    .map_or("Selected object", String::as_str),
            );
            let tools = ui.prepare_viewport_tool_scene(
                &viewport,
                viewport::ViewportToolSceneConfig::new([target]),
            );
            (Some(viewport), Some(tools))
        });

        EditorWorkflowScenes {
            dock,
            outliner,
            outliner_bounds,
            assets,
            asset_search_bounds,
            asset_search_target,
            asset_bounds,
            roughness_target,
            viewport,
            viewport_tools,
        }
    }

    fn install_workflow_pointer_plan(
        &self,
        ui: &mut Ui<'_>,
        scenes: &EditorWorkflowScenes<'_>,
    ) {
        if !self.workflow_pointer_plan_needed(ui, scenes) {
            return;
        }
        ui.resolve_pointer_targets(|plan| {
            scenes.dock.declare_pointer_targets_with_content(
                plan,
                kinetik_ui::core::PointerOrder::new(100),
                |plan, mut order| {
                    if let Some(scene) = &scenes.outliner {
                        order = scene.declare_pointer_targets(plan, order, &self.outliner_state);
                    }
                    if let Some(scene) = &scenes.assets {
                        order =
                            scene.declare_pointer_targets(plan, order, &self.asset_browser_state);
                    }
                    if let Some((id, rect)) = scenes.asset_search_target {
                        plan.target(kinetik_ui::core::PointerTarget::new(id, rect, order));
                        order = kinetik_ui::core::PointerOrder::new(
                            order.raw().saturating_add(1),
                        );
                    }
                    if let Some(viewport) = &scenes.viewport {
                        order = viewport.declare_pointer_targets(plan, order);
                    }
                    if let Some(tools) = &scenes.viewport_tools {
                        order = tools.declare_pointer_targets(plan, order);
                    }
                    if let Some((id, rect)) = scenes.roughness_target {
                        plan.target(kinetik_ui::core::PointerTarget::new(id, rect, order));
                        order = kinetik_ui::core::PointerOrder::new(
                            order.raw().saturating_add(1),
                        );
                    }
                    order
                },
            );
        })
        .expect("workflow pointer IDs and orders are deterministic");
    }

    fn workflow_pointer_plan_needed(
        &self,
        ui: &Ui<'_>,
        scenes: &EditorWorkflowScenes<'_>,
    ) -> bool {
        if self.outliner_state.dragging()
            || self.asset_browser_state.drag_source().is_some()
            || self.viewport_tool_controller.captured_handle().is_some()
            || self.dock_controller.tab_drag().is_some()
        {
            return true;
        }
        let Some(pointer) = ui.input().pointer.position else {
            return false;
        };
        scenes
            .outliner_bounds
            .into_iter()
            .chain(scenes.asset_search_target.map(|(_, rect)| rect))
            .chain(scenes.asset_bounds)
            .chain(scenes.roughness_target.map(|(_, rect)| rect))
            .any(|bounds| bounds.contains_point(pointer))
            || scenes.viewport.as_ref().is_some_and(|viewport| {
                viewport
                    .surface()
                    .effective_bounds()
                    .contains_point(pointer)
            })
            || scenes.dock.layout().frames.iter().any(|frame| {
                frame.tab_list_rect.contains_point(pointer)
                    || frame.tabs.iter().any(|tab| tab.rect.contains_point(pointer))
            })
            || scenes
                .dock
                .layout()
                .splitters
                .iter()
                .any(|splitter| splitter.rect.contains_point(pointer))
    }

    fn prepare_roughness_pointer_target(
        &self,
        ui: &Ui<'_>,
        panel: WidgetId,
        body: Rect,
    ) -> Option<(WidgetId, Rect)> {
        let rows = inspector_rows(&self.mass.text);
        let (grid, layout) = inspector_grid_geometry(body);
        let panel_scope = ui.make_id(("dock-panel-content", panel.raw()));
        let grid_root = panel_scope.child("editor.workflow.property-grid");
        let scroll = grid_root.child("property-grid-scroll");
        let content_size = Size::new(
            grid.width.max(0.0),
            layout.content_height(&rows).max(grid.height.max(0.0)),
        );
        let offset = kinetik_ui::core::clamp_scroll_offset(
            ui.memory().scroll_offset(scroll),
            grid.size(),
            content_size,
        );
        let geometry = layout
            .visible_row_rects(grid, &rows, offset.y, 1)
            .into_iter()
            .find(|geometry| rows[geometry.index].id == item_id(8))?;
        let row = &rows[geometry.index];
        let value_rect = kinetik_ui::widgets::property_grid_row_affordance_rects(
            row,
            geometry
                .value_rect
                .intersection(grid)
                .unwrap_or(Rect::ZERO)
                .inset(2.0)
                .max_zero(),
            kinetik_ui::widgets::PropertyGridAffordanceLayout::default(),
        )
        .value_rect;
        let id = grid_root
            .child(("property-grid-row", row.id.raw()))
            .child("value")
            .child("editor.inspector.roughness");
        Some((id, value_rect))
    }
}

impl EditorShowcase {
    fn apply_outliner_requests(&mut self, requests: &[outliner::OutlinerRequest]) {
        for request in requests {
            if let outliner::OutlinerRequest::Rename(inline_edit::InlineEditRequest::Commit(
                commit,
            )) = request
            {
                let draft = commit.draft_text.trim();
                let Some(index) = usize::try_from(commit.target.raw())
                    .ok()
                    .and_then(|raw| raw.checked_sub(1))
                else {
                    continue;
                };
                let Some(name) = self.object_names.get_mut(index) else {
                    continue;
                };
                if draft.is_empty() {
                    continue;
                }
                draft.clone_into(name);
                self.status = format!("Renamed object to {draft}");
            }
        }
        if let Some(selected) = self.outliner_state.selection.active {
            self.selected_node = selected;
        }
    }

    fn record_asset_drag(&mut self, drag: Option<&collection_actions::CollectionDragSource>) {
        let Some(asset) = drag.and_then(|drag| drag.items.first()).copied() else {
            return;
        };
        self.dragged_asset = Some(asset);
        self.assigned_asset = Some(asset);
        self.status = format!("Asset {} dragged into project state", asset.raw());
    }

    fn apply_viewport_interactions(
        &mut self,
        interactions: &[viewport::ViewportTransformInteractionRequest],
    ) {
        for interaction in interactions {
            let drag = &interaction.drag;
            if drag.target != WORKFLOW_VIEWPORT_TARGET
                || drag.kind != viewport::ViewportTransformHandleKind::Move
            {
                continue;
            }
            match interaction.phase {
                viewport::ViewportTransformInteractionPhase::Started
                | viewport::ViewportTransformInteractionPhase::Updated
                | viewport::ViewportTransformInteractionPhase::Finished
                    if drag.status == viewport::ViewportTransformDragStatus::Active =>
                {
                    self.viewport_selection_rect = Rect::new(
                        drag.source_content_rect.x + drag.content_delta.x,
                        drag.source_content_rect.y + drag.content_delta.y,
                        drag.source_content_rect.width,
                        drag.source_content_rect.height,
                    );
                    self.position[0] = self.viewport_selection_rect.x;
                    self.position[1] = self.viewport_selection_rect.y;
                    self.status = "Viewport object moved".to_owned();
                }
                viewport::ViewportTransformInteractionPhase::Cancelled => {
                    self.viewport_selection_rect = drag.source_content_rect;
                    self.status = "Viewport move cancelled".to_owned();
                }
                _ => {}
            }
        }
    }

    fn capture_project_snapshot(&self, revision: u64) -> EditorProjectSnapshot {
        EditorProjectSnapshot {
            revision,
            object_names: self.object_names.clone(),
            selected_object: self.outliner_state.selection.active,
            roughness: self.roughness,
            asset_query: self.asset_filter.text.clone(),
            dragged_asset: self.dragged_asset,
            assigned_asset: self.assigned_asset,
            viewport_selection_rect: self.viewport_selection_rect,
            workspace: self.dock.workspace_snapshot(editor_panel_instances()),
        }
    }

    fn save_project_in_memory(&mut self) {
        self.save_revision = self.save_revision.saturating_add(1);
        self.saved_project = Some(self.capture_project_snapshot(self.save_revision));
        self.status = format!("Project state saved in memory (revision {})", self.save_revision);
    }
}
