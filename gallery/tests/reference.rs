//! The pictures on the framework's reference pages for lists and navigation.
//!
//! Every `reference*.rs` file renders what one or two pages of
//! `xpui/docs/reference/` show, on the X3 alone so every page has one scale,
//! cropped to the component where a whole panel would bury it. Each view is the
//! one its page's example builds, so a picture and the code beside it cannot
//! drift apart unnoticed. The shared machinery is in `support/`.
//!
//! ```bash
//! UPDATE_SNAPSHOTS=1 cargo test -p xpui-gallery --test reference
//! open gallery/tests/screenshots/reference/
//! ```
//!
//! **Open every image before committing it.** A golden here is shown to every
//! reader of the page, not only compared.

mod support;

use support::{Body, Failures, Root, component, hint_bar, install, whole};
use xpui::{
    App, Divider, Hint, List, ListRow, NavigationScreen, OverlayPanel, Screen, Scrim, Section,
    Slider, Text, View, ViewExt, vstack,
};

/// The messages the pages' examples send. Nothing here sends them.
#[allow(dead_code)]
#[derive(Clone, Copy)]
enum Msg {
    Frontlight(bool),
    Sleep,
    Network,
    Hyphenation(bool),
    Brightness(i32),
    Dismiss,
}

/// The frontlight drop-down from the `OverlayPanel` example.
struct Frontlight;

impl Screen for Frontlight {
    type Message = Msg;

    fn body(&self) -> impl View<Msg> {
        OverlayPanel::new(
            Slider::new(40, 100)
                .on_change(Msg::Brightness)
                .title("Brightness")
                .readout("%"),
        )
        .scrim(Scrim::Dim)
        .on_scrim_tap(Msg::Dismiss)
    }

    fn update(&mut self, _message: Msg) {}

    fn is_overlay(&self) -> bool {
        true
    }

    fn title(&self) -> Option<&'static str> {
        Some("Frontlight")
    }
}

fn settings() -> Box<dyn View<Msg>> {
    List::new()
        .push(ListRow::toggle("Frontlight", true, "On", "Off").on_tap(Msg::Frontlight(false)))
        .push(
            ListRow::new("Sleep after")
                .value("5 min")
                .on_tap(Msg::Sleep),
        )
        .push(ListRow::new("Free heap").value("182 KB"))
        .boxed()
}

fn storage() -> NavigationScreen<Msg> {
    NavigationScreen::new(vstack![20;
        Text::new("Free space"),
        Text::new("182 KB").bold(),
    ])
    .title("Storage")
    .hints(Hint::Standard, Hint::text("Save"), Hint::None, Hint::None)
}

#[test]
fn the_pictures_on_the_reference_pages() {
    let mut failures = Failures::default();

    // docs/reference/lists.md
    failures.note(component::<Msg>("lists_list", settings));
    failures.note(component::<Msg>("lists_rows", || {
        List::new()
            .push(
                ListRow::new("Wi-Fi")
                    .subtitle("Home network")
                    .on_tap(Msg::Network),
            )
            .push(
                ListRow::new("Sleep after")
                    .value("5 min")
                    .on_tap(Msg::Sleep),
            )
            .push(ListRow::toggle("Hyphenation", true, "On", "Off").on_tap(Msg::Hyphenation(false)))
            .push(ListRow::new("Firmware").value("1.4.2"))
            .boxed()
    }));
    failures.note(component::<Msg>("lists_section", || {
        vstack![16;
            Section::new("Display", List::new().push(ListRow::new("Frontlight").value("On").on_tap(Msg::Frontlight(false)))),
            Section::new("Power", List::new().push(ListRow::new("Sleep after").value("5 min").on_tap(Msg::Sleep))),
        ]
        .boxed()
    }));
    failures.note(component::<Msg>("lists_divider", || {
        vstack![8; Text::new("Wi-Fi"), Divider::new(), Text::new("Bluetooth")].boxed()
    }));

    // docs/reference/navigation.md
    let backend = install();
    App::new(Root("Storage", storage)).render();
    failures.note(whole(backend, "navigation_screen"));
    failures.note(hint_bar(backend, "navigation_hints"));

    let backend = install();
    let mut app = App::new(Body(settings));
    app.render();
    app.push(Frontlight);
    app.render();
    failures.note(whole(backend, "navigation_overlay_panel"));

    failures.finish();
}
