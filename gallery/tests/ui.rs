//! The gallery, driven the way a person drives it.
//!
//! These go through `App` and a real backend, so they exercise the layer where
//! the interesting failures live. A test that drove the runtime directly would
//! have reported the arrow keys working for as long as they were broken.

use xpui::testing::Ui;
use xpui::{Button, Point};
use xpui_eg::{Backend, Board, Framebuffer, Palette};

/// Scrolls the menu until `row` is on screen, then opens it.
fn open(ui: &mut Ui<Backend<Framebuffer>>, row: &str) {
    for _ in 0..12 {
        if !ui.rects_of_text(row).is_empty() {
            break;
        }
        ui.press(Button::Down);
    }
    assert!(
        !ui.rects_of_text(row).is_empty(),
        "{row:?} is unreachable from the menu. Visible: {:#?}",
        ui.visible_text()
    );
    ui.tap_text(row);
}

fn on(board: Board) -> &'static Backend<Framebuffer> {
    Backend::leak_for_board(
        Framebuffer::new(board.width, board.height),
        board,
        Palette::INK_IS_ON,
    )
}

#[test]
fn the_menu_lists_every_example() {
    let mut ui = Ui::new(gallery::Menu::new(), on(Board::X4));
    let text = ui.visible_text();

    for title in ["Controls", "Lists", "Dialogs", "Scrolling", "Text"] {
        assert!(
            text.iter().any(|line| line == title),
            "the menu does not show {title:?}. Visible: {text:#?}"
        );
    }
    let _ = &mut ui;
}

#[test]
fn tapping_a_row_opens_that_example() {
    let mut ui = Ui::new(gallery::Menu::new(), on(Board::X4));
    assert_eq!(ui.depth(), 1);

    ui.tap_text("Lists");

    assert_eq!(ui.depth(), 2, "tapping a row must open its example");
    assert!(
        ui.visible_text().iter().any(|line| line == "Lists"),
        "the opened screen should be titled Lists. Visible: {:#?}",
        ui.visible_text()
    );
}

#[test]
fn back_returns_to_the_menu() {
    let mut ui = Ui::new(gallery::Menu::new(), on(Board::X4));
    ui.tap_text("Controls");
    assert_eq!(ui.depth(), 2);

    ui.press(Button::Back);

    assert_eq!(ui.depth(), 1, "Back must return to the menu");
}

#[test]
fn moving_the_focus_redraws() {
    let mut ui = Ui::new(gallery::Menu::new(), on(Board::X4));

    ui.press(Button::Down);

    assert!(
        ui.changed(),
        "moving the focus must repaint — otherwise the selection moves \
         internally and the panel never shows it, which is what made the \
         arrow keys look dead"
    );
}

#[test]
#[should_panic(expected = "nothing on screen shows")]
fn tapping_a_label_that_is_not_there_says_so() {
    let mut ui = Ui::new(gallery::Menu::new(), on(Board::X4));
    ui.tap_text("Setings");
}

/// A tap outside anything interactive must do nothing at all, rather than
/// resolving to the nearest control.
#[test]
fn tapping_empty_space_opens_nothing() {
    let mut ui = Ui::new(gallery::Menu::new(), on(Board::X4));

    ui.tap_at(Point::new(2, 2));

    assert_eq!(ui.depth(), 1, "a tap on the header must not open a screen");
}

/// Each row must open *its own* example, not the one above or below.
///
/// This is what catches a tap point that drifts by a row height: every row
/// would still open something, and only the wrong thing gives it away.
#[test]
fn every_row_opens_the_example_it_names() {
    for (row, title) in [
        ("Controls", "Controls"),
        ("Lists", "Lists"),
        ("Dialogs", "Dialogs"),
        ("Scrolling", "Scrolling"),
    ] {
        let mut ui = Ui::new(gallery::Menu::new(), on(Board::X4));
        ui.tap_text(row);

        assert_eq!(ui.depth(), 2, "tapping {row:?} opened nothing");
        assert!(
            ui.visible_text().iter().any(|line| line == title),
            "tapping {row:?} opened the wrong screen. Visible: {:#?}",
            ui.visible_text()
        );
    }
}

// -- the smallest panel ----------------------------------------------------

/// A Badger 2040's content band is 90 pixels. Every example must still be
/// usable there: reachable from the menu, and scrollable to its last control.
///
/// A screen that is not inside a `ScrollView` fails here and only here — on a
/// large panel everything fits, so nothing gives it away.
#[test]
fn every_example_opens_on_the_smallest_panel() {
    for row in ["Controls", "Lists", "Dialogs", "Scrolling", "Text"] {
        let mut ui = Ui::new(gallery::Menu::new(), on(Board::BADGER_2040));
        open(&mut ui, row);
        assert_eq!(ui.depth(), 2, "{row:?} did not open on a Badger 2040");
        assert!(
            ui.visible_text().iter().any(|line| line == row),
            "tapping {row:?} on a Badger 2040 opened the wrong screen. \
             Visible: {:#?}",
            ui.visible_text()
        );
    }
}

/// Content below the fold must be reachable on the smallest panel.
///
/// The previous version of this test pressed Down and asked whether anything
/// repainted. That can never fail: focus wraps, so on any screen with two
/// focusable things every press moves focus and repaints — and it was green
/// while four of the six screens had no `ScrollView` at all.
///
/// This asks the question that matters instead: after scrolling, is there text
/// on the panel that was not there before?
#[test]
fn scrolling_reveals_content_that_was_below_the_fold() {
    for row in ["Controls", "Lists", "Scrolling", "Text"] {
        let mut ui = Ui::new(gallery::Menu::new(), on(Board::BADGER_2040));
        open(&mut ui, row);

        let before = ui.visible_text();

        // The union of everything seen along the way, not the state at the end:
        // focus wraps, so after an even number of presses the panel is back
        // where it started and a before/after comparison finds nothing.
        let mut seen = before.clone();
        for _ in 0..24 {
            ui.press(Button::Down);
            for line in ui.visible_text() {
                if !seen.contains(&line) {
                    seen.push(line);
                }
            }
        }

        let revealed: Vec<_> = seen.iter().filter(|line| !before.contains(line)).collect();
        assert!(
            !revealed.is_empty(),
            "{row:?} never showed anything new across 24 presses of Down on a \
             Badger 2040. Its content band is 90 pixels, so either everything \
             fits — it does not — or it is not inside a ScrollView and the rest \
             is unreachable.\nVisible: {before:#?}"
        );
    }
}

// -- the keys a device actually has ----------------------------------------

/// Every key on a reader's body has to move the selection.
///
/// These devices have four keys along the bottom and two on the sides, and a
/// list is the screen you spend most of your time on. A key that does nothing
/// there is a key the hardware wasted.
///
/// The pairs are not interchangeable in general — the page pair turns pages in
/// a reader, and Left and Right adjust whatever holds focus — but on a screen
/// with nothing to adjust and nothing to page, all three walk the list. That is
/// what the firmware does, and it is why its bottom row is labelled Up and Down
/// over keys whose pins are called left and right.
#[test]
fn every_key_on_the_body_walks_the_list() {
    for (down, up) in [
        (Button::Down, Button::Up),
        (Button::PageForward, Button::PageBack),
        (Button::Right, Button::Left),
    ] {
        let mut ui = Ui::new(gallery::Menu::new(), on(Board::X3));

        ui.press(down);
        assert!(
            ui.changed(),
            "{down:?} did not move the selection — a key on the body that does \
             nothing on a list screen is a key the device wasted"
        );

        ui.press(up);
        assert!(ui.changed(), "{up:?} did not move it back");
    }
}

/// Confirm opens whatever is selected, wherever the selection got to.
#[test]
fn the_bottom_row_opens_what_it_selected() {
    let mut ui = Ui::new(gallery::Menu::new(), on(Board::X3));

    ui.press(Button::Right);
    ui.press(Button::Confirm);

    assert_eq!(ui.depth(), 2, "Select must open the highlighted row");
    assert!(
        ui.visible_text().iter().any(|line| line == "Lists"),
        "moving down once and confirming should open the second example. \
         Visible: {:#?}",
        ui.visible_text()
    );
}
