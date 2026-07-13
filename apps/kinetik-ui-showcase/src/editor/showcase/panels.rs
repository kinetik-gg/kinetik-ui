impl EditorShowcase {
    pub(super) fn scene_graph(
        &mut self,
        ui: &mut Ui<'_>,
        scene: Option<&outliner::OutlinerScene<'_>>,
    ) {
        let Some(scene) = scene else {
            return;
        };
        let output = ui.outliner(scene, &mut self.outliner_state, |_| Vec::new());
        self.scene_expansion = self.outliner_state.expansion.clone();
        if output.selection_changed
            && let Some(selected) = self.outliner_state.selection.active
        {
            self.selected_node = selected;
            self.status = format!("Selected {}", self.object_names[(selected.raw() - 1) as usize]);
        }
        self.apply_outliner_requests(&output.requests);
    }

    pub(super) fn assets_browser(
        &mut self,
        ui: &mut Ui<'_>,
        search_bounds: Option<Rect>,
        scene: Option<&asset_browser::AssetBrowserScene<'_>>,
    ) {
        if let Some(search_bounds) = search_bounds {
            ui.search_field(
                "editor.workflow.asset-search",
                search_bounds,
                &mut self.asset_filter,
                false,
            );
        }
        let Some(scene) = scene else {
            return;
        };
        let output = ui.asset_browser(
            scene,
            &mut self.asset_browser_state,
            |_, _| None,
            |_| Vec::new(),
        );
        if output.selection_changed
            && let Some(selected) = self.asset_browser_state.selection.active
            && let Some(index) = selected.raw().checked_sub(WORKFLOW_ASSET_ID_BASE)
            && let Ok(index) = usize::try_from(index)
            && index < ASSETS.len()
        {
            self.selected_asset = index;
            self.status = format!("Asset selected: {}", ASSETS[index].name);
        }
        self.record_asset_drag(output.drag_payload.as_ref());
        for request in output.requests {
            match request {
                asset_browser::AssetBrowserRequest::Drop(drop) => {
                    self.record_asset_drag(Some(&drop.source));
                }
                asset_browser::AssetBrowserRequest::Preview(asset) => {
                    self.status = format!("Asset preview requested: {}", asset.raw());
                }
                asset_browser::AssetBrowserRequest::Rename(_)
                | asset_browser::AssetBrowserRequest::Context(_) => {}
            }
        }
    }

    pub(super) fn viewport_panel(
        &mut self,
        ui: &mut Ui<'_>,
        viewport: Option<&viewport::ViewportWidget>,
        tools: Option<&viewport::ViewportToolScene>,
    ) {
        let Some(viewport) = viewport else {
            return;
        };
        let output = ui.viewport_widget(viewport, &mut self.viewport_pan_zoom, &[]);
        if output.changed() {
            self.status = "Viewport navigation updated".to_owned();
        }
        if let Some(tools) = tools {
            let output = ui.viewport_tool_scene(tools, &mut self.viewport_tool_controller);
            self.apply_viewport_interactions(&output.interactions);
        }
    }
}
