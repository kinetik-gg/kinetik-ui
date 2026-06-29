//! Data-only timeline ruler, frame-rate, and coordinate contracts.

/// Default timeline ruler scale in logical pixels per second.
pub const DEFAULT_TIMELINE_PIXELS_PER_SECOND: f32 = 100.0;
/// Minimum timeline zoom in logical pixels per second.
pub const MIN_TIMELINE_PIXELS_PER_SECOND: f32 = 0.001;
/// Maximum timeline zoom in logical pixels per second.
pub const MAX_TIMELINE_PIXELS_PER_SECOND: f32 = 1_000_000.0;
/// Maximum number of ruler ticks emitted by the convenience tick generator.
pub const DEFAULT_TIMELINE_RULER_MAX_TICKS: usize = 4096;

macro_rules! timeline_id {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(u64);

        impl $name {
            /// Creates an ID from raw bits.
            #[must_use]
            pub const fn from_raw(raw: u64) -> Self {
                Self(raw)
            }

            /// Returns raw ID bits.
            #[must_use]
            pub const fn raw(self) -> u64 {
                self.0
            }
        }
    };
}

timeline_id!(TimelineId, "Stable identity for a timeline surface.");
timeline_id!(
    TimelineRulerId,
    "Stable identity for a timeline ruler surface."
);
timeline_id!(
    TransportControlId,
    "Stable identity for a timeline transport control."
);

/// Timeline time in seconds.
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd)]
pub struct TimelineTime {
    seconds: f64,
}

impl TimelineTime {
    /// The timeline origin.
    pub const ZERO: Self = Self::from_seconds(0.0);

    /// Creates timeline time from seconds.
    #[must_use]
    pub const fn from_seconds(seconds: f64) -> Self {
        Self { seconds }
    }

    /// Returns raw seconds.
    #[must_use]
    pub const fn seconds(self) -> f64 {
        self.seconds
    }

    /// Returns a copy with non-finite seconds replaced by zero.
    #[must_use]
    pub fn sanitized(self) -> Self {
        Self::from_seconds(finite_f64_or_zero(self.seconds))
    }
}

/// Integer frame position.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TimelineFrame(i64);

impl TimelineFrame {
    /// Creates a frame position from raw frame bits.
    #[must_use]
    pub const fn from_raw(raw: i64) -> Self {
        Self(raw)
    }

    /// Returns the raw frame index.
    #[must_use]
    pub const fn raw(self) -> i64 {
        self.0
    }
}

/// Frame rounding policy for converting continuous time to integer frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelineFrameRounding {
    /// Round toward negative infinity.
    Floor,
    /// Round toward positive infinity.
    Ceil,
    /// Round to the nearest frame, with half values away from zero.
    Nearest,
    /// Round toward zero.
    Truncate,
}

/// Rational frame-rate metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimelineFrameRate {
    /// Frames-per-second numerator.
    pub numerator: u32,
    /// Frames-per-second denominator.
    pub denominator: u32,
}

impl TimelineFrameRate {
    /// Creates rational frame-rate metadata.
    #[must_use]
    pub const fn new(numerator: u32, denominator: u32) -> Self {
        Self {
            numerator,
            denominator,
        }
    }

    /// Creates an integer frame rate.
    #[must_use]
    pub const fn integer(frames_per_second: u32) -> Self {
        Self::new(frames_per_second, 1)
    }

    /// Returns deterministic non-zero frame-rate metadata.
    #[must_use]
    pub const fn sanitized(self) -> Self {
        Self {
            numerator: if self.numerator == 0 {
                24
            } else {
                self.numerator
            },
            denominator: if self.denominator == 0 {
                1
            } else {
                self.denominator
            },
        }
    }

    /// Returns frames per second as a finite number.
    #[must_use]
    pub fn frames_per_second(self) -> f64 {
        let rate = self.sanitized();
        f64::from(rate.numerator) / f64::from(rate.denominator)
    }

    /// Returns seconds per frame.
    #[must_use]
    pub fn seconds_per_frame(self) -> f64 {
        1.0 / self.frames_per_second()
    }

    /// Converts a frame position to timeline time.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn frame_to_time(self, frame: TimelineFrame) -> TimelineTime {
        TimelineTime::from_seconds(frame.raw() as f64 * self.seconds_per_frame()).sanitized()
    }

    /// Converts timeline time to an integer frame with the requested rounding policy.
    #[must_use]
    pub fn time_to_frame(
        self,
        time: TimelineTime,
        rounding: TimelineFrameRounding,
    ) -> TimelineFrame {
        let frame = time.sanitized().seconds() * self.frames_per_second();
        TimelineFrame::from_raw(round_frame(frame, rounding))
    }

    fn rounded_display_fps(self) -> i64 {
        let fps = self.frames_per_second().round();
        f64_to_i64_saturating(fps).max(1)
    }
}

impl Default for TimelineFrameRate {
    fn default() -> Self {
        Self::integer(24)
    }
}

/// Finite normalized timeline time range.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimelineRange {
    /// Start time.
    pub start: TimelineTime,
    /// End time.
    pub end: TimelineTime,
}

impl TimelineRange {
    /// Creates a timeline range.
    #[must_use]
    pub const fn new(start: TimelineTime, end: TimelineTime) -> Self {
        Self { start, end }
    }

    /// Creates a timeline range from seconds.
    #[must_use]
    pub const fn seconds(start: f64, end: f64) -> Self {
        Self::new(
            TimelineTime::from_seconds(start),
            TimelineTime::from_seconds(end),
        )
    }

    /// Returns a finite range with ascending endpoints.
    #[must_use]
    pub fn sanitized(self) -> Self {
        let start = self.start.sanitized().seconds();
        let end = self.end.sanitized().seconds();
        Self::seconds(start.min(end), start.max(end))
    }

    /// Returns range duration in seconds.
    #[must_use]
    pub fn duration_seconds(self) -> f64 {
        let range = self.sanitized();
        (range.end.seconds() - range.start.seconds()).max(0.0)
    }

    /// Returns true when the range has no positive duration.
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.duration_seconds() <= 0.0
    }

    /// Computes content width in logical pixels at the supplied zoom.
    #[must_use]
    pub fn content_width(self, zoom: TimelineZoom) -> f32 {
        finite_f64_to_f32(self.duration_seconds() * f64::from(zoom.sanitized().pixels_per_second))
    }
}

/// Timeline zoom in logical pixels per second.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimelineZoom {
    /// Logical pixels per timeline second.
    pub pixels_per_second: f32,
}

impl TimelineZoom {
    /// Creates timeline zoom metadata.
    #[must_use]
    pub const fn new(pixels_per_second: f32) -> Self {
        Self { pixels_per_second }
    }

    /// Returns a deterministic clamped zoom.
    #[must_use]
    pub fn sanitized(self) -> Self {
        Self {
            pixels_per_second: sanitize_timeline_zoom(self.pixels_per_second),
        }
    }

    /// Sets zoom with deterministic clamping.
    pub fn set_pixels_per_second(&mut self, pixels_per_second: f32) {
        self.pixels_per_second = sanitize_timeline_zoom(pixels_per_second);
    }
}

impl Default for TimelineZoom {
    fn default() -> Self {
        Self::new(DEFAULT_TIMELINE_PIXELS_PER_SECOND)
    }
}

/// Timeline viewport scale and horizontal scroll state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimelineScale {
    /// UI logical x coordinate of the viewport/ruler origin.
    pub origin_x: f32,
    /// Viewport width in logical units.
    pub viewport_width: f32,
    /// Timeline content range.
    pub content_range: TimelineRange,
    /// Logical pixels per second.
    pub zoom: TimelineZoom,
    /// Horizontal scroll offset in logical pixels from `content_range.start`.
    pub scroll_offset: f32,
}

impl TimelineScale {
    /// Creates timeline scale state.
    #[must_use]
    pub const fn new(
        origin_x: f32,
        viewport_width: f32,
        content_range: TimelineRange,
        zoom: TimelineZoom,
        scroll_offset: f32,
    ) -> Self {
        Self {
            origin_x,
            viewport_width,
            content_range,
            zoom,
            scroll_offset,
        }
    }

    /// Returns a copy with finite coordinates, clamped zoom, and clamped scroll.
    #[must_use]
    pub fn sanitized(self) -> Self {
        let content_range = self.content_range.sanitized();
        let zoom = self.zoom.sanitized();
        let viewport_width = finite_f32_non_negative(self.viewport_width);
        let max_scroll_offset = max_timeline_scroll_offset(content_range, zoom, viewport_width);
        Self {
            origin_x: finite_f32_or_zero(self.origin_x),
            viewport_width,
            content_range,
            zoom,
            scroll_offset: clamp_timeline_scroll_offset(self.scroll_offset, max_scroll_offset),
        }
    }

    /// Returns the maximum valid scroll offset in logical pixels.
    #[must_use]
    pub fn max_scroll_offset(self) -> f32 {
        let scale = self.sanitized();
        max_timeline_scroll_offset(scale.content_range, scale.zoom, scale.viewport_width)
    }

    /// Returns the visible time range represented by this scale.
    #[must_use]
    pub fn visible_range(self) -> TimelineRange {
        let scale = self.sanitized();
        let seconds_per_pixel = 1.0 / f64::from(scale.zoom.pixels_per_second);
        let start = scale.content_range.start.seconds()
            + f64::from(scale.scroll_offset) * seconds_per_pixel;
        let end = start + f64::from(scale.viewport_width) * seconds_per_pixel;
        TimelineRange::seconds(start, end.min(scale.content_range.end.seconds())).sanitized()
    }

    /// Converts timeline time to UI logical screen x.
    #[must_use]
    pub fn time_to_screen_x(self, time: TimelineTime) -> f32 {
        let scale = self.sanitized();
        let content_seconds = time.sanitized().seconds() - scale.content_range.start.seconds();
        finite_f64_to_f32(
            f64::from(scale.origin_x - scale.scroll_offset)
                + content_seconds * f64::from(scale.zoom.pixels_per_second),
        )
    }

    /// Converts UI logical screen x to timeline time.
    #[must_use]
    pub fn screen_x_to_time(self, x: f32) -> TimelineTime {
        let scale = self.sanitized();
        let content_x = finite_f32_or_zero(x) - scale.origin_x + scale.scroll_offset;
        TimelineTime::from_seconds(
            scale.content_range.start.seconds()
                + f64::from(content_x) / f64::from(scale.zoom.pixels_per_second),
        )
        .sanitized()
    }

    /// Converts a frame position to UI logical screen x.
    #[must_use]
    pub fn frame_to_screen_x(self, frame_rate: TimelineFrameRate, frame: TimelineFrame) -> f32 {
        self.time_to_screen_x(frame_rate.frame_to_time(frame))
    }

    /// Converts UI logical screen x to a frame position.
    #[must_use]
    pub fn screen_x_to_frame(
        self,
        frame_rate: TimelineFrameRate,
        x: f32,
        rounding: TimelineFrameRounding,
    ) -> TimelineFrame {
        frame_rate.time_to_frame(self.screen_x_to_time(x), rounding)
    }
}

/// Ruler tick role.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelineRulerTickKind {
    /// Labeled primary tick.
    Major,
    /// Unlabeled subdivision tick.
    Minor,
}

/// Stable ruler tick metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimelineRulerTick {
    /// Tick kind.
    pub kind: TimelineRulerTickKind,
    /// Tick frame.
    pub frame: TimelineFrame,
    /// Deterministic label. Minor ticks use an empty label.
    pub label: String,
}

impl TimelineRulerTick {
    /// Returns tick time for a frame rate.
    #[must_use]
    pub fn time(&self, frame_rate: TimelineFrameRate) -> TimelineTime {
        frame_rate.frame_to_time(self.frame)
    }
}

/// Ruler tick generation request.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimelineRulerTickRequest {
    /// Visible time range.
    pub visible_range: TimelineRange,
    /// Frame-rate metadata.
    pub frame_rate: TimelineFrameRate,
    /// Timeline zoom.
    pub zoom: TimelineZoom,
    /// Upper bound for emitted ticks.
    pub max_ticks: usize,
}

impl TimelineRulerTickRequest {
    /// Creates a ruler tick request.
    #[must_use]
    pub const fn new(
        visible_range: TimelineRange,
        frame_rate: TimelineFrameRate,
        zoom: TimelineZoom,
    ) -> Self {
        Self {
            visible_range,
            frame_rate,
            zoom,
            max_ticks: DEFAULT_TIMELINE_RULER_MAX_TICKS,
        }
    }

    /// Sets a maximum tick count.
    #[must_use]
    pub const fn with_max_ticks(mut self, max_ticks: usize) -> Self {
        self.max_ticks = max_ticks;
        self
    }

    /// Emits deterministic finite ruler ticks.
    #[must_use]
    pub fn ticks(self) -> Vec<TimelineRulerTick> {
        timeline_ruler_ticks(self)
    }
}

/// Computes maximum horizontal scroll offset in logical pixels.
#[must_use]
pub fn max_timeline_scroll_offset(
    content_range: TimelineRange,
    zoom: TimelineZoom,
    viewport_width: f32,
) -> f32 {
    (content_range.content_width(zoom) - finite_f32_non_negative(viewport_width)).max(0.0)
}

/// Clamps a scroll offset between zero and the supplied maximum offset.
#[must_use]
pub fn clamp_timeline_scroll_offset(scroll_offset: f32, max_scroll_offset: f32) -> f32 {
    finite_f32_non_negative(scroll_offset).min(finite_f32_non_negative(max_scroll_offset))
}

/// Clamps a pixels-per-second zoom value.
#[must_use]
pub fn sanitize_timeline_zoom(pixels_per_second: f32) -> f32 {
    if pixels_per_second.is_finite() && pixels_per_second > 0.0 {
        pixels_per_second.clamp(
            MIN_TIMELINE_PIXELS_PER_SECOND,
            MAX_TIMELINE_PIXELS_PER_SECOND,
        )
    } else {
        DEFAULT_TIMELINE_PIXELS_PER_SECOND
    }
}

/// Emits deterministic finite ruler ticks for the requested visible range.
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn timeline_ruler_ticks(request: TimelineRulerTickRequest) -> Vec<TimelineRulerTick> {
    let visible = request.visible_range.sanitized();
    if visible.is_empty() || request.max_ticks == 0 {
        return Vec::new();
    }

    let frame_rate = request.frame_rate.sanitized();
    let zoom = request.zoom.sanitized();
    let min_major_frames = ((80.0 / f64::from(zoom.pixels_per_second))
        * frame_rate.frames_per_second())
    .ceil()
    .max(1.0);
    let mut major_step = nice_frame_step(f64_to_i64_saturating(min_major_frames).max(1));
    let mut minor_step = (major_step / 5).max(1);

    let start_frame = frame_rate
        .time_to_frame(visible.start, TimelineFrameRounding::Floor)
        .raw();
    let end_frame = frame_rate
        .time_to_frame(visible.end, TimelineFrameRounding::Ceil)
        .raw();

    while tick_count(start_frame, end_frame, minor_step) > request.max_ticks {
        minor_step = major_step;
        if tick_count(start_frame, end_frame, minor_step) > request.max_ticks {
            major_step = nice_frame_step(major_step.saturating_mul(2));
            minor_step = major_step;
        }
    }

    let first = floor_to_step(start_frame, minor_step);
    let last = ceil_to_step(end_frame, minor_step);
    let mut ticks = Vec::new();
    let mut frame = first;
    while frame <= last && ticks.len() < request.max_ticks {
        let kind = if frame.rem_euclid(major_step) == 0 {
            TimelineRulerTickKind::Major
        } else {
            TimelineRulerTickKind::Minor
        };
        ticks.push(TimelineRulerTick {
            kind,
            frame: TimelineFrame::from_raw(frame),
            label: if kind == TimelineRulerTickKind::Major {
                timeline_timecode_label(TimelineFrame::from_raw(frame), frame_rate)
            } else {
                String::new()
            },
        });
        frame = frame.saturating_add(minor_step);
        if minor_step <= 0 {
            break;
        }
    }
    ticks
}

/// Returns a deterministic timecode-style label for a frame.
#[must_use]
pub fn timeline_timecode_label(frame: TimelineFrame, frame_rate: TimelineFrameRate) -> String {
    let display_fps = frame_rate.rounded_display_fps();
    let raw = frame.raw();
    let sign = if raw < 0 { "-" } else { "" };
    let frames = raw.saturating_abs();
    let frames_per_hour = display_fps.saturating_mul(3600);
    let frames_per_minute = display_fps.saturating_mul(60);
    let hours = frames / frames_per_hour;
    let minutes = (frames % frames_per_hour) / frames_per_minute;
    let seconds = (frames % frames_per_minute) / display_fps;
    let frame = frames % display_fps;

    format!("{sign}{hours:02}:{minutes:02}:{seconds:02}:{frame:02}")
}

fn finite_f32_or_zero(value: f32) -> f32 {
    if value.is_finite() { value } else { 0.0 }
}

fn finite_f32_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn finite_f64_or_zero(value: f64) -> f64 {
    if value.is_finite() { value } else { 0.0 }
}

#[allow(clippy::cast_possible_truncation)]
fn finite_f64_to_f32(value: f64) -> f32 {
    if !value.is_finite() {
        return 0.0;
    }
    value.clamp(f64::from(f32::MIN), f64::from(f32::MAX)) as f32
}

#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
fn f64_to_i64_saturating(value: f64) -> i64 {
    if !value.is_finite() {
        return 0;
    }
    value.clamp(i64::MIN as f64, i64::MAX as f64) as i64
}

fn round_frame(value: f64, rounding: TimelineFrameRounding) -> i64 {
    let rounded = match rounding {
        TimelineFrameRounding::Floor => value.floor(),
        TimelineFrameRounding::Ceil => value.ceil(),
        TimelineFrameRounding::Nearest => value.round(),
        TimelineFrameRounding::Truncate => value.trunc(),
    };
    f64_to_i64_saturating(rounded)
}

fn nice_frame_step(min_frames: i64) -> i64 {
    let min_frames = min_frames.max(1);
    let mut magnitude = 1_i64;
    while magnitude.saturating_mul(10) < min_frames {
        magnitude = magnitude.saturating_mul(10);
    }

    for multiplier in [1_i64, 2, 5, 10] {
        let step = magnitude.saturating_mul(multiplier);
        if step >= min_frames {
            return step.max(1);
        }
    }
    magnitude.saturating_mul(10).max(1)
}

fn floor_to_step(value: i64, step: i64) -> i64 {
    value.div_euclid(step.max(1)).saturating_mul(step.max(1))
}

fn ceil_to_step(value: i64, step: i64) -> i64 {
    let step = step.max(1);
    let floor = floor_to_step(value, step);
    if floor == value {
        floor
    } else {
        floor.saturating_add(step)
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn tick_count(start_frame: i64, end_frame: i64, step: i64) -> usize {
    let step = step.max(1);
    let first = floor_to_step(start_frame, step);
    let last = ceil_to_step(end_frame, step);
    if last < first {
        0
    } else {
        ((last - first) / step).saturating_add(1) as usize
    }
}
