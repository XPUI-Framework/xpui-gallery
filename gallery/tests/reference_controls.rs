//! The pictures on the framework's reference pages for controls and indicators.
//!
//! On the X3 like every other reference picture, and each view is the one its
//! page's example builds. The exception is the open value mode, which the X3
//! never shows: it has a Left/Right pair, so Confirm opens nothing there. Those
//! two pictures are taken on a Badger 2040, which has no pair.
//!
//! ```bash
//! UPDATE_SNAPSHOTS=1 cargo test -p xpui-gallery --test reference_controls
//! open gallery/tests/screenshots/reference/
//! ```

mod support;

use gallery::wire;
use support::{Body, Failures, Root, check, component, crop, install, whole};
use xpui::{
    App, Button, IconToggle, List, ListRow, NavigationScreen, ProgressBar, Rect, Screen, Slider,
    Stepper, Text, Theme, ThemeMetric, Toggle, VStack, View, ViewExt, hstack, vstack,
};
use xpui_boards_pimoroni::BADGER_2040;
use xpui_chrome::Icon;
use xpui_eg::{Backend, Palette};
use xpui_screenshot::Framebuffer;

/// The messages the pages' examples send. Nothing here sends them.
#[allow(dead_code)]
#[derive(Clone, Copy)]
enum Msg {
    Brightness(i32),
    BrightnessStep(i32),
    Warmth(i32),
    Frontlight(bool),
    Hyphenation(bool),
    Justify(bool),
    Light(bool),
    Night(bool),
    Elsewhere,
}

fn warmth() -> Box<dyn View<Msg>> {
    Slider::new(25, 100)
        .on_change(Msg::Warmth)
        .title("Warmth")
        .readout("%")
        .boxed()
}

fn plain_track() -> Box<dyn View<Msg>> {
    Slider::new(40, 100).on_change(Msg::Warmth).boxed()
}

fn brightness() -> Box<dyn View<Msg>> {
    Stepper::new(60)
        .on_change(Msg::Brightness)
        .on_step(Msg::BrightnessStep)
        .title("Brightness")
        .readout("%")
        .boxed()
}

fn frontlight() -> Box<dyn View<Msg>> {
    Toggle::new("Frontlight", true, "On", "Off")
        .on_change(Msg::Frontlight)
        .boxed()
}

fn display() -> NavigationScreen<Msg> {
    NavigationScreen::new(vstack![24;
        Stepper::new(60)
            .on_change(Msg::Brightness)
            .on_step(Msg::BrightnessStep)
            .title("Brightness")
            .readout("%"),
        Slider::new(25, 100).on_change(Msg::Warmth).title("Warmth").readout("%"),
        Toggle::new("Frontlight", true, "On", "Off").on_change(Msg::Frontlight),
    ])
}

fn reading() -> NavigationScreen<Msg> {
    NavigationScreen::new(vstack![16;
        Text::new("Page 120 of 340"),
        ProgressBar::new(120, 340),
    ])
}

/// A screen whose content sits above one more row that takes focus.
///
/// Up wraps from the first focus stop to the last, which is that row, so the
/// content above is painted idle.
struct Elsewhere(fn() -> Box<dyn View<Msg>>);

impl Screen for Elsewhere {
    type Message = Msg;

    fn body(&self) -> impl View<Msg> {
        // Wider than the crop's margin, so the row stays out of the picture.
        let gap = Theme::metric(ThemeMetric::SpacingSmall) * 3;
        NavigationScreen::new(
            VStack::new(gap)
                .push((self.0)())
                .push(List::new().push(ListRow::new("Elsewhere").on_tap(Msg::Elsewhere))),
        )
    }

    fn update(&mut self, _message: Msg) {}

    fn title(&self) -> Option<&'static str> {
        Some("Settings")
    }
}

/// One frame carrying `key`, then a repaint.
fn press(backend: &'static Backend<Framebuffer>, app: &mut App, key: Button) {
    backend.begin_frame(0);
    backend.press(key);
    app.tick();
    app.render();
}

/// `content` with the keys elsewhere, cropped as `support::component` crops.
fn idle(name: &str, content: fn() -> Box<dyn View<Msg>>) -> Result<(), String> {
    let backend = install();
    let mut app = App::new(Elsewhere(content));
    app.render();
    press(backend, &mut app, Button::Up);

    let band = Theme::content_area();
    let mut view = content();
    view.measure(band.size);
    let size = view.size();
    let pad = Theme::metric(ThemeMetric::SpacingSmall);
    let rect = Rect::new(
        band.x() - pad,
        band.y() - pad,
        size.width + 2 * pad,
        size.height + 2 * pad,
    );
    let frame = backend.with_display(|panel| crop(panel, rect));
    check(name, &frame)
}

/// `support::install` on a Badger 2040: a board with no Left/Right pair.
fn install_badger() -> &'static Backend<Framebuffer> {
    let backend = wire(
        Framebuffer::new(BADGER_2040.width, BADGER_2040.height),
        BADGER_2040,
        Palette::INK_IS_ON,
    )
    .leaked();
    // Safety: this file holds one test, so nothing else installs a host or
    // renders while this one is in use.
    unsafe { xpui::host::install(backend) };
    backend
}

#[test]
fn the_pictures_on_the_controls_and_indicators_pages() {
    let mut failures = Failures::default();

    // docs/reference/controls.md
    let backend = install();
    App::new(Root("Display", display)).render();
    failures.note(whole(backend, "controls_overview"));

    failures.note(idle("controls_slider", warmth));
    failures.note(component::<Msg>("controls_slider_focused", warmth));
    failures.note(idle("controls_slider_plain", plain_track));

    failures.note(idle("controls_stepper", brightness));
    failures.note(component::<Msg>("controls_stepper_focused", brightness));

    // Focus starts on the stepper, so Confirm opens it; Up raises the copy
    // the framework holds, three times, while the screen still reads 60.
    let backend = install_badger();
    let mut app = App::new(Body(brightness));
    app.render();
    failures.note(whole(backend, "controls_stepper_badger"));
    press(backend, &mut app, Button::Confirm);
    for _ in 0..3 {
        press(backend, &mut app, Button::Up);
    }
    failures.note(whole(backend, "controls_stepper_open"));

    // docs/reference/toggles.md
    failures.note(idle("controls_toggle", frontlight));
    failures.note(component::<Msg>("controls_toggle_focused", frontlight));
    failures.note(idle("controls_toggle_list", || {
        vstack![24;
            Toggle::new("Frontlight", true, "On", "Off").on_change(Msg::Frontlight),
            List::new()
                .push(ListRow::toggle("Hyphenation", false, "On", "Off").on_tap(Msg::Hyphenation(true)))
                .push(ListRow::toggle("Justify", true, "On", "Off").on_tap(Msg::Justify(false))),
        ]
        .boxed()
    }));

    failures.note(component::<Msg>("controls_icon_toggle", || {
        hstack![8;
            IconToggle::new(Icon::Sun, true).on_change(Msg::Light),
            IconToggle::new(Icon::Sun, false).on_change(Msg::Night),
        ]
        .boxed()
    }));

    // docs/reference/indicators.md
    let backend = install();
    App::new(Root("Chapter 3", reading)).render();
    failures.note(whole(backend, "indicators_overview"));
    failures.note(component::<Msg>("indicators_progress_bar", || {
        vstack![16;
            ProgressBar::new(120, 340),
            ProgressBar::percent(72),
            ProgressBar::percent(72).height(16),
        ]
        .boxed()
    }));

    failures.finish();
}
