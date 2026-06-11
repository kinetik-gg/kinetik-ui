//! Interactive showcase app state and rendering.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]

use kinetik_ui_core::{
    Brush, Color, CornerRadius, ImageId, ImagePrimitive, LinePrimitive, Point, Primitive, Rect,
    RectPrimitive, Stroke, TextPrimitive, TextureId, TexturePrimitive,
};

/// Available showcase pages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShowcasePage {
    /// Component gallery and controls.
    Components,
    /// Layout and collection primitives.
    Layout,
    /// Viewport/media surface primitives.
    Viewport,
    /// Editor-style integration demo.
    EditorDemo,
}

impl ShowcasePage {
    fn label(self) -> &'static str {
        match self {
            Self::Components => "Components",
            Self::Layout => "Layout",
            Self::Viewport => "Viewport",
            Self::EditorDemo => "Editor demo",
        }
    }
}

/// Text field focus target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusField {
    /// Search box.
    Search,
    /// Single-line text field.
    Name,
    /// Numeric field.
    Number,
}

/// Window/input snapshot consumed by the showcase.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShowcaseInput {
    /// Mouse position in logical pixels.
    pub mouse: Option<Point>,
    /// Whether the primary button is down.
    pub mouse_down: bool,
    /// Characters typed this frame.
    pub typed: Vec<char>,
    /// Backspace pressed this frame.
    pub backspace: bool,
    /// Enter pressed this frame.
    pub enter: bool,
}

/// Interactive showcase app.
#[derive(Debug, Clone, PartialEq)]
pub struct ShowcaseApp {
    page: ShowcasePage,
    previous_mouse_down: bool,
    active_slider: Option<&'static str>,
    focus: Option<FocusField>,
    action_count: u32,
    selected_row: usize,
    selected_tab: usize,
    checkbox: bool,
    toggle: bool,
    radio: usize,
    strength: f32,
    zoom: f32,
    name: String,
    number: String,
    search: String,
    status: String,
}

impl Default for ShowcaseApp {
    fn default() -> Self {
        Self {
            page: ShowcasePage::Components,
            previous_mouse_down: false,
            active_slider: None,
            focus: None,
            action_count: 0,
            selected_row: 1,
            selected_tab: 0,
            checkbox: true,
            toggle: false,
            radio: 0,
            strength: 0.62,
            zoom: 0.48,
            name: "Project".to_owned(),
            number: "42".to_owned(),
            search: "media".to_owned(),
            status: "Ready".to_owned(),
        }
    }
}

impl ShowcaseApp {
    /// Creates a showcase app.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Current page.
    #[must_use]
    pub const fn page(&self) -> ShowcasePage {
        self.page
    }

    /// Action invocation count.
    #[must_use]
    pub const fn action_count(&self) -> u32 {
        self.action_count
    }

    /// Slider value.
    #[must_use]
    pub const fn strength(&self) -> f32 {
        self.strength
    }

    /// Current search query.
    #[must_use]
    pub fn search(&self) -> &str {
        &self.search
    }

    /// Applies input and updates state.
    pub fn update(&mut self, input: &ShowcaseInput) {
        let clicked = input.mouse_down && !self.previous_mouse_down;
        if clicked {
            self.focus = None;
        }

        if let Some(mouse) = input.mouse {
            self.handle_pointer(mouse, input.mouse_down, clicked);
        }
        if !input.mouse_down {
            self.active_slider = None;
        }
        self.handle_keyboard(input);
        self.previous_mouse_down = input.mouse_down;
    }

    /// Builds the current primitive stream.
    #[must_use]
    pub fn primitives(&self) -> Vec<Primitive> {
        let mut primitives = Vec::new();
        self.chrome(&mut primitives);
        match self.page {
            ShowcasePage::Components => self.components_page(&mut primitives),
            ShowcasePage::Layout => Self::layout_page(&mut primitives),
            ShowcasePage::Viewport => self.viewport_page(&mut primitives),
            ShowcasePage::EditorDemo => Self::editor_demo_page(&mut primitives),
        }
        primitives
    }

    fn handle_pointer(&mut self, mouse: Point, down: bool, clicked: bool) {
        for (page, rect) in nav_items() {
            if clicked && rect.contains_point(mouse) {
                self.page = page;
                self.status = format!("Page: {}", page.label());
            }
        }

        if clicked && Rect::new(40.0, 144.0, 128.0, 30.0).contains_point(mouse) {
            self.action_count += 1;
            self.status = format!("Analyze clicked {}", self.action_count);
        }
        if clicked && Rect::new(188.0, 144.0, 30.0, 30.0).contains_point(mouse) {
            self.checkbox = !self.checkbox;
            self.status = format!("Checkbox: {}", self.checkbox);
        }
        if clicked && Rect::new(238.0, 148.0, 54.0, 24.0).contains_point(mouse) {
            self.toggle = !self.toggle;
            self.status = format!("Toggle: {}", self.toggle);
        }
        for (index, rect) in [
            Rect::new(320.0, 146.0, 22.0, 22.0),
            Rect::new(410.0, 146.0, 22.0, 22.0),
        ]
        .into_iter()
        .enumerate()
        {
            if clicked && rect.contains_point(mouse) {
                self.radio = index;
                self.status = format!("Radio option {}", index + 1);
            }
        }
        for (field, rect) in [
            (FocusField::Search, Rect::new(40.0, 90.0, 260.0, 30.0)),
            (FocusField::Name, Rect::new(40.0, 232.0, 220.0, 30.0)),
            (FocusField::Number, Rect::new(280.0, 232.0, 120.0, 30.0)),
        ] {
            if clicked && rect.contains_point(mouse) {
                self.focus = Some(field);
                self.status = format!("Focused {field:?}");
            }
        }
        self.slider(
            mouse,
            down,
            clicked,
            "strength",
            Rect::new(40.0, 196.0, 260.0, 18.0),
        );
        self.slider(
            mouse,
            down,
            clicked,
            "zoom",
            Rect::new(940.0, 634.0, 260.0, 18.0),
        );

        for (index, y) in [356.0, 384.0, 412.0, 440.0].into_iter().enumerate() {
            if clicked && Rect::new(40.0, y, 300.0, 24.0).contains_point(mouse) {
                self.selected_row = index;
                self.status = format!("Selected row {}", index + 1);
            }
        }
        for (index, x) in [740.0, 850.0, 960.0].into_iter().enumerate() {
            if clicked && Rect::new(x, 92.0, 100.0, 28.0).contains_point(mouse) {
                self.selected_tab = index;
                self.status = format!("Tab {}", index + 1);
            }
        }
    }

    fn slider(&mut self, mouse: Point, down: bool, clicked: bool, id: &'static str, rect: Rect) {
        if clicked && rect.contains_point(mouse) {
            self.active_slider = Some(id);
        }
        if down && self.active_slider == Some(id) {
            let value = ((mouse.x - rect.x) / rect.width).clamp(0.0, 1.0);
            match id {
                "strength" => self.strength = value,
                "zoom" => self.zoom = value,
                _ => {}
            }
            self.status = format!("{id}: {value:.2}");
        }
    }

    fn handle_keyboard(&mut self, input: &ShowcaseInput) {
        if input.enter {
            self.action_count += 1;
            self.status = format!("Enter action {}", self.action_count);
        }
        let Some(field) = self.focus else {
            return;
        };
        let target = match field {
            FocusField::Search => &mut self.search,
            FocusField::Name => &mut self.name,
            FocusField::Number => &mut self.number,
        };
        if input.backspace {
            target.pop();
        }
        for character in &input.typed {
            if field == FocusField::Number && !character.is_ascii_digit() && *character != '.' {
                continue;
            }
            target.push(*character);
        }
    }

    fn chrome(&self, primitives: &mut Vec<Primitive>) {
        rect(
            primitives,
            Rect::new(0.0, 0.0, 1440.0, 900.0),
            rgb(12, 12, 13),
            None,
        );
        rect(
            primitives,
            Rect::new(0.0, 0.0, 1440.0, 52.0),
            rgb(20, 20, 22),
            Some(rgb(58, 58, 62)),
        );
        text(
            primitives,
            20.0,
            32.0,
            "Kinetik UI Showcase",
            14.0,
            rgb(238, 238, 238),
        );
        text(
            primitives,
            1110.0,
            32.0,
            &self.status,
            11.0,
            rgb(160, 160, 164),
        );
        for (page, item) in nav_items() {
            let active = self.page == page;
            rect(
                primitives,
                item,
                if active {
                    rgb(42, 96, 224)
                } else {
                    rgb(30, 30, 33)
                },
                Some(rgb(72, 72, 76)),
            );
            text(
                primitives,
                item.x + 12.0,
                item.y + 19.0,
                page.label(),
                11.0,
                rgb(236, 236, 236),
            );
        }
    }

    fn components_page(&self, primitives: &mut Vec<Primitive>) {
        text(
            primitives,
            40.0,
            86.0,
            "Interactive controls",
            18.0,
            rgb(242, 242, 244),
        );
        input_box(
            primitives,
            Rect::new(40.0, 90.0, 260.0, 30.0),
            &self.search,
            self.focus == Some(FocusField::Search),
            "Search",
        );
        button(
            primitives,
            Rect::new(40.0, 144.0, 128.0, 30.0),
            "Analyze",
            true,
        );
        checkbox(
            primitives,
            Rect::new(188.0, 144.0, 30.0, 30.0),
            self.checkbox,
            "Check",
        );
        toggle(primitives, Rect::new(238.0, 148.0, 54.0, 24.0), self.toggle);
        radio(
            primitives,
            Rect::new(320.0, 146.0, 22.0, 22.0),
            self.radio == 0,
            "Radio A",
        );
        radio(
            primitives,
            Rect::new(410.0, 146.0, 22.0, 22.0),
            self.radio == 1,
            "Radio B",
        );
        slider(
            primitives,
            Rect::new(40.0, 196.0, 260.0, 18.0),
            self.strength,
            "Slider",
        );
        input_box(
            primitives,
            Rect::new(40.0, 232.0, 220.0, 30.0),
            &self.name,
            self.focus == Some(FocusField::Name),
            "Text field",
        );
        input_box(
            primitives,
            Rect::new(280.0, 232.0, 120.0, 30.0),
            &self.number,
            self.focus == Some(FocusField::Number),
            "Numeric",
        );
        self.collection_demo(primitives);
        Self::primitive_demo(primitives);
        self.tab_demo(primitives);
    }

    fn collection_demo(&self, primitives: &mut Vec<Primitive>) {
        text(
            primitives,
            40.0,
            332.0,
            "List, grid, and table",
            16.0,
            rgb(230, 230, 232),
        );
        for (index, y, label) in [
            (0, 356.0, "Asset: plate.mov"),
            (1, 384.0, "Asset: foreground.exr"),
            (2, 412.0, "Asset: mask.png"),
            (3, 440.0, "Asset: output.mp4"),
        ] {
            rect(
                primitives,
                Rect::new(40.0, y, 300.0, 24.0),
                if self.selected_row == index {
                    rgb(42, 96, 224)
                } else {
                    rgb(26, 26, 29)
                },
                Some(rgb(58, 58, 62)),
            );
            text(primitives, 52.0, y + 17.0, label, 10.0, rgb(232, 232, 232));
        }
        for row in 0..3 {
            for col in 0..4 {
                let x = 380.0 + col as f32 * 54.0;
                let y = 356.0 + row as f32 * 42.0;
                rect(
                    primitives,
                    Rect::new(x, y, 42.0, 30.0),
                    rgb(36, 38, 42),
                    Some(rgb(70, 70, 74)),
                );
            }
        }
        for row in 0..4 {
            for col in 0..3 {
                let x = 620.0 + col as f32 * 120.0;
                let y = 348.0 + row as f32 * 28.0;
                rect(
                    primitives,
                    Rect::new(x, y, 118.0, 26.0),
                    rgb(24, 24, 27),
                    Some(rgb(58, 58, 62)),
                );
                text(
                    primitives,
                    x + 8.0,
                    y + 17.0,
                    &format!("R{row} C{col}"),
                    9.0,
                    rgb(206, 206, 210),
                );
            }
        }
    }

    fn primitive_demo(primitives: &mut Vec<Primitive>) {
        text(
            primitives,
            40.0,
            540.0,
            "Primitives",
            16.0,
            rgb(230, 230, 232),
        );
        rect(
            primitives,
            Rect::new(40.0, 560.0, 120.0, 72.0),
            rgb(46, 48, 54),
            Some(rgb(120, 120, 126)),
        );
        primitives.push(Primitive::Line(LinePrimitive {
            from: Point::new(180.0, 560.0),
            to: Point::new(300.0, 632.0),
            stroke: Stroke::new(2.0, Brush::Solid(color(rgb(230, 230, 230)))),
        }));
        primitives.push(Primitive::Image(ImagePrimitive {
            image: ImageId::from_raw(11),
            rect: Rect::new(320.0, 560.0, 96.0, 72.0),
        }));
        text(
            primitives,
            440.0,
            595.0,
            "Text primitive",
            13.0,
            rgb(238, 238, 238),
        );
        border(
            primitives,
            Rect::new(600.0, 560.0, 140.0, 72.0),
            rgb(92, 132, 240),
        );
    }

    fn tab_demo(&self, primitives: &mut Vec<Primitive>) {
        text(
            primitives,
            740.0,
            86.0,
            "Tabs and reusable panels",
            16.0,
            rgb(230, 230, 232),
        );
        for (index, x, label) in [
            (0, 740.0, "Theme"),
            (1, 850.0, "State"),
            (2, 960.0, "Actions"),
        ] {
            rect(
                primitives,
                Rect::new(x, 92.0, 100.0, 28.0),
                if self.selected_tab == index {
                    rgb(42, 96, 224)
                } else {
                    rgb(28, 28, 31)
                },
                Some(rgb(70, 70, 74)),
            );
            text(primitives, x + 18.0, 111.0, label, 10.0, rgb(238, 238, 238));
        }
        rect(
            primitives,
            Rect::new(740.0, 120.0, 420.0, 140.0),
            rgb(22, 22, 25),
            Some(rgb(62, 62, 66)),
        );
        let body = match self.selected_tab {
            0 => "Theme tokens drive controls, panels, borders, and text.",
            1 => "Every control here mutates app state and redraws.",
            _ => "Actions can be invoked from buttons, keys, and menus.",
        };
        text(primitives, 760.0, 156.0, body, 11.0, rgb(224, 224, 226));
        text(
            primitives,
            760.0,
            190.0,
            &format!("Actions: {}", self.action_count),
            12.0,
            rgb(144, 184, 255),
        );
    }

    fn layout_page(primitives: &mut Vec<Primitive>) {
        text(
            primitives,
            40.0,
            90.0,
            "Layout primitives",
            18.0,
            rgb(242, 242, 244),
        );
        for (index, width) in [180.0, 260.0, 120.0].into_iter().enumerate() {
            rect(
                primitives,
                Rect::new(40.0, 130.0 + index as f32 * 58.0, width, 42.0),
                rgb(36, 42, 50),
                Some(rgb(90, 110, 140)),
            );
            text(
                primitives,
                54.0,
                156.0 + index as f32 * 58.0,
                "Row item",
                11.0,
                rgb(236, 236, 236),
            );
        }
        for col in 0..4 {
            for row in 0..3 {
                rect(
                    primitives,
                    Rect::new(
                        420.0 + col as f32 * 120.0,
                        130.0 + row as f32 * 88.0,
                        100.0,
                        68.0,
                    ),
                    rgb(44, 38, 52),
                    Some(rgb(120, 94, 150)),
                );
            }
        }
    }

    fn viewport_page(&self, primitives: &mut Vec<Primitive>) {
        text(
            primitives,
            40.0,
            90.0,
            "Viewport and media surfaces",
            18.0,
            rgb(242, 242, 244),
        );
        primitives.push(Primitive::Texture(TexturePrimitive {
            texture: TextureId::from_raw(99),
            rect: Rect::new(80.0, 140.0, 760.0, 430.0),
            source_size: kinetik_ui_core::Size::new(1920.0, 1080.0),
        }));
        primitives.push(Primitive::Line(LinePrimitive {
            from: Point::new(80.0, 355.0),
            to: Point::new(840.0, 355.0),
            stroke: Stroke::new(1.0, Brush::Solid(color(rgb(240, 240, 240)))),
        }));
        primitives.push(Primitive::Line(LinePrimitive {
            from: Point::new(460.0, 140.0),
            to: Point::new(460.0, 570.0),
            stroke: Stroke::new(1.0, Brush::Solid(color(rgb(240, 240, 240)))),
        }));
        slider(
            primitives,
            Rect::new(940.0, 634.0, 260.0, 18.0),
            self.zoom,
            "Viewport zoom",
        );
        text(
            primitives,
            940.0,
            600.0,
            &format!("Zoom: {:.0}%", 25.0 + self.zoom * 375.0),
            13.0,
            rgb(230, 230, 232),
        );
    }

    fn editor_demo_page(primitives: &mut Vec<Primitive>) {
        text(
            primitives,
            40.0,
            90.0,
            "Editor integration demo",
            18.0,
            rgb(242, 242, 244),
        );
        text(
            primitives,
            40.0,
            116.0,
            "This page demonstrates dense editor composition, not the default app.",
            11.0,
            rgb(180, 180, 184),
        );
        let mut demo = crate::editor_shell().primitives;
        for primitive in &mut demo {
            translate_primitive(primitive, Point::new(0.0, 40.0));
        }
        primitives.extend(demo);
    }
}

fn nav_items() -> [(ShowcasePage, Rect); 4] {
    [
        (
            ShowcasePage::Components,
            Rect::new(360.0, 12.0, 132.0, 28.0),
        ),
        (ShowcasePage::Layout, Rect::new(502.0, 12.0, 92.0, 28.0)),
        (ShowcasePage::Viewport, Rect::new(604.0, 12.0, 112.0, 28.0)),
        (
            ShowcasePage::EditorDemo,
            Rect::new(726.0, 12.0, 132.0, 28.0),
        ),
    ]
}

fn rect(primitives: &mut Vec<Primitive>, rect: Rect, fill: u32, stroke: Option<u32>) {
    primitives.push(Primitive::Rect(RectPrimitive {
        rect,
        fill: Some(Brush::Solid(color(fill))),
        stroke: stroke.map(|stroke| Stroke::new(1.0, Brush::Solid(color(stroke)))),
        radius: CornerRadius::all(0.0),
    }));
}

fn border(primitives: &mut Vec<Primitive>, rect: Rect, stroke: u32) {
    primitives.push(Primitive::Rect(RectPrimitive {
        rect,
        fill: None,
        stroke: Some(Stroke::new(1.0, Brush::Solid(color(stroke)))),
        radius: CornerRadius::all(0.0),
    }));
}

fn text(primitives: &mut Vec<Primitive>, x: f32, baseline: f32, value: &str, size: f32, fill: u32) {
    primitives.push(Primitive::Text(TextPrimitive {
        origin: Point::new(x, baseline),
        text: value.to_owned(),
        size,
        brush: Brush::Solid(color(fill)),
    }));
}

fn button(primitives: &mut Vec<Primitive>, rect_value: Rect, label: &str, enabled: bool) {
    rect(
        primitives,
        rect_value,
        if enabled {
            rgb(50, 54, 62)
        } else {
            rgb(28, 28, 30)
        },
        Some(rgb(88, 88, 94)),
    );
    text(
        primitives,
        rect_value.x + 16.0,
        rect_value.y + 20.0,
        label,
        11.0,
        rgb(238, 238, 238),
    );
}

fn checkbox(primitives: &mut Vec<Primitive>, rect_value: Rect, checked: bool, label: &str) {
    rect(
        primitives,
        rect_value,
        if checked {
            rgb(42, 96, 224)
        } else {
            rgb(30, 30, 32)
        },
        Some(rgb(88, 88, 94)),
    );
    if checked {
        text(
            primitives,
            rect_value.x + 8.0,
            rect_value.y + 21.0,
            "X",
            12.0,
            rgb(255, 255, 255),
        );
    }
    text(
        primitives,
        rect_value.x + 40.0,
        rect_value.y + 20.0,
        label,
        11.0,
        rgb(232, 232, 234),
    );
}

fn radio(primitives: &mut Vec<Primitive>, rect_value: Rect, selected: bool, label: &str) {
    rect(
        primitives,
        rect_value,
        rgb(28, 28, 31),
        Some(rgb(88, 88, 94)),
    );
    if selected {
        rect(
            primitives,
            Rect::new(rect_value.x + 6.0, rect_value.y + 6.0, 10.0, 10.0),
            rgb(42, 96, 224),
            None,
        );
    }
    text(
        primitives,
        rect_value.x + 32.0,
        rect_value.y + 17.0,
        label,
        10.0,
        rgb(232, 232, 234),
    );
}

fn toggle(primitives: &mut Vec<Primitive>, rect_value: Rect, on: bool) {
    rect(
        primitives,
        rect_value,
        if on {
            rgb(42, 96, 224)
        } else {
            rgb(48, 48, 52)
        },
        Some(rgb(88, 88, 94)),
    );
    let knob_x = if on {
        rect_value.x + rect_value.width - 21.0
    } else {
        rect_value.x + 3.0
    };
    rect(
        primitives,
        Rect::new(knob_x, rect_value.y + 3.0, 18.0, 18.0),
        rgb(238, 238, 238),
        None,
    );
}

fn slider(primitives: &mut Vec<Primitive>, rect_value: Rect, value: f32, label: &str) {
    text(
        primitives,
        rect_value.x,
        rect_value.y - 8.0,
        &format!("{label}: {value:.2}"),
        10.0,
        rgb(222, 222, 224),
    );
    rect(
        primitives,
        rect_value,
        rgb(32, 32, 35),
        Some(rgb(70, 70, 74)),
    );
    rect(
        primitives,
        Rect::new(
            rect_value.x,
            rect_value.y,
            rect_value.width * value,
            rect_value.height,
        ),
        rgb(42, 96, 224),
        None,
    );
    rect(
        primitives,
        Rect::new(
            rect_value.x + rect_value.width * value - 4.0,
            rect_value.y - 4.0,
            8.0,
            rect_value.height + 8.0,
        ),
        rgb(238, 238, 238),
        None,
    );
}

fn input_box(
    primitives: &mut Vec<Primitive>,
    rect_value: Rect,
    value: &str,
    focused: bool,
    label: &str,
) {
    text(
        primitives,
        rect_value.x,
        rect_value.y - 8.0,
        label,
        10.0,
        rgb(190, 190, 194),
    );
    rect(
        primitives,
        rect_value,
        rgb(18, 18, 20),
        Some(if focused {
            rgb(42, 96, 224)
        } else {
            rgb(72, 72, 76)
        }),
    );
    text(
        primitives,
        rect_value.x + 10.0,
        rect_value.y + 20.0,
        value,
        11.0,
        rgb(238, 238, 238),
    );
}

const fn rgb(red: u8, green: u8, blue: u8) -> u32 {
    ((red as u32) << 16) | ((green as u32) << 8) | blue as u32
}

fn color(pixel: u32) -> Color {
    let red = ((pixel >> 16) & 0xff) as f32 / 255.0;
    let green = ((pixel >> 8) & 0xff) as f32 / 255.0;
    let blue = (pixel & 0xff) as f32 / 255.0;
    Color::rgb(red, green, blue)
}

fn translate_primitive(primitive: &mut Primitive, offset: Point) {
    match primitive {
        Primitive::Rect(rect) => {
            rect.rect.x += offset.x;
            rect.rect.y += offset.y;
        }
        Primitive::Line(line) => {
            line.from.x += offset.x;
            line.from.y += offset.y;
            line.to.x += offset.x;
            line.to.y += offset.y;
        }
        Primitive::Text(text) => {
            text.origin.x += offset.x;
            text.origin.y += offset.y;
        }
        Primitive::Image(image) => {
            image.rect.x += offset.x;
            image.rect.y += offset.y;
        }
        Primitive::Texture(texture) => {
            texture.rect.x += offset.x;
            texture.rect.y += offset.y;
        }
        Primitive::ClipBegin { rect, .. } => {
            rect.x += offset.x;
            rect.y += offset.y;
        }
        Primitive::ClipEnd { .. }
        | Primitive::LayerBegin { .. }
        | Primitive::LayerEnd { .. }
        | Primitive::TransformBegin(_)
        | Primitive::TransformEnd => {}
    }
}

#[cfg(test)]
mod tests {
    use super::{FocusField, ShowcaseApp, ShowcaseInput, ShowcasePage};
    use kinetik_ui_core::Point;

    #[test]
    fn clicking_button_changes_action_state() {
        let mut app = ShowcaseApp::new();

        app.update(&ShowcaseInput {
            mouse: Some(Point::new(60.0, 150.0)),
            mouse_down: true,
            ..ShowcaseInput::default()
        });

        assert_eq!(app.action_count(), 1);
    }

    #[test]
    fn clicking_navigation_changes_page() {
        let mut app = ShowcaseApp::new();

        app.update(&ShowcaseInput {
            mouse: Some(Point::new(620.0, 20.0)),
            mouse_down: true,
            ..ShowcaseInput::default()
        });

        assert_eq!(app.page(), ShowcasePage::Viewport);
    }

    #[test]
    fn slider_drag_updates_value() {
        let mut app = ShowcaseApp::new();

        app.update(&ShowcaseInput {
            mouse: Some(Point::new(40.0, 200.0)),
            mouse_down: true,
            ..ShowcaseInput::default()
        });
        app.update(&ShowcaseInput {
            mouse: Some(Point::new(300.0, 200.0)),
            mouse_down: true,
            ..ShowcaseInput::default()
        });

        assert!(app.strength() > 0.95);
    }

    #[test]
    fn focused_search_accepts_keyboard_input() {
        let mut app = ShowcaseApp::new();

        app.update(&ShowcaseInput {
            mouse: Some(Point::new(50.0, 100.0)),
            mouse_down: true,
            ..ShowcaseInput::default()
        });
        app.update(&ShowcaseInput {
            typed: vec!['x'],
            ..ShowcaseInput::default()
        });

        assert!(app.search().ends_with('x'));
        assert_eq!(app.focus, Some(FocusField::Search));
    }

    #[test]
    fn state_changes_produce_different_frames() {
        let mut app = ShowcaseApp::new();
        let before = crate::raster::rasterize(&app.primitives(), 1440, 900);

        app.update(&ShowcaseInput {
            mouse: Some(Point::new(60.0, 150.0)),
            mouse_down: true,
            ..ShowcaseInput::default()
        });
        let after = crate::raster::rasterize(&app.primitives(), 1440, 900);

        assert_ne!(before.pixels, after.pixels);
    }
}
