use super::super::{ACTION_SAVE, EditorShowcase, item_id, workflow_asset_id};
use kinetik_ui::core::WidgetId;
use kinetik_ui::widgets::inline_edit::{
    InlineEditCommitReason, InlineEditCommitRequest, InlineEditRequest,
};
use kinetik_ui::widgets::outliner::OutlinerRequest;

#[test]
fn application_owned_rename_is_reflected_by_the_public_outliner_model() {
    let mut editor = EditorShowcase::new();
    editor.apply_outliner_requests(&[OutlinerRequest::Rename(InlineEditRequest::Commit(
        InlineEditCommitRequest {
            target: item_id(7),
            draft_text: "Hero".to_owned(),
            text_widget_id: WidgetId::from_raw(77),
            reason: InlineEditCommitReason::Enter,
        },
    ))]);

    assert_eq!(editor.object_names[6], "Hero");
    assert_eq!(editor.outliner_state.selection.active, Some(item_id(7)));
    let model = super::super::workflow_outliner_model(&editor.object_names);
    assert_eq!(
        model.item_by_id(item_id(7)).map(|item| item.label.as_str()),
        Some("Hero")
    );
}

#[test]
fn save_action_captures_the_resulting_project_state_in_memory() {
    let mut editor = EditorShowcase::new();
    editor.object_names[6] = "Hero".to_owned();
    editor.roughness = 0.81;
    editor.asset_filter.text = "terrain".to_owned();
    editor.dragged_asset = Some(workflow_asset_id(1));
    editor.assigned_asset = Some(workflow_asset_id(1));
    editor.viewport_selection_rect.x += 24.0;

    assert!(editor.apply_action(ACTION_SAVE));
    let saved = editor.saved_project.as_ref().expect("saved snapshot");
    assert_eq!(saved.revision, 1);
    assert_eq!(saved.object_names[6], "Hero");
    assert_eq!(saved.selected_object, Some(item_id(7)));
    assert_eq!(saved.roughness, 0.81);
    assert_eq!(saved.asset_query, "terrain");
    assert_eq!(saved.dragged_asset, Some(workflow_asset_id(1)));
    assert_eq!(saved.assigned_asset, Some(workflow_asset_id(1)));
    assert_eq!(saved.viewport_selection_rect.x, 744.0);
    assert!(
        saved
            .workspace
            .diagnostics(super::super::editor_panel_registry().descriptors())
            .is_valid()
    );
    assert_eq!(editor.status, "Project state saved in memory (revision 1)");
}
