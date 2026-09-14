//! The pictures on the framework's reference pages for dialogs, and for text
//! and images.
//!
//! Each view is the one its page's example builds, on the X3 like every other
//! reference picture. A dialog is a whole panel, because what it does to the
//! screen behind it is the point; text, a bitmap and icons are cropped to the
//! component. The shared machinery is in `support/`.
//!
//! ```bash
//! UPDATE_SNAPSHOTS=1 cargo test -p xpui-gallery --test reference_dialogs
//! open gallery/tests/screenshots/reference/
//! ```
//!
//! **Open every image before committing it.** A golden here is shown to every
//! reader of the page, not only compared.

mod support;

use support::{Failures, Root, component, install, whole};
use xpui::{
    Alignment, App, Font, FontRole, FontStyle, Icon, Image, List, ListRow, Modal, NavigationScreen,
    Scrim, Spacer, Text, ViewExt, hstack, vstack,
};
use xpui_chrome::Icon as Glyph;

/// The messages the pages' examples send. Nothing here sends them.
#[allow(dead_code)]
#[derive(Clone, Copy)]
enum Msg {
    Open,
    Chose(usize),
    Ask(usize),
    Answer(usize),
    Action(usize),
    Frontlight,
    Sleep,
    SleepAfter(usize),
}

const FONTS: [&str; 3] = ["Serif", "Sans", "Mono"];
const BOOKS: [&str; 3] = ["Middlemarch", "The Odyssey", "Walden"];
const ACTIONS: [&str; 3] = ["Open", "Mark as finished", "Remove from shelf"];
const DELAYS: [&str; 4] = ["1 min", "5 min", "15 min", "Never"];

/// The bookmark from the `Image` example, written with 1 as ink.
const RIBBON: [u16; 16] = [
    0b0011111111111100,
    0b0011111111111100,
    0b0011111111111100,
    0b0011111111111100,
    0b0011111111111100,
    0b0011111111111100,
    0b0011111111111100,
    0b0011111111111100,
    0b0011111111111100,
    0b0011111111111100,
    0b0011111111111100,
    0b0011111001111100,
    0b0011110000111100,
    0b0011100000011100,
    0b0011000000001100,
    0b0010000000000100,
];

static BOOKMARK: [u8; 32] = {
    let mut bytes = [0; 32];
    let mut row = 0;
    while row < 16 {
        let [high, low] = (!RIBBON[row]).to_be_bytes(); // bit 0 is ink
        bytes[row * 2] = high;
        bytes[row * 2 + 1] = low;
        row += 1;
    }
    bytes
};

/// The typeface picker, open on Sans.
fn typeface() -> NavigationScreen<Msg> {
    NavigationScreen::new(
        List::new().push(ListRow::new("Typeface").value(FONTS[1]).on_tap(Msg::Open)),
    )
    .overlay(
        Modal::picker("Typeface", FONTS)
            .selected(1)
            .on_select(Msg::Chose)
            .scrim(Scrim::Dim),
    )
}

fn library() -> List<Msg> {
    List::new().extend(
        BOOKS
            .iter()
            .enumerate()
            .map(|(index, title)| ListRow::new(*title).on_tap(Msg::Ask(index))),
    )
}

/// "Delete this book?", asked about the second book.
fn confirm() -> NavigationScreen<Msg> {
    NavigationScreen::new(library()).overlay(
        Modal::confirm("Delete this book?", ["Delete", "Keep"])
            .on_select(Msg::Answer)
            .scrim(Scrim::Dim),
    )
}

/// The action menu built with `Modal::new`.
fn actions() -> NavigationScreen<Msg> {
    NavigationScreen::new(library()).overlay(
        Modal::new("Middlemarch", ACTIONS)
            .on_select(Msg::Action)
            .scrim(Scrim::Dim),
    )
}

fn sleep_after(scrim: Scrim) -> NavigationScreen<Msg> {
    NavigationScreen::new(
        List::new()
            .push(
                ListRow::new("Frontlight")
                    .value("On")
                    .on_tap(Msg::Frontlight),
            )
            .push(
                ListRow::new("Sleep after")
                    .value(DELAYS[1])
                    .on_tap(Msg::Sleep),
            )
            .push(ListRow::new("Free heap").value("182 KB")),
    )
    .overlay(
        Modal::picker("Sleep after", DELAYS)
            .selected(1)
            .on_select(Msg::SleepAfter)
            .scrim(scrim),
    )
}

#[test]
fn the_pictures_on_the_dialogs_and_text_pages() {
    let mut failures = Failures::default();

    // docs/reference/dialogs.md
    let backend = install();
    App::new(Root("Typeface", typeface)).render();
    failures.note(whole(backend, "dialogs_picker"));

    let backend = install();
    App::new(Root("Library", confirm)).render();
    failures.note(whole(backend, "dialogs_confirm"));

    let backend = install();
    App::new(Root("Library", actions)).render();
    failures.note(whole(backend, "dialogs_new"));

    let backend = install();
    App::new(Root("Settings", || sleep_after(Scrim::Dim))).render();
    failures.note(whole(backend, "dialogs_scrim_dim"));

    let backend = install();
    App::new(Root("Settings", || sleep_after(Scrim::None))).render();
    failures.note(whole(backend, "dialogs_scrim_none"));

    // docs/reference/text.md
    failures.note(component::<Msg>("text_overview", || {
        hstack![12;
            Icon::new(Glyph::Book),
            vstack![4; Text::new("Middlemarch").bold(), Text::new("George Eliot").font(Font::ui_small())],
            Spacer::new(),
            Image::new(&BOOKMARK, 16, 16),
        ]
        .align(Alignment::Center)
        .boxed()
    }));
    failures.note(component::<Msg>("text_styles", || {
        vstack![8;
            Text::new("Regular"),
            Text::new("Bold").bold(),
            Text::new("Italic").italic(),
            Text::new("Bold italic").font(Font::ui().with_style(FontStyle::BoldItalic)),
        ]
        .boxed()
    }));
    failures.note(component::<Msg>("text_roles", || {
        vstack![8;
            Text::new("Interface text"),
            Text::new("A caption beneath it").font(Font::role(FontRole::UiSmall)),
            Text::new("The face chosen for reading").font(Font::role(FontRole::Reader)),
        ]
        .boxed()
    }));
    // docs/reference/images.md
    failures.note(component::<Msg>("text_image", || {
        hstack![8; Image::new(&BOOKMARK, 16, 16), Text::new("Bookmarked")]
            .align(Alignment::Center)
            .boxed()
    }));
    failures.note(component::<Msg>("text_icons", || {
        hstack![16;
            Icon::new(Glyph::Sun),
            Icon::new(Glyph::Sun).filled(true),
            Icon::new(Glyph::Book),
            Icon::new(Glyph::Battery).filled(true).size(24),
        ]
        .align(Alignment::Center)
        .boxed()
    }));

    failures.finish();
}
