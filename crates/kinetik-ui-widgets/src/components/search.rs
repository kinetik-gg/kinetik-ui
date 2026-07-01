use super::{
    Rect, SemanticRole, TextEditState, TextFieldOutput, TextLayoutStore, Theme, UiInput, UiMemory,
    WidgetId, text_field_with_text_layouts_and_caret_visibility,
};

/// Output emitted by search fields.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchFieldOutput {
    /// Text field output.
    pub field: TextFieldOutput,
    /// Current query.
    pub query: String,
    /// Whether the query is empty.
    pub empty: bool,
}

/// Emits a search-oriented text field.
pub fn search_field(
    id: WidgetId,
    rect: Rect,
    state: &mut TextEditState,
    input: &UiInput,
    memory: &mut UiMemory,
    theme: &Theme,
    disabled: bool,
) -> SearchFieldOutput {
    search_field_with_text_layouts(id, rect, state, input, memory, theme, disabled, None)
}

/// Emits a search-oriented text field using shaped text layout cache when available.
#[allow(clippy::too_many_arguments)]
pub fn search_field_with_text_layouts(
    id: WidgetId,
    rect: Rect,
    state: &mut TextEditState,
    input: &UiInput,
    memory: &mut UiMemory,
    theme: &Theme,
    disabled: bool,
    text_layouts: Option<&mut TextLayoutStore>,
) -> SearchFieldOutput {
    search_field_with_text_layouts_and_caret_visibility(
        id,
        rect,
        state,
        input,
        memory,
        theme,
        disabled,
        text_layouts,
        true,
    )
}

/// Emits a search-oriented text field with explicit caret visibility.
#[allow(clippy::too_many_arguments)]
pub(crate) fn search_field_with_text_layouts_and_caret_visibility(
    id: WidgetId,
    rect: Rect,
    state: &mut TextEditState,
    input: &UiInput,
    memory: &mut UiMemory,
    theme: &Theme,
    disabled: bool,
    text_layouts: Option<&mut TextLayoutStore>,
    caret_visible: bool,
) -> SearchFieldOutput {
    let mut field = text_field_with_text_layouts_and_caret_visibility(
        id,
        rect,
        state,
        input,
        memory,
        theme,
        disabled,
        text_layouts,
        caret_visible,
    );
    let query = state.text.clone();
    for node in &mut field.widget.semantics {
        if node.id == id {
            node.role = SemanticRole::SearchField;
            node.label = Some("Search".to_owned());
        }
    }

    SearchFieldOutput {
        field,
        empty: query.is_empty(),
        query,
    }
}
