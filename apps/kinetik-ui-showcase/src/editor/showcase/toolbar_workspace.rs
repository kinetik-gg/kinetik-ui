impl EditorShowcase {
    pub(super) fn tool_bar(
        &mut self,
        ui: &mut Ui<'_>,
        viewport: Rect,
        invocations: &mut Vec<EditorInvocation>,
    ) {
        let toolbar = self.toolbar_model();
        let chrome = EditorChromeMetrics::from_theme(ui.theme());
        let mut x = 10.0;
        let tool_items = toolbar
            .group(EditorToolbarGroupKind::Tools.id())
            .expect("editor toolbar declares tool group")
            .visible_items();
        let mut tool_responses = Vec::new();
        for (visible_index, ((_, icon, _label, action), item)) in
            EDITOR_TOOL_BUTTONS.into_iter().zip(tool_items).enumerate()
        {
            let button = Rect::new(x, TOOLBAR_Y, chrome.toolbar_button, chrome.toolbar_button);
            let id = ui.id(("editor.tool", action));
            let disabled = !item.enabled();
            let response = ui.pressable_with_id(id, button, disabled);
            if response.clicked {
                ui.request_repaint(RepaintRequest::NextFrame);
                let mut queue = ActionQueue::new();
                if toolbar.invoke_group_visible(
                    EditorToolbarGroupKind::Tools.id(),
                    visible_index,
                    &mut queue,
                    ActionContext::Editor,
                ) {
                    self.handle_action_queue(invocations, &mut queue);
                }
            }
            tool_responses.push((
                id,
                response,
                button,
                EDITOR_TOOL_BUTTONS[visible_index].0,
                icon,
                item.label(),
                disabled,
            ));
            x += chrome.toolbar_stride;
        }
        for (id, response, button, tool, icon, label, disabled) in tool_responses {
            paint_toolbar_icon_button_sized(
                ui,
                id,
                response,
                button,
                icon,
                label,
                self.selected_tool == tool,
                disabled,
                chrome.toolbar_icon,
            );
        }

        rect(
            ui,
            Rect::new(x + 4.0, TOOLBAR_Y + 3.0, 1.0, chrome.toolbar_button - 6.0),
            rgb(57, 60, 66),
            None,
        );
        x += 18.0;
        let viewport_items = toolbar
            .group(EditorToolbarGroupKind::Viewport.id())
            .expect("editor toolbar declares viewport group")
            .visible_items();
        for ((icon, _label, action), item) in [
            (ToolbarIcon::Grid, "Toggle grid", ACTION_GRID),
            (
                ToolbarIcon::Crosshair,
                "Frame selected",
                ACTION_VIEWPORT_FIT_SELECTION,
            ),
            (
                ToolbarIcon::Reset,
                "Reset view",
                ACTION_VIEWPORT_FIT_CONTENT,
            ),
        ]
        .into_iter()
        .zip(viewport_items)
        {
            let response = toolbar_icon_button(
                ui,
                ("editor.viewport-tool", action, icon.raw()),
                Rect::new(x, TOOLBAR_Y, chrome.toolbar_button, chrome.toolbar_button),
                icon,
                item.label(),
                false,
                !item.enabled(),
            );
            if response.clicked && item.can_invoke() {
                self.trigger(invocations, action, ActionSource::Button);
            }
            x += chrome.toolbar_stride;
        }

        rect(
            ui,
            Rect::new(x + 4.0, TOOLBAR_Y + 3.0, 1.0, chrome.toolbar_button - 6.0),
            rgb(57, 60, 66),
            None,
        );
        x += 18.0;
        let dock_items = toolbar
            .group(EditorToolbarGroupKind::Dock.id())
            .expect("editor toolbar declares dock group")
            .visible_items();
        for ((kind, icon, _label, action), item) in [
            (
                DockSplitterContextActionKind::Join,
                ToolbarIcon::Component,
                "Join dock splitter",
                ACTION_DOCK_JOIN,
            ),
            (
                DockSplitterContextActionKind::Swap,
                ToolbarIcon::Layers,
                "Swap dock frames",
                ACTION_DOCK_SWAP,
            ),
        ]
        .into_iter()
        .zip(dock_items)
        {
            let response = toolbar_icon_button(
                ui,
                ("editor.dock-action", action),
                Rect::new(x, TOOLBAR_Y, chrome.toolbar_button, chrome.toolbar_button),
                icon,
                item.label(),
                false,
                !item.enabled(),
            );
            if response.clicked && item.can_invoke() {
                let bounds = editor_workspace_rect(ui.theme(), viewport);
                if self.apply_splitter_context_action(bounds, kind) {
                    invocations.push(ActionInvocation::new(
                        ActionId::new(action),
                        ActionSource::Button,
                        ActionContext::Editor,
                    ));
                }
                ui.request_repaint(RepaintRequest::NextFrame);
            }
            x += chrome.toolbar_stride;
        }

        let run_items = toolbar
            .group(EditorToolbarGroupKind::Run.id())
            .expect("editor toolbar declares run group")
            .visible_items();
        for ((index, icon, _label, action, rect), item) in run_toolbar_buttons(viewport, chrome)
            .into_iter()
            .zip(run_items)
        {
            let response = toolbar_icon_button(
                ui,
                ("editor.run", action, index),
                rect,
                icon,
                item.label(),
                false,
                !item.enabled(),
            );
            if response.clicked {
                let mut queue = ActionQueue::new();
                if toolbar.invoke_group_visible(
                    EditorToolbarGroupKind::Run.id(),
                    index,
                    &mut queue,
                    ActionContext::Editor,
                ) {
                    self.handle_action_queue(invocations, &mut queue);
                }
            }
        }
    }

    pub(super) fn apply_splitter_context_action(
        &mut self,
        bounds: Rect,
        kind: DockSplitterContextActionKind,
    ) -> bool {
        let frame_layouts = solve_dock_layout(&self.dock, bounds);
        let Some(splitter) =
            solve_dock_splitters_with_style(&self.dock, bounds, editor_dock_chrome_style())
                .into_iter()
                .next()
        else {
            "No dock splitter action available".clone_into(&mut self.status);
            return false;
        };
        let policy = editor_dock_interaction_policy();
        let actions = resolve_dock_splitter_context_actions_with_policy(
            &self.dock,
            &frame_layouts,
            &splitter,
            policy,
        );
        let Some(action) = actions
            .into_iter()
            .find(|action| action.kind == kind && action.enabled)
        else {
            match kind {
                DockSplitterContextActionKind::Join => "No dock join action available",
                DockSplitterContextActionKind::Swap => "No dock swap action available",
            }
            .clone_into(&mut self.status);
            return false;
        };

        match kind {
            DockSplitterContextActionKind::Join => {
                let Some(request) = action.join_request() else {
                    "No dock join action available".clone_into(&mut self.status);
                    return false;
                };
                let source = request.source_frame();
                let target = request.target_frame();
                if self
                    .dock
                    .apply_join_request_with_policy(bounds, request, policy)
                {
                    self.status = format!(
                        "Dock splitter joined frame {} into frame {}",
                        source.raw(),
                        target.raw()
                    );
                    true
                } else {
                    "Dock join request rejected".clone_into(&mut self.status);
                    false
                }
            }
            DockSplitterContextActionKind::Swap => {
                let Some(request) = action.swap_request() else {
                    "No dock swap action available".clone_into(&mut self.status);
                    return false;
                };
                let source = request.source_frame();
                let target = request.target_frame();
                if self
                    .dock
                    .apply_swap_request_with_policy(bounds, request, policy)
                {
                    self.status = format!(
                        "Dock splitter swapped frame {} with frame {}",
                        source.raw(),
                        target.raw()
                    );
                    true
                } else {
                    "Dock swap request rejected".clone_into(&mut self.status);
                    false
                }
            }
        }
    }

    pub(super) fn workspace(
        &mut self,
        ui: &mut Ui<'_>,
        workflow: &EditorWorkflowScenes<'_>,
    ) {
        let output = ui.dock_controller(
            &workflow.dock,
            &mut self.dock,
            &mut self.dock_controller,
            dock::DockControllerConfig::new(FrameId::from_raw(self.next_drop_frame))
                .with_policy(editor_dock_interaction_policy()),
        );
        if output.changed {
            self.next_drop_frame = self.next_drop_frame.saturating_add(1);
            self.status = "Dock layout updated".to_owned();
        }
        if let Some(close) = output.close_requests.first() {
            self.status = format!("Close requested for panel {}", close.panel.raw());
        }
        if !output.splitter_context_requests.is_empty() {
            self.status = "Dock splitter actions requested".to_owned();
        }

        let _ = ui.dock_scene(&workflow.dock, |ui, panel| match panel.panel {
            PANEL_SCENE => self.scene_graph(ui, workflow.outliner.as_ref()),
            PANEL_ASSETS => self.assets_browser(
                ui,
                workflow.asset_search_bounds,
                workflow.assets.as_ref(),
            ),
            PANEL_VIEWPORT => self.viewport_panel(
                ui,
                workflow.viewport.as_ref(),
                workflow.viewport_tools.as_ref(),
            ),
            PANEL_CONSOLE => Self::console_panel(ui, panel.rect),
            PANEL_TIMELINE => Self::timeline_panel(ui, panel.rect),
            PANEL_INSPECTOR => self.inspector(ui, panel.rect),
            PANEL_NODE_GRAPH => Self::node_graph_panel(ui, panel.rect),
            _ => {}
        });
    }
}
