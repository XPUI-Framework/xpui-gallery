//! The tutorial's screen, driven the way the tutorial says it behaves.
//!
//! If a claim in the framework's `docs/tutorial.md` is not true of this crate, one
//! of these fails. That is the whole arrangement: the prose is proven by its
//! snippets compiling, and the *result* is proven here.
//!
//! Behaviour only. The pixel tests live in `screenshots.rs` because they
//! install a *different* host, and `testing::install` is idempotent — it will
//! not reclaim the global from another backend. Two files means two processes,
//! which is the only reliable way to keep the two hosts apart.

use std::sync::{Mutex, MutexGuard};

use tutorial::{Message, SleepTimer};
use xpui::screen::{Driver, Runtime, Screen};
use xpui::testing;
use xpui::{App, Button};

/// The installed host is process-wide, so these take turns.
static SERIAL: Mutex<()> = Mutex::new(());

fn serial() -> MutexGuard<'static, ()> {
    let guard = SERIAL
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    testing::install();
    testing::reset();
    guard
}

// -- what the tutorial claims ----------------------------------------------

#[test]
fn it_starts_at_fifteen_minutes() {
    let screen = SleepTimer::new();
    assert_eq!(screen.minutes(), 15);
    assert!(screen.sleeps_on_close());
    assert!(!screen.is_picking());
}

/// The stepper reports an absolute value when its track is used, and a nudge
/// when its end glyphs are. Both clamp.
#[test]
fn the_stepper_sets_and_nudges_within_its_range() {
    let mut screen = SleepTimer::new();

    screen.update(Message::SetMinutes(45));
    assert_eq!(screen.minutes(), 45);

    screen.update(Message::StepMinutes(1));
    assert_eq!(screen.minutes(), 46);

    screen.update(Message::SetMinutes(9999));
    assert_eq!(screen.minutes(), 120, "clamped to the top of the range");

    screen.update(Message::SetMinutes(-5));
    assert_eq!(screen.minutes(), 1, "and to the bottom");
}

/// The toggle hands over the state it is moving *to*. Applying the same
/// message twice must therefore be idempotent — a screen that flipped instead
/// would be back where it started.
#[test]
fn the_toggle_moves_to_the_state_it_reports() {
    let mut screen = SleepTimer::new();
    assert!(screen.sleeps_on_close());

    screen.update(Message::SetSleepOnClose(false));
    screen.update(Message::SetSleepOnClose(false));
    assert!(!screen.sleeps_on_close(), "twice is the same as once");
}

#[test]
fn choosing_a_preset_sets_the_minutes_and_closes_the_dialog() {
    let mut screen = SleepTimer::new();

    screen.update(Message::OpenPresets);
    assert!(screen.is_picking());

    screen.update(Message::ChoosePreset(3));
    assert_eq!(screen.minutes(), 60, "the fourth preset is an hour");
    assert!(!screen.is_picking(), "choosing closed it");
}

#[test]
fn tapping_the_scrim_dismisses_the_dialog_without_changing_anything() {
    let mut screen = SleepTimer::new();
    screen.update(Message::OpenPresets);

    let message = screen.on_background_tap(xpui::Point::new(10, 10));
    assert_eq!(message, Some(Message::DismissPresets));

    screen.update(Message::DismissPresets);
    assert!(!screen.is_picking());
    assert_eq!(screen.minutes(), 15, "dismissing changed nothing");
}

/// A background tap must do nothing when there is no dialog, or every tap on
/// empty space would be swallowed.
#[test]
fn a_background_tap_does_nothing_when_no_dialog_is_open() {
    let screen = SleepTimer::new();
    assert_eq!(screen.on_background_tap(xpui::Point::new(10, 10)), None);
}

// -- the runtime drives it -------------------------------------------------

/// Confirm on the focused control fires the same message a tap would. Focus
/// opens on the first control, which is the stepper.
#[test]
fn the_arrows_nudge_the_focused_stepper() {
    let _guard = serial();
    let mut runtime = Runtime::new(SleepTimer::new());
    runtime.render();

    testing::press(Button::Right);
    runtime.loop_();

    testing::reset();
    runtime.render();
    let sliders = testing::drawn_sliders();
    assert_eq!(sliders[0].1, 16, "Right nudged the stepper from 15 to 16");
}

/// While the dialog is up, the list behind it must stop being highlighted —
/// that is what "captures input" means, seen from the outside.
#[test]
fn the_dialog_captures_input_from_the_list_behind_it() {
    let _guard = serial();
    let mut screen = SleepTimer::new();
    screen.update(Message::OpenPresets);

    let mut runtime = Runtime::new(screen);
    runtime.render();

    assert_eq!(testing::drawn_popups().len(), 1, "the dialog is up");
    let lists = testing::drawn_lists();
    assert!(
        lists.iter().all(|(_, selected)| *selected == -1),
        "nothing behind it is highlighted: {lists:?}"
    );
}

/// Back finishes the screen, which is what makes it poppable from a stack.
#[test]
fn back_finishes_the_screen() {
    let _guard = serial();
    let mut app = App::new(SleepTimer::new());
    assert_eq!(app.depth(), 1);

    testing::press(Button::Back);
    app.tick();

    assert_eq!(app.depth(), 0, "Back popped it");
    assert!(!app.is_running());
}
