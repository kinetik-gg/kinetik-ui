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
