//! The pictures on the framework's reference pages for layout and modifiers.
//!
//! Each view is the one its page's example builds, on the X3 like every other
//! reference picture. The shared machinery is in `support/`.
//!
//! ```bash
//! UPDATE_SNAPSHOTS=1 cargo test -p xpui-gallery --test reference_layout
//! open gallery/tests/screenshots/reference/
//! ```
//!
//! **Open every image before committing it.** A golden here is shown to every
//! reader of the page, not only compared.

mod support;

use support::{Body, Failures, component, install, whole};
use xpui::{
    Alignment, App, Button, Divider, HStack, Icon, List, ListRow, Modifiers, NavigationScreen,
    Padding, Screen, ScrollView, Slider, Spacer, Text, VStack, View, ViewExt, hstack, vstack,
};
use xpui_chrome::Icon as Glyph;

/// The messages the pages' examples send. Nothing here sends them.
#[allow(dead_code)]
#[derive(Clone, Copy)]
enum Msg {
    Open(usize),
    Brightness(i32),
    Down,
    Up,
}

/// The `vstack!` example: a titled column with its footer pushed down.
fn storage() -> Box<dyn View<Msg>> {
    vstack![12;
        Text::new("Storage").bold(),
        hstack![8; Text::new("Books"), Spacer::new(), Text::new("128")],
        hstack![8; Text::new("Free space"), Spacer::new(), Text::new("182 KB")],
        Spacer::new(),
        Text::new("Last synced at 09:14"),
    ]
    .boxed()
}

/// The `ScrollView` example: more chapters than the panel holds.
struct Contents {
    chapters: Vec<String>,
}

impl Contents {
    fn new() -> Self {
        Contents {
            chapters: (1..=24).map(|n| format!("Chapter {n}")).collect(),
        }
    }
}

impl Screen for Contents {
    type Message = Msg;

    fn body(&self) -> impl View<Msg> {
        NavigationScreen::new(ScrollView::new(
            List::new().extend(
                self.chapters
                    .iter()
                    .enumerate()
                    .map(|(index, title)| ListRow::new(title.as_str()).on_tap(Msg::Open(index))),
            ),
        ))
    }

    fn update(&mut self, _message: Msg) {}

    fn title(&self) -> Option<&'static str> {
        Some("Contents")
    }
}

/// One row of the `Alignment` picture: an icon in a tall square beside a word.
fn aligned(alignment: Alignment, label: &'static str) -> HStack<Msg> {
    hstack![12; Modifiers::<Msg>::frame(Icon::new(Glyph::Sun), 56, 56), Text::new(label)]
        .align(alignment)
}

#[test]
fn the_pictures_on_the_layout_and_modifier_pages() {
    let mut failures = Failures::default();

    // docs/reference/layout.md
    let backend = install();
    App::new(Body(storage)).render();
    failures.note(whole(backend, "layout_vstack_footer"));

    failures.note(component::<Msg>("layout_vstack", || {
        VStack::new(8)
            .push(Text::new("Wi-Fi").bold())
            .push(Text::new("Connected to Home"))
            .push(Text::new("Signal strong"))
            .boxed()
    }));
    failures.note(component::<Msg>("layout_hstack", || {
        HStack::new(24)
            .push(Text::new("Books"))
            .push(Text::new("Fonts"))
            .push(Text::new("Notes"))
            .boxed()
    }));
    failures.note(component::<Msg>("layout_hstack_label", || {
        hstack![8; Text::new("Battery"), Text::new("72%").bold()].boxed()
    }));
    failures.note(component::<Msg>("layout_hstack_icon", || {
        hstack![12; Modifiers::<Msg>::frame(Icon::new(Glyph::Sun), 44, 44), Text::new("Frontlight")]
            .align(Alignment::Center)
            .boxed()
    }));
    failures.note(component::<Msg>("layout_hstack_spacer", || {
        hstack![8; Text::new("Battery"), Spacer::new(), Text::new("72%")].boxed()
    }));
    failures.note(component::<Msg>("layout_push_if", || {
        let syncing = true;
        let warning: Option<&str> = None;
        VStack::new(8)
            .push(Text::new("Library").bold())
            .push_if(syncing, Text::new("Syncing…"))
            .push_some(warning.map(Text::new))
            .push(Text::new("128 books"))
            .boxed()
    }));
    failures.note(component::<Msg>("layout_extend", || {
        let networks = ["Home", "Office", "Library"];
        VStack::new(8)
            .push(Text::new("Networks").bold())
            .extend(networks.into_iter().map(Text::new))
            .boxed()
    }));
    failures.note(component::<Msg>("layout_spacer", || {
        hstack![0; Spacer::new(), Text::new("Page 12 of 240"), Spacer::new()].boxed()
    }));
    failures.note(component::<Msg>("layout_padding_all", || {
        vstack![0;
            Divider::new(),
            Padding::all(Text::new("Sleep after 5 min"), 12),
            Divider::new(),
        ]
        .boxed()
    }));
    failures.note(component::<Msg>("layout_padding_symmetric", || {
        vstack![0;
            Divider::new(),
            Padding::symmetric(Text::new("Sleep after 5 min"), 40, 4),
            Divider::new(),
        ]
        .boxed()
    }));
    failures.note(component::<Msg>("layout_alignment", || {
        vstack![0;
            aligned(Alignment::Start, "Start"),
            Divider::new(),
            aligned(Alignment::Center, "Center"),
            Divider::new(),
            aligned(Alignment::End, "End"),
        ]
        .boxed()
    }));

    let backend = install();
    let mut app = App::new(Contents::new());
    app.render();
    failures.note(whole(backend, "layout_scroll_top"));
    for _ in 0..20 {
        backend.begin_frame(0);
        backend.press(Button::Down);
        app.tick();
        app.render();
    }
    failures.note(whole(backend, "layout_scroll_scrolled"));

    // docs/reference/modifiers.md
    failures.note(component::<Msg>("modifiers_overview", || {
        hstack![8;
            Modifiers::<Msg>::frame(Text::new("-"), 44, 44).on_tap(Msg::Down),
            Slider::new(30, 100).on_change(Msg::Brightness).flexible(),
            Modifiers::<Msg>::frame(Text::new("+"), 44, 44).on_tap(Msg::Up),
        ]
        .align(Alignment::Center)
        .boxed()
    }));
    failures.note(component::<Msg>("modifiers_frame", || {
        vstack![8;
            hstack![0; Text::new("-"), Text::new("+")],
            Divider::new(),
            hstack![0; Modifiers::<Msg>::frame(Text::new("-"), 44, 44), Modifiers::<Msg>::frame(Text::new("+"), 44, 44)],
        ]
        .boxed()
    }));
    failures.note(component::<Msg>("modifiers_flexible", || {
        vstack![12;
            hstack![8; Text::new("-"), Slider::new(40, 100), Text::new("+")],
            hstack![8; Text::new("-"), Slider::new(40, 100).flexible(), Text::new("+")],
        ]
        .boxed()
    }));

    failures.finish();
}
