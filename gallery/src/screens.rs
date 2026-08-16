//! One screen per example.
//!
//! Each is a complete `Screen`: a `Message` enum, a `body()` describing what
//! it looks like, and an `update()` that is the only place its state changes.
//! Read them in order — `Controls` is the shortest useful one.

use alloc::format;
use alloc::string::String;

use xpui::screen::Screen;
use xpui::{
    Divider, Font, Hint, List, ListRow, Modal, NavigationScreen, ProgressBar, Scrim, ScrollView,
    Section, Slider, Stepper, Text, Toggle, View, hstack, vstack,
};

// -- controls --------------------------------------------------------------

/// Everything adjustable, on one page.
pub struct Controls {
    brightness: i32,
    warmth: i32,
    frontlight: bool,
    downloaded: u32,
    /// The two percentages, already formatted.
    ///
    /// `body()` runs on every paint *and* on every frame that carries input,
    /// so a `format!` in there allocates several times a second and drags
    /// `core::fmt` into the binary. Building the string in `update()` costs
    /// one allocation per actual change instead.
    labels: Labels,
}

#[derive(Clone)]
struct Labels {
    brightness: String,
    warmth: String,
}

impl Labels {
    fn of(brightness: i32, warmth: i32) -> Self {
        Labels {
            brightness: format!("{brightness}%"),
            warmth: format!("{warmth}%"),
        }
    }
}

#[derive(Clone, Copy)]
pub enum ControlsMsg {
    Brightness(i32),
    BrightnessStep(i32),
    Warmth(i32),
    Frontlight(bool),
}

impl Default for Controls {
    fn default() -> Self {
        Controls::new()
    }
}

impl Controls {
    pub fn new() -> Self {
        Controls {
            brightness: 60,
            warmth: 25,
            frontlight: true,
            downloaded: 42,
            labels: Labels::of(60, 25),
        }
    }
}

impl Screen for Controls {
    type Message = ControlsMsg;

    fn body(&self) -> impl View<Self::Message> {
        NavigationScreen::new(vstack![14;
            // A stepper is one focus stop but three touch targets: the two
            // glyphs nudge, the track sets an absolute value.
            label("Brightness", &self.labels.brightness),
            Stepper::new(self.brightness)
                .on_change(ControlsMsg::Brightness)
                .on_step(ControlsMsg::BrightnessStep),

            label("Warmth", &self.labels.warmth),
            Slider::new(self.warmth, 100).on_change(ControlsMsg::Warmth),

            Divider::new(),

            Toggle::new("Frontlight", self.frontlight, "On", "Off")
                .on_change(ControlsMsg::Frontlight),

            Text::new("Downloading").font(Font::ui_small()),
            ProgressBar::percent(self.downloaded),
        ])
        .title("Controls")
        .hints(
            Hint::Standard,
            Hint::Standard,
            Hint::text("-"),
            Hint::text("+"),
        )
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            ControlsMsg::Brightness(value) => self.brightness = value.clamp(0, 100),
            ControlsMsg::BrightnessStep(delta) => {
                self.brightness = (self.brightness + delta).clamp(0, 100)
            }
            ControlsMsg::Warmth(value) => self.warmth = value.clamp(0, 100),
            // The toggle hands over the state it is moving to, so no screen
            // ever writes `!self.something`.
            ControlsMsg::Frontlight(next) => self.frontlight = next,
        }
        self.labels = Labels::of(self.brightness, self.warmth);
    }

    fn title(&self) -> Option<&'static str> {
        Some("Controls")
    }
}

/// A name on the left and its value on the right — the row every settings
/// screen is made of, and a plain function is a first-class component.
fn label<M: Clone + 'static>(name: &str, value: &str) -> impl View<M> + use<M> {
    hstack![8;
        Text::new(name),
        xpui::Spacer::new(),
        Text::new(value).bold(),
    ]
}

// -- lists -----------------------------------------------------------------

pub struct Lists {
    pub chosen: Option<usize>,
}

impl Default for Lists {
    fn default() -> Self {
        Lists::new()
    }
}

impl Lists {
    pub fn new() -> Self {
        Lists { chosen: None }
    }
}

impl Screen for Lists {
    type Message = usize;

    fn body(&self) -> impl View<Self::Message> {
        NavigationScreen::new(vstack![12;
            Section::new("One line", List::new()
                .push(ListRow::new("Wi-Fi").value("Off").on_tap(0))
                .push(ListRow::new("Bluetooth").value("On").on_tap(1))),

            // A subtitle anywhere makes every row in that list taller, which
            // is how a themed list keeps its rows uniform.
            Section::new("With subtitles", List::new()
                .push(ListRow::new("Storage").subtitle("3.1 GB free").value("32 GB").on_tap(2))
                .push(ListRow::new("Battery").subtitle("Charging").value("72%").on_tap(3))),
        ])
        .title("Lists")
    }

    fn update(&mut self, message: Self::Message) {
        self.chosen = Some(message);
    }

    fn title(&self) -> Option<&'static str> {
        Some("Lists")
    }
}

// -- dialogs ---------------------------------------------------------------

pub struct Dialogs {
    open: bool,
    font: usize,
}

#[derive(Clone, Copy)]
pub enum DialogMsg {
    Open,
    Chose(usize),
    Dismiss,
}

const FONTS: [&str; 4] = ["Serif", "Sans", "Mono", "Slab"];

impl Default for Dialogs {
    fn default() -> Self {
        Dialogs::new()
    }
}

impl Dialogs {
    pub fn new() -> Self {
        Dialogs {
            open: false,
            font: 0,
        }
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn chosen(&self) -> &'static str {
        FONTS[self.font]
    }
}

impl Screen for Dialogs {
    type Message = DialogMsg;

    fn body(&self) -> impl View<Self::Message> {
        NavigationScreen::new(
            List::new().push(
                ListRow::new("Typeface")
                    .value(self.chosen())
                    .on_tap(DialogMsg::Open),
            ),
        )
        .title("Dialogs")
        // A dialog captures input: nothing behind it can be reached, and the
        // side buttons walk its options rather than the list underneath. A
        // screen decides only whether it is in the tree.
        .overlay_if(
            self.open,
            Modal::picker("Typeface", FONTS)
                .selected(self.font)
                .on_select(DialogMsg::Chose)
                .scrim(Scrim::Dim),
        )
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            DialogMsg::Open => self.open = true,
            DialogMsg::Chose(index) => {
                self.font = index.min(FONTS.len() - 1);
                self.open = false;
            }
            DialogMsg::Dismiss => self.open = false,
        }
    }

    fn on_background_tap(&self, _point: xpui::Point) -> Option<Self::Message> {
        self.open.then_some(DialogMsg::Dismiss)
    }

    fn title(&self) -> Option<&'static str> {
        Some("Dialogs")
    }
}

// -- scrolling -------------------------------------------------------------

pub struct Scrolling;

impl Default for Scrolling {
    fn default() -> Self {
        Scrolling::new()
    }
}

impl Scrolling {
    pub fn new() -> Self {
        Scrolling
    }
}

impl Screen for Scrolling {
    type Message = usize;

    fn body(&self) -> impl View<Self::Message> {
        // Deliberately longer than any panel. The runtime scrolls to keep the
        // focused row visible; the screen never tracks an offset.
        let rows = (0..24).map(|index| {
            ListRow::new(ROWS[index % ROWS.len()])
                .value(if index % 3 == 0 { "On" } else { "Off" })
                .on_tap(index)
        });

        NavigationScreen::new(ScrollView::new(vstack![10;
            Section::new("A long list", List::new().extend(rows)),
        ]))
        .title("Scrolling")
    }

    fn update(&mut self, _message: Self::Message) {}

    fn title(&self) -> Option<&'static str> {
        Some("Scrolling")
    }
}

const ROWS: [&str; 8] = [
    "Hyphenation",
    "Justification",
    "Margins",
    "Line spacing",
    "Page turn",
    "Refresh rate",
    "Sleep timer",
    "Orientation",
];

// -- text ------------------------------------------------------------------

pub struct TextSizes;

impl Default for TextSizes {
    fn default() -> Self {
        TextSizes::new()
    }
}

impl TextSizes {
    pub fn new() -> Self {
        TextSizes
    }
}

impl Screen for TextSizes {
    type Message = ();

    fn body(&self) -> impl View<Self::Message> {
        NavigationScreen::new(vstack![10;
            Text::new("Reader").font(Font::reader()),
            Text::new("Interface").font(Font::ui()),
            Text::new("Interface bold").font(Font::ui().bold()),
            Text::new("Small").font(Font::ui_small()),
            Divider::new(),
            // Wider than the panel on purpose: a label that would overrun is
            // cut on a character boundary and given an ellipsis, rather than
            // running into whatever sits beside it.
            Text::new("A line long enough that it cannot possibly fit across the panel"),
        ])
        .title("Text")
    }

    fn update(&mut self, _message: Self::Message) {}

    fn title(&self) -> Option<&'static str> {
        Some("Text")
    }
}
