//! Choosing the typeface the whole example is set in.
//!
//! A screen like any other: a list, one row per family, a tick beside the one
//! in use. What makes it worth having is what happens *after* — every screen
//! already open is re-measured and repainted in the new face, because the ids
//! the roles report change with it and nothing that was measured against the
//! old ones is still valid.
//!
//! The list is the application's, not the backend's. The backend ships one
//! family and is told which to use; everything else here was assembled by
//! [`crate::fonts`] out of faces the backend has never heard of.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use xpui::host::Font;
use xpui::screen::Screen;
use xpui::{List, ListRow, NavigationScreen, ScrollView, Text, View, vstack};
use xpui_eg::{Family, request_family};

/// A list of families, and the one the example is set in.
pub struct Typefaces {
    families: &'static [&'static Family],
    /// Which row is in use. Held here rather than asked of the backend,
    /// because a screen has a `&dyn Host` and no way to ask it about type.
    chosen: usize,
    /// How many sizes each family was cut in, already written out.
    ///
    /// Built once here rather than in `body`, which runs on every paint and
    /// every frame carrying input: `format!` allocates and pulls in
    /// `core::fmt`.
    sizes: Vec<String>,
}

impl Typefaces {
    /// Offers `families`, opening on the first.
    ///
    /// The caller passes the same slice it registered, so what the list shows
    /// and what the backend can be set to cannot drift apart.
    pub fn new(families: &'static [&'static Family]) -> Self {
        Typefaces {
            families,
            chosen: 0,
            sizes: families
                .iter()
                .map(|family| format!("{} sizes", family.tiers.len()))
                .collect(),
        }
    }

    /// The family every screen is set in. A swap takes effect at the top of
    /// the next frame, so for one frame this is what the list chose rather
    /// than what is painted.
    pub fn chosen(&self) -> &'static Family {
        self.families[self.chosen]
    }

    fn row(&self, at: usize, family: &'static Family) -> ListRow<usize> {
        ListRow::new(family.name)
            .subtitle(if at == self.chosen {
                "In use"
            } else {
                "Set everything in this"
            })
            .value(self.sizes[at].clone())
            .on_tap(at)
    }
}

impl Screen for Typefaces {
    type Message = usize;

    fn body(&self) -> impl View<Self::Message> {
        NavigationScreen::new(ScrollView::new(vstack![
            8;
            Text::new("Every screen redraws in the face you pick.").font(Font::ui_small()),
            List::new().extend(
                self.families
                    .iter()
                    .enumerate()
                    .map(|(at, family)| self.row(at, family)),
            )
        ]))
        .title("Typeface")
    }

    fn update(&mut self, message: Self::Message) {
        if message >= self.families.len() {
            return;
        }
        self.chosen = message;
        // Applied at the top of the next frame, which is the only moment
        // nothing has been measured against the face it replaces.
        request_family(self.families[message]);
    }

    fn title(&self) -> Option<&'static str> {
        Some("Typeface")
    }
}
