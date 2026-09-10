//! The list you land on, and what each row opens.

use xpui::screen::Screen;
use xpui::{List, ListRow, NavigationScreen, ScrollView, View, present};

use crate::screens;

/// Every example, in the order the menu shows them.
///
/// Adding one is a variant, an entry in `ALL` and an arm in each match
/// below. The compiler catches a missing arm; only `ALL` can silently leave
/// an example out of the menu.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Example {
    /// Every value control the chrome paints, on one screen.
    Controls,
    /// Rows with and without subtitles, and rows carrying a value.
    Lists,
    /// An option picker opened over a screen that keeps painting behind it.
    Dialogs,
    /// Twenty-four rows, more than the tallest panel here shows at once.
    Scrolling,
    /// What the faces look like at each size, and where a label is cut.
    Text,
    /// Switches every screen to another font family, at run time.
    Typeface,
    /// The heap, the frame timings and every control together.
    Developers,
}

impl Example {
    /// What the menu iterates, and what the screenshot suite renders on every
    /// board. A variant missing from here is an example nothing opens.
    pub const ALL: [Example; 7] = [
        Example::Controls,
        Example::Lists,
        Example::Dialogs,
        Example::Scrolling,
        Example::Text,
        Example::Typeface,
        Example::Developers,
    ];

    /// The row's title, and the screen's.
    pub fn title(self) -> &'static str {
        match self {
            Example::Controls => "Controls",
            Example::Lists => "Lists",
            Example::Dialogs => "Dialogs",
            Example::Scrolling => "Scrolling",
            Example::Text => "Text",
            Example::Typeface => "Typeface",
            Example::Developers => "Developers",
        }
    }

    /// The row's subtitle.
    pub fn summary(self) -> &'static str {
        match self {
            Example::Controls => "Slider, stepper, toggle, progress",
            Example::Lists => "Rows, subtitles, values",
            Example::Dialogs => "A picker over content",
            Example::Scrolling => "More than fits on a panel",
            Example::Text => "Fonts, weights and truncation",
            Example::Typeface => "Set the whole example in another face",
            Example::Developers => "Everything at once, on one screen",
        }
    }

    fn open(self) {
        match self {
            Example::Controls => present(screens::Controls::new()),
            Example::Lists => present(screens::Lists::new()),
            Example::Dialogs => present(screens::Dialogs::new()),
            Example::Scrolling => present(screens::Scrolling::new()),
            Example::Text => present(screens::TextSizes::new()),
            Example::Typeface => present(crate::Typefaces::new(crate::fonts::FAMILIES)),
            Example::Developers => present(crate::DevelopersScreen::new()),
        };
    }
}

/// The gallery's root screen.
pub struct Menu {
    /// The last example opened, so a test can assert the row did something.
    pub opened: Option<Example>,
}

impl Default for Menu {
    fn default() -> Self {
        Menu::new()
    }
}

impl Menu {
    /// The menu with nothing opened yet.
    pub fn new() -> Self {
        Menu { opened: None }
    }
}

impl Screen for Menu {
    type Message = Example;

    fn body(&self) -> impl View<Self::Message> {
        NavigationScreen::new(ScrollView::new(List::new().extend(
            Example::ALL.into_iter().map(|example| {
                ListRow::new(example.title())
                    .subtitle(example.summary())
                    .on_tap(example)
            }),
        )))
        .title("xpui examples")
    }

    fn update(&mut self, message: Self::Message) {
        self.opened = Some(message);
        message.open();
    }

    fn title(&self) -> Option<&'static str> {
        Some("xpui examples")
    }
}
