//! The pictures on the framework's reference pages for screens and views, and
//! the walkthrough on its navigation page.
//!
//! Each view is the one its page's example builds, on the X3 like every other
//! reference picture. The walkthrough is driven with real key presses through
//! the backend, so its two pictures are the frames a user would see.
//!
//! ```bash
//! UPDATE_SNAPSHOTS=1 cargo test -p xpui-gallery --test reference_screens
//! open gallery/tests/screenshots/reference/
//! ```

mod support;

use support::{Failures, Root, component, install, whole};
use xpui::testing::Drive;
use xpui::{
    Alignment, App, Button, Divider, List, ListRow, Modifiers, NavigationScreen, OverlayPanel,
    Point, Rect, Renderer, Screen, Size, Stepper, Text, VStack, View, ViewExt, hstack, present,
    vstack,
};

/// The messages the pages' examples send. Nothing here sends them.
#[allow(dead_code)]
#[derive(Clone, Copy)]
enum Msg {
    Set(i32),
    Step(i32),
    Units(UnitMsg),
}

#[derive(Clone, Copy)]
enum UnitMsg {
    Cycle,
}

/// The screen from `Screen`'s minimal example, at 40.
fn brightness() -> NavigationScreen<Msg> {
    NavigationScreen::new(vstack![12;
        Text::new("Brightness"),
        Stepper::new(40).on_change(Msg::Set).on_step(Msg::Step),
    ])
}

fn library() -> NavigationScreen<Msg> {
    NavigationScreen::new(vstack![12;
        Text::new("Dune"),
        Text::new("Emma"),
        Text::new("Ivanhoe"),
    ])
}

/// The overlay from `Screen::is_overlay`'s example.
struct Sort;

impl Screen for Sort {
    type Message = Msg;

    fn body(&self) -> impl View<Msg> {
        OverlayPanel::new(vstack![0; Text::new("By title")])
    }

    fn update(&mut self, _message: Msg) {}

    fn is_overlay(&self) -> bool {
        true
    }

    fn title(&self) -> Option<&'static str> {
        Some("Sort")
    }
}

/// The gauge from `View`'s example.
struct Battery {
    percent: i32,
    measured: Size,
}

impl<M> View<M> for Battery {
    fn measure(&mut self, available: Size) {
        self.measured = Size::new(64.min(available.width), 28.min(available.height));
    }

    fn size(&self) -> Size {
        self.measured
    }

    fn render(&self, origin: Point) {
        let Size { width, height } = self.measured;
        Renderer::stroke_rect(Rect::new(origin.x, origin.y, width - 4, height));
        Renderer::fill_rect(
            Rect::new(origin.x + width - 4, origin.y + height / 3, 4, height / 3),
            true,
        );
        let fill = (width - 10) * self.percent / 100;
        Renderer::fill_rect(
            Rect::new(origin.x + 3, origin.y + 3, fill, height - 6),
            true,
        );
    }
}

/// The component from `ViewExt`'s example.
struct Units {
    binary: bool,
}

impl Units {
    fn view(&self, bytes: i32) -> impl View<UnitMsg> + use<> {
        let text = if self.binary {
            format!("{} KiB", bytes / 1024)
        } else {
            format!("{} kB", bytes / 1000)
        };
        Text::new(text).on_tap(UnitMsg::Cycle)
    }
}

/// The networks from the navigation walkthrough.
const NETWORKS: [(&str, &str); 3] = [
    ("Home", "Connected"),
    ("Office", "Saved"),
    ("Library", "Not saved"),
];

struct Networks;

impl Screen for Networks {
    type Message = usize;

    fn body(&self) -> impl View<usize> {
        NavigationScreen::new(
            List::new().extend(
                NETWORKS.iter().enumerate().map(|(index, (name, status))| {
                    ListRow::new(*name).value(*status).on_tap(index)
                }),
            ),
        )
    }

    fn update(&mut self, index: usize) {
        present(Network { index });
    }

    fn title(&self) -> Option<&'static str> {
        Some("Wi-Fi")
    }
}

struct Network {
    index: usize,
}

impl Screen for Network {
    type Message = ();

    fn body(&self) -> impl View<()> {
        let (_, status) = NETWORKS[self.index];
        NavigationScreen::new(
            List::new()
                .push(ListRow::new("Status").value(status))
                .push(ListRow::new("Security").value("WPA2")),
        )
    }

    fn update(&mut self, _message: ()) {}

    fn title(&self) -> Option<&'static str> {
        Some(NETWORKS[self.index].0)
    }
}

/// One frame with `key` pressed and released, then a paint.
fn press<H: Drive>(backend: &H, app: &mut App, millis: u32, key: Button) {
    backend.begin(millis);
    backend.inject_press(key);
    backend.inject_release(key);
    app.tick();
    app.render();
}

#[test]
fn the_pictures_on_the_screens_and_views_pages() {
    let mut failures = Failures::default();

    // docs/reference/screens.md
    let backend = install();
    App::new(Root("Brightness", brightness)).render();
    failures.note(whole(backend, "screens_screen"));

    let backend = install();
    let mut app = App::new(Root("Library", library));
    app.render();
    app.push(Sort);
    app.render();
    failures.note(whole(backend, "screens_overlay"));

    // docs/reference/views.md
    failures.note(component::<()>("views_battery", || {
        hstack![12;
            Battery { percent: 72, measured: Size::ZERO },
            Text::new("72%"),
        ]
        .align(Alignment::Center)
        .boxed()
    }));
    failures.note(component::<()>("views_boxed", || {
        let rows: Vec<Box<dyn View<()>>> = vec![
            Text::new("Wi-Fi").boxed(),
            Divider::new().boxed(),
            Text::new("Bluetooth").boxed(),
        ];
        VStack::new(8).extend(rows).boxed()
    }));
    failures.note(component::<Msg>("views_units", || {
        let units = Units { binary: false };
        vstack![8;
            Text::new("Free space"),
            units.view(182_000).map(Msg::Units),
        ]
        .boxed()
    }));

    // docs/reference/navigation.md, the walkthrough
    let backend = install();
    let mut app = App::new(Networks);
    app.render();
    press(backend, &mut app, 16, Button::Down);
    assert_eq!(app.depth(), 1);
    failures.note(whole(backend, "navigation_walkthrough_list"));

    press(backend, &mut app, 32, Button::Confirm);
    assert_eq!(app.depth(), 2, "Confirm on Office opened its screen");
    failures.note(whole(backend, "navigation_walkthrough_detail"));

    press(backend, &mut app, 48, Button::Back);
    assert_eq!(app.depth(), 1, "Back returned to the list");

    failures.finish();
}
