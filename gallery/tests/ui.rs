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

// -- a value row, reached and changed with keys -------------------------------
//
// A value row has to be reachable by whatever keys a board has. On a board with
// a Left/Right pair the keys must also keep their meaning; on one without, they
// deliberately change, and saying so on the panel is still to come.

/// The value beside `label`, for a panel small enough to scroll one out of
/// view. Reads the first percentage at or after the label rather than counting
/// from the top, so which rows happen to be on screen does not matter.
fn value_of(ui: &Ui<Backend<Framebuffer>>, label: &str) -> String {
    let text = ui.visible_text();
    let at = text
        .iter()
        .position(|line| line == label)
        .unwrap_or_else(|| panic!("{label:?} is not on screen. Visible: {text:#?}"));
    text[at..]
        .iter()
        .find(|line| line.ends_with('%'))
        .unwrap_or_else(|| panic!("{label:?} has no value beside it. Visible: {text:#?}"))
        .clone()
}

/// Both percentages on the Controls screen, brightness first.
///
/// Asserted to be exactly two, so a layout change that adds a third fails here
/// rather than silently moving which one a caller reads.
fn percentages(ui: &Ui<Backend<Framebuffer>>) -> Vec<String> {
    let found: Vec<String> = ui
        .visible_text()
        .into_iter()
        .filter(|line| line.ends_with('%'))
        .collect();
    assert_eq!(
        found.len(),
        2,
        "Controls should show brightness and warmth. Visible: {:#?}",
        ui.visible_text()
    );
    found
}

/// The Warmth row can be reached by the keys and changed by them.
///
/// The X3 has the pair and no touchscreen, so keys are the only way in. A
/// slider declaring touch alone is stepped over by Up and Down and never
/// reached by Left and Right, which leaves the row inert on every button-only
/// board — the fault a person found by opening the simulator, on a board this
/// suite already covered.
#[test]
fn the_warmth_row_can_be_reached_and_changed() {
    let mut ui = Ui::new(gallery::Menu::new(), on(Board::X3));
    open(&mut ui, "Controls");

    let before = percentages(&ui);
    ui.press(Button::Down);
    ui.press(Button::Right);
    let after = percentages(&ui);

    assert_eq!(
        after[0], before[0],
        "Down should have left brightness behind, not adjusted it"
    );
    assert_ne!(
        after[1], before[1],
        "one Down from brightness should focus Warmth, and Right should change it"
    );
}

/// A board with no Left/Right pair can still reach and change a value.
///
/// The Badger's five keys are Back, Confirm, a bare third, and an up/down pair
/// that walks the list. With no Left/Right among them a value has to be entered
/// and left again, so Confirm opens the row and the list keys move it.
///
/// What that mode still lacks is any sign of itself on the panel; this pins the
/// behaviour, not the appearance.
///
/// Pinned on a Badger because the other new tests here run on an X3, where the
/// `!has_left_right_keys()` branch never executes.
#[test]
fn a_value_row_is_reachable_on_a_board_with_no_pair() {
    let mut ui = Ui::new(gallery::Menu::new(), on(Board::BADGER_2040));
    open(&mut ui, "Controls");

    // Read by label: this panel is 296x128 and scrolls, so which rows are on
    // screen changes as the focus moves.
    let before = value_of(&ui, "Warmth");

    // Down walks to the Warmth row; on this board Left and Right do not exist,
    // so Confirm is the way in and must not fire the control instead.
    ui.press(Button::Down);
    ui.press(Button::Confirm);
    assert_eq!(
        value_of(&ui, "Warmth"),
        before,
        "Confirm opens a value row on a board with no pair; it must not set it"
    );

    // The keys that walked the list now move the value, which is the whole
    // reason the mode exists.
    ui.press(Button::Up);
    assert_ne!(
        value_of(&ui, "Warmth"),
        before,
        "with the value open, the list keys move it"
    );

    ui.press(Button::Back);
    assert_eq!(
        value_of(&ui, "Warmth"),
        before,
        "and Back puts it back exactly"
    );
    assert_eq!(ui.depth(), 2, "without leaving the screen");
}

/// Confirm on a value row does not quietly change what the other keys mean.
///
/// Spec 26 opens an edit mode on Confirm that repaints an identical frame: Up
/// and Down stop moving between rows and adjust instead, and Back stops leaving
/// the screen. Whatever replaces it has to be visible or must not happen — a
/// person cannot be expected to discover that four keys changed meaning.
#[test]
fn confirm_does_not_silently_change_what_the_keys_mean() {
    let mut ui = Ui::new(gallery::Menu::new(), on(Board::X3));
    open(&mut ui, "Controls");

    let before = percentages(&ui);
    ui.press(Button::Confirm);
    ui.press(Button::Down);

    assert_eq!(
        percentages(&ui)[0],
        before[0],
        "Down moves between rows; after Confirm it adjusted brightness instead"
    );

    // And on the row below, which is a `Slider` — an absolute control, whose
    // trigger resolves at the centre of its own track. Firing that from a focus
    // rather than a touch is what set it to 50%.
    let before = percentages(&ui);
    ui.press(Button::Confirm);
    assert_eq!(
        percentages(&ui)[1],
        before[1],
        "Confirm on a focused slider must not jump it to the middle of its track"
    );

    ui.press(Button::Back);
    assert_eq!(
        ui.depth(),
        1,
        "Back leaves the screen; after Confirm it was spent cancelling an edit"
    );
}
