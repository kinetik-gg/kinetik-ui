//! Text layout, editing state, and engine adapters for Kinetik UI.

use std::collections::HashMap;

use cosmic_text::FontSystem;
use kinetik_ui_core::{Key, KeyEvent, KeyState, Size, TextInputEvent};

/// Font properties used by text measurement and layout.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextStyle {
    /// Font family name.
    pub family: String,
    /// Font size in logical units.
    pub size_bits: u32,
    /// Line height in logical units.
    pub line_height_bits: u32,
}

impl TextStyle {
    /// Creates a text style from logical sizes.
    #[must_use]
    pub fn new(family: impl Into<String>, size: f32, line_height: f32) -> Self {
        Self {
            family: family.into(),
            size_bits: size.to_bits(),
            line_height_bits: line_height.to_bits(),
        }
    }

    /// Returns the font size.
    #[must_use]
    pub const fn size(&self) -> f32 {
        f32::from_bits(self.size_bits)
    }

    /// Returns the line height.
    #[must_use]
    pub const fn line_height(&self) -> f32 {
        f32::from_bits(self.line_height_bits)
    }
}

/// Request for measuring or laying out text.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextLayoutKey {
    /// Text content.
    pub text: String,
    /// Style.
    pub style: TextStyle,
    /// Maximum width in logical units.
    pub width_bits: u32,
    /// Whether text may wrap.
    pub wrap: bool,
}

impl TextLayoutKey {
    /// Creates a text layout key.
    #[must_use]
    pub fn new(text: impl Into<String>, style: TextStyle, width: f32, wrap: bool) -> Self {
        Self {
            text: text.into(),
            style,
            width_bits: width.to_bits(),
            wrap,
        }
    }

    /// Returns the maximum width.
    #[must_use]
    pub const fn width(&self) -> f32 {
        f32::from_bits(self.width_bits)
    }
}

/// A measured text run.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextLayout {
    /// Logical size of the laid out text.
    pub size: Size,
    /// Number of visible lines.
    pub line_count: usize,
}

/// Text layout cache.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TextLayoutCache {
    layouts: HashMap<TextLayoutKey, TextLayout>,
}

impl TextLayoutCache {
    /// Creates an empty text layout cache.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a cached layout.
    #[must_use]
    pub fn get(&self, key: &TextLayoutKey) -> Option<TextLayout> {
        self.layouts.get(key).copied()
    }

    /// Inserts a cached layout.
    pub fn insert(&mut self, key: TextLayoutKey, layout: TextLayout) {
        self.layouts.insert(key, layout);
    }

    /// Returns an existing layout or inserts a newly measured layout.
    pub fn get_or_measure(&mut self, key: TextLayoutKey) -> TextLayout {
        if let Some(layout) = self.get(&key) {
            layout
        } else {
            let layout = fallback_measure(&key);
            self.insert(key, layout);
            layout
        }
    }

    /// Clears all cached layouts.
    pub fn clear(&mut self) {
        self.layouts.clear();
    }

    /// Returns the number of cached entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.layouts.len()
    }

    /// Returns true when the cache is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.layouts.is_empty()
    }
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
fn fallback_measure(key: &TextLayoutKey) -> TextLayout {
    let line_height = key.style.line_height();
    let char_width = key.style.size() * 0.55;
    let raw_width = key.text.chars().count() as f32 * char_width;
    let line_count = if key.wrap && key.width() > 0.0 && raw_width > key.width() {
        (raw_width / key.width()).ceil() as usize
    } else {
        1
    };
    let width = if key.wrap {
        raw_width.min(key.width()).max(0.0)
    } else {
        raw_width
    };

    TextLayout {
        size: Size::new(width, line_height * line_count as f32),
        line_count,
    }
}

/// Cosmic-text backed engine handle.
pub struct CosmicTextEngine {
    font_system: FontSystem,
}

impl CosmicTextEngine {
    /// Creates a cosmic-text engine.
    #[must_use]
    pub fn new() -> Self {
        Self {
            font_system: FontSystem::new(),
        }
    }

    /// Returns access to the underlying font system for renderer adapters.
    pub fn font_system(&mut self) -> &mut FontSystem {
        &mut self.font_system
    }
}

impl Default for CosmicTextEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Selection range in byte offsets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextSelection {
    /// Anchor byte offset.
    pub anchor: usize,
    /// Active byte offset.
    pub active: usize,
}

impl TextSelection {
    /// Creates a selection.
    #[must_use]
    pub const fn new(anchor: usize, active: usize) -> Self {
        Self { anchor, active }
    }

    /// Returns the sorted selection range.
    #[must_use]
    pub fn range(self) -> core::ops::Range<usize> {
        self.anchor.min(self.active)..self.anchor.max(self.active)
    }

    /// Returns true when the selection is collapsed.
    #[must_use]
    pub const fn is_caret(self) -> bool {
        self.anchor == self.active
    }
}

/// Editable single-line text state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextEditState {
    /// Text buffer.
    pub text: String,
    /// Current selection.
    pub selection: TextSelection,
    undo: TextUndoStack,
}

impl TextEditState {
    /// Creates text editing state.
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        let caret = text.len();
        Self {
            text,
            selection: TextSelection::new(caret, caret),
            undo: TextUndoStack::new(),
        }
    }

    /// Returns the caret byte offset.
    #[must_use]
    pub const fn caret(&self) -> usize {
        self.selection.active
    }

    /// Sets a collapsed caret.
    pub fn set_caret(&mut self, caret: usize) {
        let caret = clamp_boundary(&self.text, caret);
        self.selection = TextSelection::new(caret, caret);
    }

    /// Applies committed text input.
    pub fn insert_text(&mut self, text: &str) {
        self.record_undo();
        self.replace_selection(text);
    }

    /// Deletes backward from the current selection or caret.
    pub fn backspace(&mut self) {
        if !self.selection.is_caret() {
            self.record_undo();
            self.replace_selection("");
        } else if let Some(previous) = previous_boundary(&self.text, self.caret()) {
            self.record_undo();
            self.text.replace_range(previous..self.caret(), "");
            self.set_caret(previous);
        }
    }

    /// Deletes forward from the current selection or caret.
    pub fn delete_forward(&mut self) {
        if !self.selection.is_caret() {
            self.record_undo();
            self.replace_selection("");
        } else if let Some(next) = next_boundary(&self.text, self.caret()) {
            self.record_undo();
            let caret = self.caret();
            self.text.replace_range(caret..next, "");
            self.set_caret(caret);
        }
    }

    /// Moves the caret left.
    pub fn move_left(&mut self) {
        if let Some(previous) = previous_boundary(&self.text, self.caret()) {
            self.set_caret(previous);
        }
    }

    /// Moves the caret right.
    pub fn move_right(&mut self) {
        if let Some(next) = next_boundary(&self.text, self.caret()) {
            self.set_caret(next);
        }
    }

    /// Applies text and key events from a frame.
    pub fn apply_input(&mut self, text_events: &[TextInputEvent], key_events: &[KeyEvent]) {
        for event in text_events {
            if let TextInputEvent::Commit(text) = event {
                self.insert_text(text);
            }
        }
        for event in key_events {
            if event.state != KeyState::Pressed {
                continue;
            }
            match event.key {
                Key::Backspace => self.backspace(),
                Key::Delete => self.delete_forward(),
                Key::ArrowLeft => self.move_left(),
                Key::ArrowRight => self.move_right(),
                _ => {}
            }
        }
    }

    /// Performs local undo.
    pub fn undo(&mut self) -> bool {
        if let Some(previous) = self.undo.undo(EditSnapshot::from_state(self)) {
            self.restore(previous);
            true
        } else {
            false
        }
    }

    /// Performs local redo.
    pub fn redo(&mut self) -> bool {
        if let Some(next) = self.undo.redo(EditSnapshot::from_state(self)) {
            self.restore(next);
            true
        } else {
            false
        }
    }

    fn replace_selection(&mut self, replacement: &str) {
        let range = self.selection.range();
        self.text.replace_range(range.clone(), replacement);
        self.set_caret(range.start + replacement.len());
    }

    fn record_undo(&mut self) {
        self.undo.push(EditSnapshot::from_state(self));
    }

    fn restore(&mut self, snapshot: EditSnapshot) {
        self.text = snapshot.text;
        self.selection = snapshot.selection;
    }
}

/// Text-field-local undo/redo history.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextUndoStack {
    undo: Vec<EditSnapshot>,
    redo: Vec<EditSnapshot>,
}

impl TextUndoStack {
    /// Creates an empty undo stack.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
        }
    }

    /// Pushes a new undo snapshot and clears redo history.
    fn push(&mut self, snapshot: EditSnapshot) {
        if self.undo.last() != Some(&snapshot) {
            self.undo.push(snapshot);
            self.redo.clear();
        }
    }

    /// Returns the previous snapshot and stores the current snapshot for redo.
    fn undo(&mut self, current: EditSnapshot) -> Option<EditSnapshot> {
        let previous = self.undo.pop()?;
        self.redo.push(current);
        Some(previous)
    }

    /// Returns the redo snapshot and stores the current snapshot for undo.
    fn redo(&mut self, current: EditSnapshot) -> Option<EditSnapshot> {
        let next = self.redo.pop()?;
        self.undo.push(current);
        Some(next)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EditSnapshot {
    text: String,
    selection: TextSelection,
}

impl EditSnapshot {
    fn from_state(state: &TextEditState) -> Self {
        Self {
            text: state.text.clone(),
            selection: state.selection,
        }
    }
}

fn clamp_boundary(text: &str, offset: usize) -> usize {
    let offset = offset.min(text.len());
    if text.is_char_boundary(offset) {
        offset
    } else {
        text.char_indices()
            .map(|(index, _)| index)
            .take_while(|index| *index < offset)
            .last()
            .unwrap_or(0)
    }
}

fn previous_boundary(text: &str, offset: usize) -> Option<usize> {
    text.char_indices()
        .map(|(index, _)| index)
        .take_while(|index| *index < offset)
        .last()
}

fn next_boundary(text: &str, offset: usize) -> Option<usize> {
    text.char_indices()
        .map(|(index, _)| index)
        .find(|index| *index > offset)
        .or_else(|| (offset < text.len()).then_some(text.len()))
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::{
        CosmicTextEngine, TextEditState, TextLayoutCache, TextLayoutKey, TextSelection, TextStyle,
    };
    use kinetik_ui_core::{Key, KeyEvent, KeyState, Modifiers, TextInputEvent};

    #[test]
    fn creates_cosmic_text_engine() {
        let mut engine = CosmicTextEngine::new();

        let _ = engine.font_system();
    }

    #[test]
    fn cache_returns_hits_and_can_invalidate() {
        let style = TextStyle::new("Inter", 12.0, 16.0);
        let key = TextLayoutKey::new("hello", style, 100.0, false);
        let mut cache = TextLayoutCache::new();

        let first = cache.get_or_measure(key.clone());
        let second = cache.get_or_measure(key);

        assert_eq!(cache.len(), 1);
        assert_eq!(first, second);
        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn wrapped_measurement_increases_line_count() {
        let style = TextStyle::new("Inter", 10.0, 14.0);
        let key = TextLayoutKey::new("long text string", style, 10.0, true);
        let mut cache = TextLayoutCache::new();

        let layout = cache.get_or_measure(key);

        assert!(layout.line_count > 1);
    }

    #[test]
    fn inserts_text_at_caret() {
        let mut state = TextEditState::new("ab");
        state.set_caret(1);

        state.insert_text("X");

        assert_eq!(state.text, "aXb");
        assert_eq!(state.caret(), 2);
    }

    #[test]
    fn replaces_selection() {
        let mut state = TextEditState::new("abcd");
        state.selection = TextSelection::new(1, 3);

        state.insert_text("X");

        assert_eq!(state.text, "aXd");
        assert_eq!(state.caret(), 2);
    }

    #[test]
    fn applies_text_and_key_events() {
        let mut state = TextEditState::new("");

        state.apply_input(&[TextInputEvent::Commit("a".to_owned())], &[]);
        state.apply_input(
            &[],
            &[KeyEvent::new(
                Key::Backspace,
                KeyState::Pressed,
                Modifiers::default(),
                false,
            )],
        );

        assert_eq!(state.text, "");
    }

    #[test]
    fn moves_caret_by_character_boundaries() {
        let mut state = TextEditState::new("aé");

        state.move_left();
        assert_eq!(state.caret(), 1);
        state.move_right();
        assert_eq!(state.caret(), 3);
    }

    #[test]
    fn undo_and_redo_are_local_to_text_state() {
        let mut state = TextEditState::new("");

        state.insert_text("a");
        state.insert_text("b");
        assert_eq!(state.text, "ab");

        assert!(state.undo());
        assert_eq!(state.text, "a");
        assert!(state.redo());
        assert_eq!(state.text, "ab");
    }
}
