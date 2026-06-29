//! Timeline ruler and coordinate contract conformance tests.

mod timeline_conformance {
    use kinetik_ui_widgets::{
        TimelineFrame, TimelineFrameRate, TimelineFrameRounding, TimelineId, TimelineRange,
        TimelineRulerId, TimelineRulerTickKind, TimelineRulerTickRequest, TimelineScale,
        TimelineTime, TimelineZoom, TransportControlId, clamp_timeline_scroll_offset,
        max_timeline_scroll_offset, sanitize_timeline_zoom, timeline_timecode_label,
    };

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() <= 0.001,
            "expected {actual} to equal {expected}"
        );
    }

    fn assert_seconds_close(actual: TimelineTime, expected: f64) {
        assert!(
            (actual.seconds() - expected).abs() <= 0.000_001,
            "expected {} to equal {expected}",
            actual.seconds()
        );
    }

    fn scale() -> TimelineScale {
        TimelineScale::new(
            10.0,
            400.0,
            TimelineRange::seconds(0.0, 10.0),
            TimelineZoom::new(100.0),
            250.0,
        )
    }

    #[test]
    fn timeline_ids_round_trip_raw_bits() {
        assert_eq!(TimelineId::from_raw(1).raw(), 1);
        assert_eq!(TimelineRulerId::from_raw(2).raw(), 2);
        assert_eq!(TransportControlId::from_raw(3).raw(), 3);
    }

    #[test]
    fn integer_frame_rate_converts_time_and_frames() {
        let rate = TimelineFrameRate::integer(24);

        assert_seconds_close(rate.frame_to_time(TimelineFrame::from_raw(48)), 2.0);
        assert_eq!(
            rate.time_to_frame(
                TimelineTime::from_seconds(2.5),
                TimelineFrameRounding::Nearest,
            ),
            TimelineFrame::from_raw(60)
        );
        assert_eq!(
            rate.time_to_frame(
                TimelineTime::from_seconds(1.49),
                TimelineFrameRounding::Floor,
            ),
            TimelineFrame::from_raw(35)
        );
        assert_eq!(
            rate.time_to_frame(
                TimelineTime::from_seconds(1.49),
                TimelineFrameRounding::Ceil,
            ),
            TimelineFrame::from_raw(36)
        );
        assert_eq!(
            rate.time_to_frame(
                TimelineTime::from_seconds(-1.49),
                TimelineFrameRounding::Truncate,
            ),
            TimelineFrame::from_raw(-35)
        );
    }

    #[test]
    fn fractional_frame_rate_preserves_rational_metadata() {
        let rate = TimelineFrameRate::new(24_000, 1001);

        assert_seconds_close(rate.frame_to_time(TimelineFrame::from_raw(24_000)), 1001.0);
        assert_eq!(
            rate.time_to_frame(
                TimelineTime::from_seconds(1001.0),
                TimelineFrameRounding::Nearest,
            ),
            TimelineFrame::from_raw(24_000)
        );
    }

    #[test]
    fn time_and_frame_screen_conversions_round_trip() {
        let x = scale().time_to_screen_x(TimelineTime::from_seconds(4.0));
        let time = scale().screen_x_to_time(x);
        let frame_x =
            scale().frame_to_screen_x(TimelineFrameRate::integer(24), TimelineFrame::from_raw(96));
        let frame = scale().screen_x_to_frame(
            TimelineFrameRate::integer(24),
            frame_x,
            TimelineFrameRounding::Nearest,
        );

        assert_close(x, 160.0);
        assert_seconds_close(time, 4.0);
        assert_close(frame_x, 160.0);
        assert_eq!(frame, TimelineFrame::from_raw(96));
    }

    #[test]
    fn visible_range_content_width_and_scroll_clamp_are_deterministic() {
        let range = TimelineRange::seconds(20.0, 0.0);
        let zoom = TimelineZoom::new(50.0);

        assert_close(range.content_width(zoom), 1000.0);
        assert_close(max_timeline_scroll_offset(range, zoom, 100.0), 900.0);
        assert_close(clamp_timeline_scroll_offset(-10.0, 900.0), 0.0);
        assert_close(clamp_timeline_scroll_offset(f32::INFINITY, 900.0), 0.0);
        assert_close(clamp_timeline_scroll_offset(1200.0, 900.0), 900.0);

        let visible = scale().visible_range();
        assert_seconds_close(visible.start, 2.5);
        assert_seconds_close(visible.end, 6.5);
    }

    #[test]
    fn zoom_scroll_and_non_finite_inputs_sanitize() {
        assert_close(sanitize_timeline_zoom(f32::NAN), 100.0);
        assert_close(sanitize_timeline_zoom(-1.0), 100.0);
        assert_close(sanitize_timeline_zoom(0.000_000_1), 0.001);
        assert_close(sanitize_timeline_zoom(f32::MAX), 1_000_000.0);

        let sanitized = TimelineScale::new(
            f32::NAN,
            f32::INFINITY,
            TimelineRange::seconds(f64::NAN, 2.0),
            TimelineZoom::new(f32::NAN),
            f32::NEG_INFINITY,
        )
        .sanitized();

        assert_close(sanitized.origin_x, 0.0);
        assert_close(sanitized.viewport_width, 0.0);
        assert_seconds_close(sanitized.content_range.start, 0.0);
        assert_seconds_close(sanitized.content_range.end, 2.0);
        assert_close(sanitized.zoom.pixels_per_second, 100.0);
        assert_close(sanitized.scroll_offset, 0.0);
    }

    #[test]
    fn ruler_ticks_are_deterministic_finite_and_ordered() {
        let request = TimelineRulerTickRequest::new(
            TimelineRange::seconds(0.0, 5.0),
            TimelineFrameRate::integer(24),
            TimelineZoom::new(120.0),
        );

        let first = request.ticks();
        let second = request.ticks();

        assert_eq!(first, second);
        assert!(!first.is_empty());
        assert!(
            first
                .iter()
                .all(|tick| tick.time(request.frame_rate).seconds().is_finite())
        );
        assert!(first.windows(2).all(|pair| pair[0].frame < pair[1].frame));
        assert!(
            first
                .iter()
                .any(|tick| tick.kind == TimelineRulerTickKind::Major)
        );
        assert!(
            first
                .iter()
                .any(|tick| tick.kind == TimelineRulerTickKind::Minor)
        );
        assert!(first.iter().any(|tick| {
            tick.kind == TimelineRulerTickKind::Major && tick.label == "00:00:00:00"
        }));
        assert!(
            first
                .iter()
                .filter(|tick| tick.kind == TimelineRulerTickKind::Minor)
                .all(|tick| tick.label.is_empty())
        );
    }

    #[test]
    fn ruler_ticks_respect_max_tick_bound_for_large_ranges() {
        let ticks = TimelineRulerTickRequest::new(
            TimelineRange::seconds(0.0, 1_000_000.0),
            TimelineFrameRate::integer(24),
            TimelineZoom::new(1_000_000.0),
        )
        .with_max_ticks(128)
        .ticks();

        assert!(ticks.len() <= 128);
        assert!(ticks.windows(2).all(|pair| pair[0].frame < pair[1].frame));
    }

    #[test]
    fn timecode_labels_are_stable_for_positive_negative_and_fractional_rates() {
        assert_eq!(
            timeline_timecode_label(TimelineFrame::from_raw(49), TimelineFrameRate::integer(24)),
            "00:00:02:01"
        );
        assert_eq!(
            timeline_timecode_label(TimelineFrame::from_raw(-25), TimelineFrameRate::integer(24)),
            "-00:00:01:01"
        );
        assert_eq!(
            timeline_timecode_label(
                TimelineFrame::from_raw(30),
                TimelineFrameRate::new(30_000, 1001),
            ),
            "00:00:01:00"
        );
    }
}
