//! Every example, driven the way a person would drive it.
//!
//! The gallery is the framework's own dogfood: if a screen here needs a
//! workaround, the framework has a gap. So these tests open each example, poke
//! it, and snapshot the result — which is also what stops the examples rotting
//! while nobody is looking at them.

use std::sync::{Mutex, MutexGuard};

use gallery::Menu;
use gallery::menu::Example;
use gallery::screens::{Controls, Dialogs, Lists, Scrolling, TextSizes};
use xpui::screen::{Driver, Runtime, Screen};
use xpui::{App, Button, testing};

/// `App` installs itself as the process-wide navigator, so one at a time.
static SERIAL: Mutex<()> = Mutex::new(());

fn serial() -> MutexGuard<'static, ()> {
    let guard = SERIAL
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    testing::install();
    testing::reset();
    guard
}

/// Renders one screen on its own and compares it against its golden.
fn snapshot<S: Screen + 'static>(name: &str, screen: S) {
    testing::install();
    testing::reset();
    let mut runtime = Runtime::new(screen);
    runtime.render();
    testing::assert_snapshot(name);
}

// -- the menu --------------------------------------------------------------

#[test]
fn the_menu_lists_every_example() {
    let _guard = serial();
    let mut runtime = Runtime::new(Menu::new());
    runtime.render();

    let rows = testing::drawn_list_rows();
    let listed: Vec<&str> = rows[0]
        .iter()
        .filter_map(|cells| cells[0].as_deref())
        .collect();

    let expected: Vec<&str> = Example::ALL.iter().map(|e| e.title()).collect();
    assert_eq!(
        listed, expected,
        "the menu shows exactly the examples the table declares"
    );
}

#[test]
fn every_example_carries_a_summary() {
    for example in Example::ALL {
        assert!(
            !example.summary().is_empty(),
            "{:?} has no summary, so its row would be a bare title",
            example
        );
    }
}

/// Confirming a row must actually push a screen. A menu that highlights rows
/// and opens nothing looks completely correct until you press the button.
#[test]
fn confirming_a_row_opens_that_example() {
    let _guard = serial();
    let mut app = App::new(Menu::new());
    assert_eq!(app.depth(), 1);

    testing::press(Button::Confirm);
    app.tick();

    assert_eq!(app.depth(), 2, "the first example opened");
}

/// And Back must come home again, or the gallery is a one-way trip.
#[test]
fn back_returns_to_the_menu() {
    let _guard = serial();
    let mut app = App::new(Menu::new());

    testing::press(Button::Confirm);
    app.tick();
    assert_eq!(app.depth(), 2);

    testing::press(Button::Back);
    app.tick();
    assert_eq!(app.depth(), 1, "Back popped the example");
    assert!(app.is_running(), "and the menu is still there");
}

/// Walking the whole menu and opening each row in turn — the thing a person
/// does first, and the one path that touches every example.
#[test]
fn every_example_opens_and_closes() {
    let _guard = serial();

    for (index, example) in Example::ALL.into_iter().enumerate() {
        let mut app = App::new(Menu::new());

        // Focus starts on the first row; step down to this one.
        for _ in 0..index {
            testing::press(Button::Down);
            app.tick();
        }

        testing::press(Button::Confirm);
        app.tick();
        assert_eq!(app.depth(), 2, "{example:?} did not open from row {index}");

        // It must survive a frame and a paint.
        app.render();
        app.tick();

        testing::press(Button::Back);
        app.tick();
        assert_eq!(app.depth(), 1, "{example:?} did not close");
    }
}

// -- the screens themselves ------------------------------------------------

#[test]
fn controls_screen() {
    snapshot("controls", Controls::new());
}

#[test]
fn lists_screen() {
    snapshot("lists", Lists::new());
}

#[test]
fn dialogs_screen_closed() {
    snapshot("dialogs_closed", Dialogs::new());
}

#[test]
fn scrolling_screen() {
    snapshot("scrolling", Scrolling::new());
}

#[test]
fn text_screen() {
    snapshot("text", TextSizes::new());
}

/// Left and Right nudge whatever holds focus, so one pair of keys drives every
/// adjustable control on a screen. The stepper is first, so it gets them.
#[test]
fn arrows_adjust_the_focused_control() {
    let _guard = serial();
    let mut runtime = Runtime::new(Controls::new());
    runtime.render();

    testing::press(Button::Right);
    runtime.loop_();
    testing::reset();
    runtime.render();

    let after = testing::drawn_sliders();
    assert!(
        !after.is_empty(),
        "the controls screen still draws its sliders"
    );
    assert_eq!(after[0].1, 61, "Right nudged the focused stepper up by one");
}

/// A dialog is opened by the screen putting it in `body()`, and it captures
/// input while it is there — the list behind must stop being highlighted.
#[test]
fn opening_the_dialog_captures_input() {
    let _guard = serial();
    let mut runtime = Runtime::new(Dialogs::new());
    runtime.render();
    assert!(testing::drawn_popups().is_empty(), "it starts closed");

    testing::press(Button::Confirm);
    runtime.loop_();
    testing::reset();
    runtime.render();

    assert_eq!(testing::drawn_popups().len(), 1, "the dialog opened");
    let lists = testing::drawn_lists();
    assert_eq!(
        lists[0].1, -1,
        "and the list behind it stopped highlighting a row"
    );
}

/// Choosing an option closes the dialog and keeps the choice.
#[test]
fn choosing_an_option_closes_the_dialog() {
    let _guard = serial();
    let mut screen = Dialogs::new();
    assert_eq!(screen.chosen(), "Serif");

    screen.update(gallery::screens::DialogMsg::Open);
    assert!(screen.is_open());

    screen.update(gallery::screens::DialogMsg::Chose(2));
    assert!(!screen.is_open(), "choosing closed it");
    assert_eq!(screen.chosen(), "Mono", "and kept what was chosen");
}

/// A toggle hands over the state it is moving to, so the screen never writes
/// `!self.something` — the bug that makes a toggle flip twice per press.
#[test]
fn a_toggle_moves_to_the_state_it_reports() {
    let mut screen = Controls::new();
    screen.update(gallery::screens::ControlsMsg::Frontlight(false));
    screen.update(gallery::screens::ControlsMsg::Frontlight(false));

    // Applying the same message twice must be idempotent. A screen that
    // flipped instead would be back where it started.
    testing::install();
    testing::reset();
    let mut runtime = Runtime::new(screen);
    runtime.render();

    let rows = testing::drawn_list_rows();
    let toggle_value = rows
        .iter()
        .flatten()
        .find(|cells| cells[0].as_deref() == Some("Frontlight"))
        .and_then(|cells| cells[2].clone());

    assert_eq!(
        toggle_value.as_deref(),
        Some("Off"),
        "two identical messages leave it where they said, not flipped back"
    );
}

/// A stepper is one focus stop, not three, so Up and Down move between
/// settings rather than through its glyphs.
#[test]
fn a_stepper_is_a_single_focus_stop() {
    let _guard = serial();
    let mut runtime = Runtime::new(Controls::new());
    runtime.render();
    let first = runtime.focused_index();

    testing::press(Button::Down);
    runtime.loop_();

    assert_eq!(
        runtime.focused_index(),
        first + 1,
        "one press moved to the next control, not into the stepper's glyphs"
    );
}
