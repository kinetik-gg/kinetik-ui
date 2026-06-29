//! Collection projection, filtering, sorting, and selection conformance tests.

mod collection_projection_conformance {
    use kinetik_ui_core::{Rect, Size};
    use kinetik_ui_widgets::{
        AssetBrowserItem, AssetBrowserLayout, AssetBrowserModel, AssetBrowserSort,
        AssetBrowserSortKey, AssetBrowserViewMode, CollectionProjection, GridColumns, GridLayout,
        ItemId, ListLayout, OutlinerItem, OutlinerModel, Selection, SelectionProjectionPolicy,
        SortDirection, TreeExpansion,
    };

    fn id(raw: u64) -> ItemId {
        ItemId::from_raw(raw)
    }

    fn asset(raw: u64, name: &str, kind: &str, tags: &[&str]) -> AssetBrowserItem {
        AssetBrowserItem::new(id(raw), name, kind).with_tags(tags.iter().copied())
    }

    fn asset_model() -> AssetBrowserModel {
        AssetBrowserModel::new(vec![
            asset(10, "Concrete", "material", &["surface", "rough"]),
            asset(20, "Camera Rig", "prefab", &["scene", "camera"]),
            asset(30, "Studio HDRI", "image", &["lighting", "hdr"]),
            asset(40, "Terrain", "mesh", &["surface", "large"]),
        ])
    }

    fn asset_layout(view_mode: AssetBrowserViewMode) -> AssetBrowserLayout {
        AssetBrowserLayout::new(
            view_mode,
            GridLayout {
                columns: GridColumns::Fixed(2),
                item_size: Size::new(80.0, 60.0),
                gap: 4.0,
            },
            ListLayout::new(24.0),
        )
    }

    #[test]
    fn selected_ids_survive_filtering_out_and_back_in() {
        let source_ids = [id(10), id(20), id(30), id(40)];
        let all = CollectionProjection::from_source_ids(&source_ids);
        let filtered = all.filtered_by(|item| item.id != id(20));
        let mut selection = Selection::new();
        selection.replace(id(20));

        let hidden = selection.project(&filtered, SelectionProjectionPolicy::PreserveHidden);
        assert_eq!(selection.selected(), vec![id(20)]);
        assert_eq!(hidden.visible_selected, Vec::<ItemId>::new());
        assert_eq!(hidden.hidden_selected, vec![id(20)]);
        assert!(hidden.has_hidden_selection());
        assert_eq!(hidden.repaired_active, None);
        assert_eq!(hidden.repaired_anchor, None);

        let visible_again = selection.project(&all, SelectionProjectionPolicy::PreserveHidden);
        assert_eq!(visible_again.visible_selected, vec![id(20)]);
        assert_eq!(visible_again.hidden_selected, Vec::<ItemId>::new());
        assert_eq!(visible_again.repaired_active, Some(id(20)));
        assert_eq!(visible_again.repaired_anchor, Some(id(20)));
    }

    #[test]
    fn active_and_anchor_repair_is_deterministic_in_projected_order() {
        let projection = CollectionProjection::from_source_ids(&[id(30), id(20), id(10)])
            .filtered_by(|item| item.id == id(30) || item.id == id(20));
        let mut selection = Selection::new();
        selection.replace(id(20));
        selection.toggle(id(10));

        let projected = selection.project(&projection, SelectionProjectionPolicy::PreserveHidden);

        assert_eq!(projected.selected, vec![id(10), id(20)]);
        assert_eq!(projected.visible_selected, vec![id(20)]);
        assert_eq!(projected.hidden_selected, vec![id(10)]);
        assert_eq!(projected.active, Some(id(10)));
        assert_eq!(projected.anchor, Some(id(10)));
        assert_eq!(projected.visible_active, None);
        assert_eq!(projected.visible_anchor, None);
        assert_eq!(projected.repaired_active, Some(id(20)));
        assert_eq!(projected.repaired_anchor, Some(id(20)));
    }

    #[test]
    fn source_index_mapping_is_stable_under_sort() {
        let projection = CollectionProjection::from_source_ids(&[id(10), id(20), id(30)])
            .sorted_by(|lhs, rhs| rhs.id.raw().cmp(&lhs.id.raw()));

        assert_eq!(projection.visible_ids(), vec![id(30), id(20), id(10)]);
        assert_eq!(projection.source_indices(), vec![2, 1, 0]);
        assert_eq!(projection.source_index(id(30)), Some(2));
        assert_eq!(projection.projected_index(id(10)), Some(2));
    }

    #[test]
    fn tree_expansion_survives_filtered_hidden_descendants() {
        let model = OutlinerModel::new(vec![
            OutlinerItem::new(id(1), "World").with_has_children(true),
            OutlinerItem::new(id(2), "Rig")
                .with_parent(id(1))
                .with_has_children(true),
            OutlinerItem::new(id(3), "Camera").with_parent(id(2)),
            OutlinerItem::new(id(4), "Material"),
        ]);
        let mut expansion = TreeExpansion::new();
        expansion.expand(id(1));
        expansion.expand(id(2));

        let filtered = model.filtered_visible_rows(&expansion, |item| item.id != id(3));

        assert_eq!(
            filtered.iter().map(|row| row.id).collect::<Vec<_>>(),
            vec![id(1), id(2), id(4)]
        );
        assert_eq!(expansion.expanded(), vec![id(1), id(2)]);

        let unfiltered = model.visible_rows(&expansion);
        assert_eq!(
            unfiltered.iter().map(|row| row.id).collect::<Vec<_>>(),
            vec![id(1), id(2), id(3), id(4)]
        );
    }

    #[test]
    fn outliner_filter_keeps_matching_descendant_context() {
        let model = OutlinerModel::new(vec![
            OutlinerItem::new(id(1), "World").with_has_children(true),
            OutlinerItem::new(id(2), "Lighting").with_parent(id(1)),
            OutlinerItem::new(id(3), "Materials"),
        ]);
        let mut expansion = TreeExpansion::new();
        expansion.expand(id(1));

        let rows = model.filtered_visible_rows(&expansion, |item| item.label.contains("Light"));

        assert_eq!(
            rows.iter()
                .map(|row| (row.id, row.depth, row.label.as_str()))
                .collect::<Vec<_>>(),
            vec![(id(1), 0, "World"), (id(2), 1, "Lighting")]
        );
        assert_eq!(expansion.expanded(), vec![id(1)]);
    }

    #[test]
    fn asset_sorting_uses_app_provided_keys_and_source_indices() {
        let model = asset_model();
        let by_kind = model.projected(
            |_| true,
            Some(AssetBrowserSort::new(
                AssetBrowserSortKey::Kind,
                SortDirection::Ascending,
            )),
        );
        assert_eq!(by_kind.visible_ids(), vec![id(30), id(10), id(40), id(20)]);
        assert_eq!(by_kind.source_indices(), vec![2, 0, 3, 1]);

        let by_tags_desc = model.projected(
            |_| true,
            Some(AssetBrowserSort::new(
                AssetBrowserSortKey::Tags,
                SortDirection::Descending,
            )),
        );
        assert_eq!(
            by_tags_desc.visible_ids(),
            vec![id(10), id(40), id(20), id(30)]
        );
    }

    #[test]
    fn asset_grid_and_list_selection_survives_view_mode_switch() {
        let model = asset_model();
        let projection = model.projected(
            |item| item.tags.iter().any(|tag| tag == "surface"),
            Some(AssetBrowserSort::new(
                AssetBrowserSortKey::Name,
                SortDirection::Ascending,
            )),
        );
        let mut selection = Selection::new();
        selection.replace(id(40));

        let grid = asset_layout(AssetBrowserViewMode::Grid).resolve_projected(
            Rect::new(0.0, 0.0, 200.0, 80.0),
            &model,
            &projection,
            0.0,
            &selection,
            None,
        );
        let list = asset_layout(AssetBrowserViewMode::List).resolve_projected(
            Rect::new(0.0, 0.0, 200.0, 80.0),
            &model,
            &projection,
            0.0,
            &selection,
            None,
        );

        assert_eq!(projection.visible_ids(), vec![id(10), id(40)]);
        assert_eq!(grid.materialized_item_ids(), vec![id(10), id(40)]);
        assert_eq!(list.materialized_item_ids(), vec![id(10), id(40)]);
        assert_eq!(grid.items[1].item.index, 3);
        assert_eq!(list.items[1].item.index, 3);
        assert!(grid.items[1].item.state.selected);
        assert!(list.items[1].item.state.selected);
    }
}
