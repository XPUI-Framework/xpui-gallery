//! The Developers screen: unit, integration and UI.
//!
//! This screen exists to put the list, the dialog, the slider, the stepper and
//! focus navigation on one panel, because the interesting failures are between
//! them rather than inside any one — a dialog capturing input from the list
//! behind it, a slider and a stepper disagreeing about the value they share.

use gallery::heap::Heap;
use gallery::wire;
use gallery::{DevelopersScreen, Menu, Units};
use xpui::testing::Ui;
use xpui::{Button, Screen};
use xpui_boards_core::Board;
use xpui_boards_pimoroni as pimoroni;
use xpui_boards_xteink as xteink;
use xpui_eg::{Backend, Palette};
use xpui_screenshot::Framebuffer;

fn on(board: Board) -> &'static Backend<Framebuffer> {
    wire(
        Framebuffer::new(board.width, board.height),
        board,
        Palette::INK_IS_ON,
    )
    .leaked()
}

// -- unit ------------------------------------------------------------------

#[test]
fn kilobytes_round_rather_than_truncate() {
    // 700 bytes is most of a kilobyte. Truncating would show it as 0 KB, which
    // reads as "nothing here" rather than "not quite one".
    assert_eq!(Units::Kilobytes.format(700), "1 KB");
    assert_eq!(Units::Kilobytes.format(1_048_576), "1,024 KB");
    // A negative figure rounds away from zero too, rather than reading as none.
    assert_eq!(Units::Kilobytes.format(-700), "-1 KB");
    assert_eq!(Units::Kilobytes.format(-300), "0 KB");
}

#[test]
fn long_figures_are_grouped() {
    assert_eq!(Units::Bytes.format(1_048_576), "1,048,576 B");
    assert_eq!(Units::Bytes.format(999), "999 B");
    assert_eq!(Units::Bytes.format(-1_234), "-1,234 B");
}

#[test]
fn the_two_scales_cycle() {
    assert_eq!(Units::Bytes.next(), Units::Kilobytes);
    assert_eq!(Units::Kilobytes.next(), Units::Bytes);
}

/// The figures have to move, or a repaint test cannot tell a redraw from a
/// no-op.
#[test]
fn successive_heap_readings_differ() {
    let heap = Heap::default();
    let first = heap.reading();
    let second = heap.reading();

    assert_ne!(first, second, "a constant heap makes repaint tests vacuous");
    assert!(first.free < first.total, "something must be in use");
    assert!(
        first.largest_block <= first.free,
        "the biggest block that fits cannot exceed what is free"
    );
    assert!(first.min_free <= first.free, "a low-water mark is a floor");
}

// -- integration -----------------------------------------------------------

/// The slider and the stepper share one value on purpose: that is how a drag
/// and a button press get compared against each other.
#[test]
fn the_slider_and_the_stepper_move_together() {
    let mut screen = DevelopersScreen::new();
    let before = screen.level();

    screen.update(gallery::developers::Msg::StepBrightness(1));
    assert_eq!(screen.level(), before + 1);

    screen.update(gallery::developers::Msg::Brightness(80));
    assert_eq!(screen.level(), 80, "the slider sets an absolute value");

    screen.update(gallery::developers::Msg::StepBrightness(1));
    assert_eq!(screen.level(), 81, "the stepper nudges what the slider set");
}

#[test]
fn the_level_is_clamped_at_both_ends() {
    let mut screen = DevelopersScreen::new();

    screen.update(gallery::developers::Msg::Brightness(500));
    assert_eq!(screen.level(), 100);

    screen.update(gallery::developers::Msg::StepBrightness(1));
    assert_eq!(screen.level(), 100, "a nudge past the top stays at the top");

    screen.update(gallery::developers::Msg::Brightness(-5));
    assert_eq!(screen.level(), 0);
}

#[test]
fn choosing_a_scale_closes_the_picker() {
    let mut screen = DevelopersScreen::new();

    screen.update(gallery::developers::Msg::PickUnits);
    assert!(screen.is_picking());

    screen.update(gallery::developers::Msg::ChoseUnits(0));
    assert!(!screen.is_picking());
    assert_eq!(screen.units(), Units::Bytes);
}

/// Back closes the dialog rather than the screen. The runtime does not assume
/// this, so a screen showing one has to say so.
#[test]
fn back_closes_the_picker_before_the_screen() {
    let screen = DevelopersScreen::new();
    assert!(
        screen.on_key(Button::Back).is_none(),
        "with no dialog up, Back belongs to the runtime"
    );

    let mut screen = DevelopersScreen::new();
    screen.update(gallery::developers::Msg::PickUnits);
    assert!(
        screen.on_key(Button::Back).is_some(),
        "with the picker open, Back must close it"
    );
}

// -- ui --------------------------------------------------------------------

#[test]
fn it_opens_from_the_menu() {
    let mut ui = Ui::new(Menu::new(), on(xteink::X4));

    ui.tap_text("Developers");

    assert_eq!(ui.depth(), 2);
    assert!(
        ui.visible_text().iter().any(|line| line == "Memory"),
        "the memory section should be the first thing on it. Visible: {:#?}",
        ui.visible_text()
    );
}

/// Tapping a memory row opens the scale picker, and choosing rewrites every
/// figure — the reason the rows share one scale.
#[test]
fn choosing_a_scale_rewrites_every_figure() {
    let mut ui = Ui::new(DevelopersScreen::new(), on(xteink::X4));

    let before = ui.visible_text();
    assert!(
        before.iter().any(|line| line.ends_with(" KB")),
        "kilobytes are the default. Visible: {before:#?}"
    );

    ui.tap_text("Total");
    assert!(
        ui.visible_text().iter().any(|line| line == "Bytes"),
        "tapping a figure must open the picker. Visible: {:#?}",
        ui.visible_text()
    );

    ui.tap_text("Bytes");
    assert!(
        ui.visible_text().iter().any(|line| line.ends_with(" B")),
        "choosing bytes must rewrite the figures. Visible: {:#?}",
        ui.visible_text()
    );
}

/// Every control has to be reachable on the smallest panel, which is the one
/// where nothing fits.
#[test]
fn every_section_is_reachable_on_the_smallest_panel() {
    let mut ui = Ui::new(DevelopersScreen::new(), on(pimoroni::BADGER_2040));

    let mut seen = ui.visible_text();
    for _ in 0..60 {
        ui.press(Button::Down);
        for line in ui.visible_text() {
            if !seen.contains(&line) {
                seen.push(line);
            }
        }
    }

    for wanted in ["Memory", "Widget bench", "Scrolling", "Row 14"] {
        assert!(
            seen.iter().any(|line| line == wanted),
            "{wanted:?} is unreachable on a Badger 2040. Seen: {seen:#?}"
        );
    }
}
