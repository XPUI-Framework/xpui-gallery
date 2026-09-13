//! The chrome a board gets, and whether a screen fits in it.
//!
//! A board is panel size, keys and millimetres; [`Metrics`] is what a painter
//! measures to. Nothing below this crate joins the two — that is
//! [`gallery::metrics_for`], and these are the claims it has to keep.
//!
//! A board stores no chrome, so it cannot disagree with the panel it describes:
//! there is nothing stored to drift. What is left is what a derivation cannot
//! make true by construction — that the numbers it produces are usable on the
//! glass.
//!
//! The millimetre floors are next door, in `tests/physical.rs`.

use std::sync::{Mutex, MutexGuard};

use gallery::{metrics_for, wire};
use xpui_boards_core::Board;
use xpui_eg::Palette;
use xpui_screenshot::Framebuffer as TestDisplay;
use xpui_simulator::{Panel, Session};

/// `Session::new` installs the process-wide host, so one at a time.
static SERIAL: Mutex<()> = Mutex::new(());

fn serial() -> MutexGuard<'static, ()> {
    SERIAL
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn display() -> TestDisplay {
    TestDisplay::new(200, 120)
}

/// Two rows to compare and one to show there is more. Below that a list is not
/// a list, and the screen is unusable rather than merely cramped.
///
/// One derivation, so one test: `tests/screenshots.rs` paints with the same
/// numbers and does not assert them again.
#[test]
fn every_board_holds_at_least_three_list_rows() {
    for board in gallery::boards::ALL {
        let rows = metrics_for(board).list_rows_for(board.height);
        assert!(
            rows >= 3,
            "{} ({}x{}) fits {} list rows",
            board.name,
            board.width,
            board.height,
            rows
        );
    }
}

/// A device with no row of keys along the bottom gets no hint bar, which would
/// name keys that do not exist.
///
/// The firmware's themes return before drawing one on exactly these boards.
///
/// The empty half of this is how [`metrics_for`] is written, and would survive
/// the function being wrong in every other way. The other half is the one that
/// bites: it says every keyed board's panel preset reserves a band at all, and
/// nothing in [`Metrics::for_panel`](xpui_chrome::Metrics::for_panel) promises
/// that — a preset added with the field left at zero would drop the hints on
/// that panel and paint an otherwise correct screen.
#[test]
fn only_a_board_with_keys_reserves_a_hint_band() {
    for board in gallery::boards::ALL {
        if board.keys.is_empty() {
            assert_eq!(
                metrics_for(board).button_hints_height,
                0,
                "{}: a board must not reserve a band for keys it lacks",
                board.name
            );
        } else {
            assert!(
                metrics_for(board).button_hints_height > 0,
                "{}: its keys need labelling",
                board.name
            );
        }
    }
}

/// A custom board driven by a finger gets no hint band either.
///
/// `Board::custom` is the escape hatch for a panel nobody here has described,
/// and it is the one board [`metrics_for`] can be handed that is not in
/// `gallery::boards::ALL` — so the loop above never reaches it. `custom` gives
/// a touch board an empty row, and a hint band derived from the row keeps a
/// finger's panel clear of one.
#[test]
fn a_custom_touch_board_gets_no_hint_band_either() {
    let finger = Board::custom("finger", 480, 800, true);
    assert_eq!(metrics_for(finger).button_hints_height, 0);

    let keys = Board::custom("keys", 480, 800, false);
    assert!(metrics_for(keys).button_hints_height > 0);
}

/// The simulator wires a backend exactly as [`gallery::wire`] does.
///
/// It cannot call `wire` — the gallery depends on the simulator, not the other
/// way round — so `xpui-simulator`'s `src/session.rs` repeats the
/// composition. Two copies of a derivation drift, and this is the only thing
/// that would notice: change the labels `wire` picks and every golden moves,
/// but the *window* would keep painting the old ones with nothing red.
///
/// Compared field by field rather than by rendering, because a difference in
/// `keys` paints nothing at all — it changes which key sends Back.
#[test]
fn the_simulator_wires_what_the_application_wires() {
    let _guard = serial();

    for board in gallery::boards::ALL {
        let session = Session::new(Panel::of(board));
        let theirs = session.backend();
        let mine = wire(display(), board, Palette::INK_IS_ON);

        assert_eq!(
            theirs.metrics(),
            mine.metrics(),
            "{}: the window measures differently from the firmware",
            board.name
        );
        assert_eq!(
            theirs.labels(),
            mine.labels(),
            "{}: the window says different words from the firmware",
            board.name
        );
        assert_eq!(
            theirs.keys(),
            mine.keys(),
            "{}: the window reads its keys differently from the firmware",
            board.name
        );
        assert_eq!(
            theirs.left_right_keys(),
            mine.left_right_keys(),
            "{}: the window and the firmware disagree about the Left/Right pair",
            board.name
        );
    }
}
