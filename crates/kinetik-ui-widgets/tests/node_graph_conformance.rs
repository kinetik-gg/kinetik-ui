//! Node graph identity and coordinate-space conformance tests.

mod node_graph_conformance {
    use kinetik_ui_core::{Point, Rect};
    use kinetik_ui_widgets::{
        EdgeDescriptor, EdgeId, GraphPoint, GraphRect, GraphVector, NodeDescriptor,
        NodeFrameDescriptor, NodeFrameId, NodeGraphDescriptor, NodeGraphPanZoom,
        NodeGraphValidationError, NodeGraphViewport, NodeGroupDescriptor, NodeGroupId, NodeId,
        PortCompatibilityError, PortDescriptor, PortDirection, PortEndpoint, PortId, PortTypeId,
        ports_are_compatible, validate_node_graph_descriptors, validate_port_compatibility,
    };

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() <= 0.001,
            "expected {actual} to equal {expected}"
        );
    }

    fn assert_point_close(actual: Point, expected: Point) {
        assert_close(actual.x, expected.x);
        assert_close(actual.y, expected.y);
    }

    fn assert_graph_point_close(actual: GraphPoint, expected: GraphPoint) {
        assert_close(actual.x, expected.x);
        assert_close(actual.y, expected.y);
    }

    fn assert_rect_close(actual: Rect, expected: Rect) {
        assert_close(actual.x, expected.x);
        assert_close(actual.y, expected.y);
        assert_close(actual.width, expected.width);
        assert_close(actual.height, expected.height);
    }

    fn assert_graph_rect_close(actual: GraphRect, expected: GraphRect) {
        assert_close(actual.x, expected.x);
        assert_close(actual.y, expected.y);
        assert_close(actual.width, expected.width);
        assert_close(actual.height, expected.height);
    }

    fn viewport() -> NodeGraphViewport {
        NodeGraphViewport::new(
            Rect::new(25.0, 40.0, 320.0, 240.0),
            NodeGraphPanZoom::new(GraphVector::new(12.5, -7.25), 1.5),
        )
    }

    #[test]
    fn node_graph_ids_round_trip_raw_bits() {
        assert_eq!(NodeId::from_raw(1).raw(), 1);
        assert_eq!(PortId::from_raw(2).raw(), 2);
        assert_eq!(EdgeId::from_raw(3).raw(), 3);
        assert_eq!(NodeFrameId::from_raw(4).raw(), 4);
        assert_eq!(NodeGroupId::from_raw(5).raw(), 5);
        assert_eq!(PortTypeId::from_raw(6).raw(), 6);
    }

    #[test]
    fn node_graph_descriptors_preserve_data_only_metadata() {
        let number = PortTypeId::from_raw(10);
        let vector = PortTypeId::from_raw(11);
        let output =
            PortDescriptor::new(PortId::from_raw(1), PortDirection::Output, "Color", number);
        let input =
            PortDescriptor::new(PortId::from_raw(2), PortDirection::Input, "Vector", vector)
                .with_enabled(false);
        let frame = NodeFrameDescriptor::new(
            NodeFrameId::from_raw(30),
            "Frame A",
            GraphRect::new(-10.0, -20.0, 300.0, 180.0),
        )
        .with_enabled(false);
        let group = NodeGroupDescriptor::new(
            NodeGroupId::from_raw(40),
            "Group A",
            GraphRect::new(0.0, 0.0, 200.0, 120.0),
        )
        .with_nodes(vec![NodeId::from_raw(20)])
        .with_enabled(false);
        let node = NodeDescriptor::new(
            NodeId::from_raw(20),
            "Mix",
            GraphRect::new(5.0, 10.0, 140.0, 90.0),
        )
        .with_ports(vec![output.clone(), input.clone()])
        .with_frame(frame.id)
        .with_group(group.id)
        .with_enabled(false);
        let edge = EdgeDescriptor::new(
            EdgeId::from_raw(50),
            PortEndpoint::new(node.id, output.id),
            PortEndpoint::new(NodeId::from_raw(21), PortId::from_raw(3)),
        )
        .with_enabled(false);
        let graph = NodeGraphDescriptor {
            nodes: vec![node.clone()],
            edges: vec![edge],
            frames: vec![frame.clone()],
            groups: vec![group.clone()],
        };

        assert_eq!(node.title, "Mix");
        assert_eq!(node.rect, GraphRect::new(5.0, 10.0, 140.0, 90.0));
        assert_eq!(node.ports, vec![output.clone(), input.clone()]);
        assert_eq!(node.frame, Some(frame.id));
        assert_eq!(node.group, Some(group.id));
        assert!(!node.enabled);

        assert_eq!(input.direction, PortDirection::Input);
        assert_eq!(input.label, "Vector");
        assert_eq!(input.port_type, vector);
        assert!(!input.enabled);

        assert_eq!(graph.edges[0].id, EdgeId::from_raw(50));
        assert_eq!(graph.edges[0].from, PortEndpoint::new(node.id, output.id));
        assert_eq!(
            graph.edges[0].to,
            PortEndpoint::new(NodeId::from_raw(21), PortId::from_raw(3))
        );
        assert!(!graph.edges[0].enabled);
        assert_eq!(graph.frames, vec![frame]);
        assert_eq!(graph.groups, vec![group]);
        assert_eq!(graph.validate(), Ok(()));
    }

    #[test]
    fn descriptor_validation_reports_duplicate_node_ids_deterministically() {
        let id = NodeId::from_raw(1);
        let nodes = vec![
            NodeDescriptor::new(id, "First", GraphRect::ZERO),
            NodeDescriptor::new(NodeId::from_raw(2), "Second", GraphRect::ZERO),
            NodeDescriptor::new(id, "Duplicate", GraphRect::ZERO),
        ];

        assert_eq!(
            validate_node_graph_descriptors(&nodes),
            Err(NodeGraphValidationError::DuplicateNodeId { id })
        );
    }

    #[test]
    fn descriptor_validation_reports_duplicate_port_ids_within_one_node() {
        let node_id = NodeId::from_raw(1);
        let port_id = PortId::from_raw(7);
        let port_type = PortTypeId::from_raw(10);
        let nodes = vec![
            NodeDescriptor::new(node_id, "Node", GraphRect::ZERO).with_ports(vec![
                PortDescriptor::new(port_id, PortDirection::Input, "A", port_type),
                PortDescriptor::new(port_id, PortDirection::Output, "B", port_type),
            ]),
            NodeDescriptor::new(NodeId::from_raw(2), "Other", GraphRect::ZERO).with_ports(vec![
                PortDescriptor::new(port_id, PortDirection::Input, "Scoped", port_type),
            ]),
        ];

        assert_eq!(
            validate_node_graph_descriptors(&nodes),
            Err(NodeGraphValidationError::DuplicatePortId {
                node: node_id,
                port: port_id,
            })
        );
    }

    #[test]
    fn descriptor_validation_scopes_port_ids_by_node() {
        let port_id = PortId::from_raw(7);
        let port_type = PortTypeId::from_raw(10);
        let nodes = vec![
            NodeDescriptor::new(NodeId::from_raw(1), "A", GraphRect::ZERO).with_ports(vec![
                PortDescriptor::new(port_id, PortDirection::Input, "Input", port_type),
            ]),
            NodeDescriptor::new(NodeId::from_raw(2), "B", GraphRect::ZERO).with_ports(vec![
                PortDescriptor::new(port_id, PortDirection::Output, "Output", port_type),
            ]),
        ];

        assert_eq!(validate_node_graph_descriptors(&nodes), Ok(()));
    }

    #[test]
    fn compatibility_is_directed_enabled_and_keyed_by_app_metadata() {
        let number = PortTypeId::from_raw(10);
        let vector = PortTypeId::from_raw(11);
        let output = PortDescriptor::new(PortId::from_raw(1), PortDirection::Output, "Out", number);
        let input = PortDescriptor::new(PortId::from_raw(2), PortDirection::Input, "In", number);
        let other_input =
            PortDescriptor::new(PortId::from_raw(3), PortDirection::Input, "Other", vector);
        let disabled_input =
            PortDescriptor::new(PortId::from_raw(4), PortDirection::Input, "Off", number)
                .with_enabled(false);

        assert!(ports_are_compatible(&output, &input));
        assert_eq!(validate_port_compatibility(&output, &input), Ok(()));

        assert_eq!(
            validate_port_compatibility(&input, &output),
            Err(PortCompatibilityError::DirectionMismatch {
                output: PortDirection::Input,
                input: PortDirection::Output,
            })
        );
        assert!(matches!(
            validate_port_compatibility(&output, &output),
            Err(PortCompatibilityError::DirectionMismatch { .. })
        ));
        assert!(matches!(
            validate_port_compatibility(&input, &input),
            Err(PortCompatibilityError::DirectionMismatch { .. })
        ));
        assert_eq!(
            validate_port_compatibility(&output, &other_input),
            Err(PortCompatibilityError::TypeMismatch {
                output: number,
                input: vector,
            })
        );
        assert_eq!(
            validate_port_compatibility(&output, &disabled_input),
            Err(PortCompatibilityError::DisabledPort {
                output_enabled: true,
                input_enabled: false,
            })
        );
        assert!(!ports_are_compatible(&output, &disabled_input));
    }

    #[test]
    fn pan_zoom_sanitizes_invalid_zoom_and_pan() {
        let pan_zoom =
            NodeGraphPanZoom::new(GraphVector::new(f32::NAN, f32::INFINITY), -2.0).sanitized();

        assert_close(pan_zoom.pan.x, 0.0);
        assert_close(pan_zoom.pan.y, 0.0);
        assert_close(pan_zoom.zoom, 1.0);

        for invalid_zoom in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let pan_zoom =
                NodeGraphPanZoom::new(GraphVector::new(5.0, -3.0), invalid_zoom).sanitized();

            assert_close(pan_zoom.pan.x, 5.0);
            assert_close(pan_zoom.pan.y, -3.0);
            assert_close(pan_zoom.zoom, 1.0);
        }

        let mut pan_zoom = NodeGraphPanZoom::default();
        pan_zoom.set_zoom(0.0);
        pan_zoom.pan_by(GraphVector::new(5.0, f32::NEG_INFINITY));

        assert_close(pan_zoom.zoom, 1.0);
        assert_close(pan_zoom.pan.x, 5.0);
        assert_close(pan_zoom.pan.y, 0.0);
    }

    #[test]
    fn graph_and_screen_points_round_trip_with_fractional_pan_zoom() {
        let viewport = viewport();
        let graph = GraphPoint::new(100.25, -20.5);
        let screen = viewport.graph_to_screen(graph);
        let round_trip = viewport.screen_to_graph(screen);

        assert_point_close(screen, Point::new(187.875, 1.999_999));
        assert_graph_point_close(round_trip, graph);
    }

    #[test]
    fn conversions_account_for_non_origin_viewport_bounds() {
        let viewport = NodeGraphViewport::new(
            Rect::new(100.0, 200.0, 400.0, 300.0),
            NodeGraphPanZoom::new(GraphVector::new(-25.0, 15.0), 2.0),
        );

        let screen = viewport.graph_to_screen(GraphPoint::new(10.0, 20.0));
        let graph = viewport.screen_to_graph(Point::new(75.0, 215.0));

        assert_point_close(screen, Point::new(95.0, 255.0));
        assert_graph_point_close(graph, GraphPoint::new(0.0, 0.0));
    }

    #[test]
    fn graph_and_screen_rects_round_trip() {
        let viewport = viewport();
        let graph = GraphRect::new(10.0, 20.0, 120.0, 80.0);
        let screen = viewport.graph_rect_to_screen(graph);
        let round_trip = viewport.screen_rect_to_graph(screen);

        assert_rect_close(screen, Rect::new(52.5, 62.75, 180.0, 120.0));
        assert_graph_rect_close(round_trip, graph);
    }

    #[test]
    fn graph_coordinates_and_rect_sizes_sanitize_deterministically() {
        let viewport = NodeGraphViewport::new(
            Rect::new(f32::NAN, f32::INFINITY, -10.0, f32::NAN),
            NodeGraphPanZoom::new(GraphVector::new(f32::INFINITY, f32::NAN), 0.0),
        );

        assert_rect_close(viewport.effective_bounds(), Rect::new(0.0, 0.0, 0.0, 0.0));

        let screen = viewport.graph_to_screen(GraphPoint::new(f32::NAN, f32::NEG_INFINITY));
        let graph = viewport.screen_to_graph(Point::new(f32::NAN, f32::INFINITY));
        let screen_rect = viewport.graph_rect_to_screen(GraphRect::new(
            f32::INFINITY,
            f32::NAN,
            -20.0,
            f32::NEG_INFINITY,
        ));
        let graph_rect = viewport.screen_rect_to_graph(Rect::new(
            f32::NAN,
            f32::INFINITY,
            -30.0,
            f32::NEG_INFINITY,
        ));

        assert_point_close(screen, Point::new(0.0, 0.0));
        assert_graph_point_close(graph, GraphPoint::new(0.0, 0.0));
        assert_rect_close(screen_rect, Rect::new(0.0, 0.0, 0.0, 0.0));
        assert_graph_rect_close(graph_rect, GraphRect::new(0.0, 0.0, 0.0, 0.0));
    }
}
