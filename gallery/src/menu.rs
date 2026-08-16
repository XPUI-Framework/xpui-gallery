//! The list you land on, and what each row opens.

use xpui::screen::Screen;
use xpui::{List, ListRow, NavigationScreen, ScrollView, View, present};

use crate::screens;

/// Every example, in the order the menu shows them.
///
/// One table rather than a match at each site, so adding an example is one
/// line and the menu, the count and the opener cannot disagree.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Example {
    Controls,
    Lists,
    Dialogs,
    Scrolling,
    Text,
}

impl Example {
    pub const ALL: [Example; 5] = [
        Example::Controls,
        Example::Lists,
        Example::Dialogs,
        Example::Scrolling,
        Example::Text,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Example::Controls => "Controls",
            Example::Lists => "Lists",
            Example::Dialogs => "Dialogs",
            Example::Scrolling => "Scrolling",
            Example::Text => "Text",
        }
    }

    pub fn summary(self) -> &'static str {
        match self {
            Example::Controls => "Slider, stepper, toggle, progress",
            Example::Lists => "Rows, subtitles, values",
            Example::Dialogs => "A picker over content",
            Example::Scrolling => "More than fits on a panel",
            Example::Text => "Fonts, weights and truncation",
        }
    }

    /// Pushes this example onto the stack.
    fn open(self) {
        match self {
            Example::Controls => present(screens::Controls::new()),
            Example::Lists => present(screens::Lists::new()),
            Example::Dialogs => present(screens::Dialogs::new()),
            Example::Scrolling => present(screens::Scrolling::new()),
            Example::Text => present(screens::TextSizes::new()),
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
