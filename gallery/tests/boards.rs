//! What the seven boards promise, taken together.
//!
//! Every assertion here compares one vendor's board to another's, or walks all
//! seven at once. **That has no home below the caller**: `xpui-boards-core`
//! describes no device, and `xpui-boards-pimoroni` knows nothing of Xteink's
//! panels or Seeed's. The gallery is the application that claims to fit all
//! seven, so this is where the claim is checked.
//!
//! What is *not* here: the vocabulary's own rules, in `xpui-boards`'s
//! `core/tests/vocabulary.rs`, and each vendor's own data, in its
//! `<vendor>/tests/<vendor>.rs`. A test that names one vendor
//! and no other belongs there — it is not a census, and putting it here would
//! make a Pimoroni change fail in a crate Pimoroni has never heard of.
//!
//! The chrome derived from these panels is next door, in
//! `chrome_for_a_board.rs` and `physical.rs`.

use gallery::boards::ALL;
use xpui::Button;
use xpui::host::RowKey;
use xpui_boards_core::{Bezel, Board, KeyAction, PhysicalButton};
use xpui_boards_pimoroni as pimoroni;
use xpui_boards_seeed as seeed;
use xpui_boards_xteink as xteink;

/// A slug has to survive the round trip, or `--board x3` opens something else.
#[test]
fn every_board_is_found_by_its_own_slug() {
    for board in ALL {
        assert_eq!(
            gallery::boards::from_slug(board.slug),
            Some(board),
            "{} calls itself {:?} and that does not find it",
            board.name,
            board.slug
        );
    }
}

/// The X4 and the Sticky are both 800x480 and differ in whether a finger works.
///
/// Identity is a stored slug rather than a pair of dimensions for exactly this
/// reason: a size-based match cannot tell these two apart, and an earlier
/// version returned the same answer for both.
#[test]
fn two_boards_with_one_panel_size_stay_distinct() {
    assert_eq!(
        (xteink::X4.width, xteink::X4.height),
        (seeed::STICKY.width, seeed::STICKY.height),
        "this test is pointless if they stop sharing a size"
    );

    assert_ne!(xteink::X4.slug, seeed::STICKY.slug);
    assert_ne!(xteink::X4, seeed::STICKY);

    // Const rather than runtime: these are compile-time facts, so a change to
    // either one fails the build rather than a test run.
    const _: () = assert!(!xteink::X4.touch, "the X4 has no touchscreen");
    const _: () = assert!(seeed::STICKY.touch, "the Sticky has one");
}

/// A board of some other size must not claim to be one of these.
#[test]
fn a_custom_board_does_not_borrow_another_slug() {
    let mine = Board::custom("Mine", 800, 480, false);

    for board in ALL {
        assert_ne!(
            mine.slug, board.slug,
            "a custom board reported itself as {}",
            board.name
        );
    }
    assert_eq!(gallery::boards::from_slug(mine.slug), None);
}

/// Slugs are what a command line accepts, so two boards cannot share one.
#[test]
fn no_two_boards_share_a_slug() {
    for (index, board) in ALL.iter().enumerate() {
        for other in &ALL[index + 1..] {
            assert_ne!(
                board.slug, other.slug,
                "{} and {} both answer to {:?}",
                board.name, other.name, board.slug
            );
        }
    }
}

/// An unknown name is not silently one of the presets.
#[test]
fn an_unknown_slug_finds_nothing() {
    for name in ["", "reader", "reader-landscape", "x5", "badger2041"] {
        assert_eq!(
            gallery::boards::from_slug(name),
            None,
            "{name:?} matched something"
        );
    }
}

// -- how big a pixel is ----------------------------------------------------

/// A panel with no diagonal cannot be asked how big its chrome is, and every
/// preset here has to be able to answer.
#[test]
fn every_preset_panel_has_been_measured() {
    for board in ALL {
        assert!(
            board.ppi().is_some(),
            "{} has no diagonal, so nothing can say what its chrome measures",
            board.name
        );
    }

    assert_eq!(
        Board::custom("Mine", 800, 480, false).ppi(),
        None,
        "a panel nobody has measured says so rather than guessing"
    );
}

/// The derivation, against the densities these panels are sold with. A digit
/// wrong in a diagonal is invisible until something is measured in
/// millimetres, and then it is wrong everywhere at once.
#[test]
fn the_derived_density_matches_the_panel() {
    for (board, published) in [
        (xteink::X3, 257),
        (xteink::X4, 218),
        (xteink::X4_PRO, 218),
        (seeed::STICKY, 234),
        (pimoroni::BADGER_2040, 111),
        (pimoroni::TUFTY_2040, 166),
    ] {
        let ppi = board.ppi().expect("a measured panel");
        assert!(
            (ppi - published).abs() <= 2,
            "{}: {}x{} over {}\" derives {ppi} ppi, not the {published} it is sold as",
            board.name,
            board.framebuffer.0,
            board.framebuffer.1,
            board.diagonal_hundredths_inch.unwrap_or(0) as f32 / 100.0
        );
    }
}

/// The scale is the firmware's, board for board: a finger bumps it, a button
/// does not.
///
/// If a board ever needs one without the other, this is the place to say why —
/// the rule is `BoardConfig`'s, not an accident of which boards exist.
#[test]
fn the_touch_boards_are_the_scaled_ones() {
    for board in ALL {
        let expected = if board.touch { 120 } else { 100 };
        assert_eq!(
            board.ui_scale_percent,
            expected,
            "{} is a {} board",
            board.name,
            if board.touch { "touch" } else { "button" }
        );
    }
}

// -- bezels ----------------------------------------------------------------

fn bezels() -> impl Iterator<Item = (Board, Bezel)> {
    ALL.into_iter()
        .filter_map(|board| board.bezel.map(|bezel| (board, bezel)))
}

/// A button has to be somewhere a thumb can reach: on the body, and not on top
/// of the screen.
#[test]
fn every_button_is_on_the_body_and_off_the_panel() {
    for (board, bezel) in bezels() {
        let (px, py, pw, ph) = bezel.panel_rect();

        for button in bezel.buttons {
            let (cx, cy) = button.centre;
            let (w, h) = button.size;
            let (left, top) = (cx - w / 2, cy - h / 2);
            let (right, bottom) = (cx + w / 2, cy + h / 2);

            assert!(
                left >= 0 && top >= 0 && right <= bezel.body.0 && bottom <= bezel.body.1,
                "{}: {:?} is off the body",
                board.name,
                button.label
            );

            let overlaps = right > px && left < px + pw && bottom > py && top < py + ph;
            assert!(
                !overlaps,
                "{}: {:?} sits on the panel. A physical button belongs beside \
                 the screen — one drawn over it would take taps the firmware \
                 should have had",
                board.name, button.label
            );
        }
    }
}

/// The glass a body is drawn around is the glass its diagonal describes.
///
/// Both are written by hand — the diagonal in hundredths of an inch, the plan's
/// panel in tenths of a millimetre — and they answer different callers: every
/// millimetre a screen asks for comes from the diagonal, and the simulator
/// draws the plan. Two percent is the rounding in `ppi`.
#[test]
fn every_body_holds_the_panel_its_diagonal_describes() {
    let mut checked = 0;
    for (board, bezel) in bezels() {
        let ppi = board.ppi().expect("a measured panel");
        let described = (board.width * 254 / ppi, board.height * 254 / ppi);
        for (drawn, wanted, axis) in [
            (bezel.panel_size.0, described.0, "wide"),
            (bezel.panel_size.1, described.1, "tall"),
        ] {
            assert!(
                (drawn - wanted).abs() * 50 <= wanted,
                "{}: the body is drawn around glass {drawn} tenths of a mm {axis}, \
                 and the diagonal makes it {wanted}",
                board.name
            );
        }
        checked += 1;
    }
    assert_eq!(checked, ALL.len(), "a board has no body to check");
}

/// A board whose footer carries no keys describes no row.
///
/// A row over keys that are not there paints nothing while the board reserves
/// no hint band, and waits for the first consumer that asks for one. The
/// X4 Pro's Home pad sits below its panel but is a gesture, not a row key.
#[test]
fn a_board_with_no_footer_describes_no_row() {
    let mut without = 0;
    for (board, bezel) in bezels() {
        let footer = footer_of(bezel)
            .into_iter()
            .filter(|key| key.action != KeyAction::Home)
            .count();
        assert_eq!(
            board.keys.is_empty(),
            footer == 0,
            "{}: its row describes {} keys and its footer carries {footer}",
            board.name,
            board.keys.len()
        );
        if footer == 0 {
            without += 1;
        }
    }
    assert_eq!(without, 2, "the X4 Pro and the Sticky carry no footer row");
}

/// The panel has to fit inside the body it is set into.
#[test]
fn the_panel_fits_in_the_body() {
    for (board, bezel) in bezels() {
        let (x, y, w, h) = bezel.panel_rect();
        assert!(
            x >= 0 && y >= 0 && x + w <= bezel.body.0 && y + h <= bezel.body.1,
            "{}: the panel hangs off the body",
            board.name
        );
    }
}

/// Two buttons in the same place means one of them can never be pressed.
#[test]
fn no_two_buttons_overlap() {
    for (board, bezel) in bezels() {
        for (index, first) in bezel.buttons.iter().enumerate() {
            for second in &bezel.buttons[index + 1..] {
                let apart = (first.centre.0 - second.centre.0).abs()
                    >= (first.size.0 + second.size.0) / 2
                    || (first.centre.1 - second.centre.1).abs()
                        >= (first.size.1 + second.size.1) / 2;
                assert!(
                    apart,
                    "{}: {:?} and {:?} overlap",
                    board.name, first.label, second.label
                );
            }
        }
    }
}

/// Hit-testing has to find the button you aimed at, and nothing where there is
/// none.
#[test]
fn a_press_finds_the_button_under_it() {
    for (board, bezel) in bezels() {
        for button in bezel.buttons {
            assert_eq!(
                bezel.button_at(button.centre).map(|found| found.label),
                Some(button.label),
                "{}: pressing the middle of {:?} found something else",
                board.name,
                button.label
            );
        }

        // The middle of the panel is not a button on any of these.
        let (x, y, w, h) = bezel.panel_rect();
        assert!(
            bezel.button_at((x + w / 2, y + h / 2)).is_none(),
            "{}: the middle of the screen reported a button",
            board.name
        );
    }
}

// -- orientation -----------------------------------------------------------

/// The readers scan landscape and are held portrait, so their canvas is the
/// framebuffer turned a quarter. Getting this backwards lays every screen out
/// against the wrong shape.
#[test]
fn a_portrait_board_presents_its_framebuffer_turned() {
    for board in ALL {
        let (width, height) = board.orientation.canvas(board.framebuffer);
        assert_eq!(
            (width, height),
            (board.width, board.height),
            "{}: a {:?} board with a {}x{} framebuffer presents {}x{}, not {}x{}",
            board.name,
            board.orientation,
            board.framebuffer.0,
            board.framebuffer.1,
            width,
            height,
            board.width,
            board.height
        );
    }
}

/// Every Xteink reader is used upright, whatever way its controller scans.
#[test]
fn the_readers_are_portrait() {
    use xpui_boards_core::Orientation;

    for board in [xteink::X3, xteink::X4, xteink::X4_PRO, seeed::STICKY] {
        assert_eq!(board.orientation, Orientation::Portrait, "{}", board.name);
        assert!(
            board.height > board.width,
            "{}: a portrait canvas is taller than it is wide",
            board.name
        );
        assert!(
            board.framebuffer.0 > board.framebuffer.1,
            "{}: these panels scan landscape",
            board.name
        );
    }
}

/// The touch boards are the ones with a touchscreen, and only those.
#[test]
fn touch_is_recorded_where_the_hardware_has_it() {
    // Const, because these are compile-time facts: changing one fails the
    // build rather than a test run.
    const _: () = assert!(xteink::X4_PRO.touch, "the X4 Pro has a touchscreen");
    const _: () = assert!(seeed::STICKY.touch, "the Sticky has one");

    for board in [
        xteink::X3,
        xteink::X4,
        pimoroni::BADGER_2040,
        pimoroni::TUFTY_2040,
    ] {
        assert!(!board.touch, "{} has buttons only", board.name);
    }
}

/// A panel sits in the middle of what is left for it.
///
/// Not simply in the middle of the body: a badge carries both its side keys on
/// one edge, so its screen is pushed off-centre by exactly the column those
/// keys need. Asserting plain symmetry there would be asserting the device is
/// something other than it is.
#[test]
fn every_panel_is_centred_in_the_room_it_has() {
    for (board, bezel) in bezels() {
        let (x, _, width, _) = bezel.panel_rect();
        let (left, right) = (x, bezel.body.0 - (x + width));

        let (_, top, _, height) = bezel.panel_rect();
        // Only keys level with the screen take room from its margins. One below
        // it is in the footer and costs the sides nothing.
        let flanking = |on_the_right: bool| {
            bezel
                .buttons
                .iter()
                .filter(|key| key.centre.1 < top + height)
                .filter(|key| {
                    let beside = key.centre.0 < x || key.centre.0 > x + width;
                    beside && (key.centre.0 > x) == on_the_right
                })
                .map(|key| key.size.0)
                .max()
                .unwrap_or(0)
        };

        // What each side needs for the keys on it, and what is left over.
        let spare = |margin: i32, keys: i32| margin - keys;

        assert!(
            (spare(left, flanking(false)) - spare(right, flanking(true))).abs() <= 2,
            "{}: {left} on the left and {right} on the right, with {} and {} of \
             that taken by keys — the screen is not centred in what remains",
            board.name,
            flanking(false),
            flanking(true)
        );
    }
}

/// How many keys sit below the panel, which is how many hints there is room to
/// label.
fn bottom_keys(board: Board) -> u8 {
    let Some(bezel) = board.bezel else { return 4 };
    let (_, y, _, height) = bezel.panel_rect();

    bezel
        .buttons
        .iter()
        .filter(|key| key.centre.1 > y + height)
        .count() as u8
}

/// A board must not label more keys than it has.
///
/// Four hints over three keys is worse than none: every label after the first
/// sits over the wrong key, and the last names one that is not there.
///
/// The row is written by hand and the count is derived from where the bezel
/// actually puts the keys, so this is the two disagreeing rather than a
/// restatement of one of them.
#[test]
fn a_board_labels_only_the_keys_it_has() {
    for board in ALL {
        if board.keys.is_empty() {
            continue;
        }
        assert_eq!(
            board.keys.len(),
            bottom_keys(board) as usize,
            "{}: its row describes {} keys and its body has {}",
            board.name,
            board.keys.len(),
            bottom_keys(board)
        );
    }
}

/// The hint painted over a key names the job that key actually does.
///
/// Two descriptions of the same thing sit in `xpui-boards`: the row, which
/// is what the hint bar paints, and the footer keys, which are what pressing
/// one sends. Nothing makes them agree, and they did not — a change to the row
/// left the keys alone, so on both Pimoroni boards every label sat one key to
/// the left of what it named and no key anywhere sent `Back`. The suite was
/// green and a person found it by clicking.
///
/// This is the check that was missing. It walks every board rather than the
/// one someone happened to open.
#[test]
fn a_boards_keys_match_the_row_it_paints() {
    // Three `continue`s below, and a board added before its body is described
    // would slip past all of them. A test that skips everything passes.
    let mut checked = 0;

    for board in ALL {
        // A board with no row paints no hint band, so there is nothing to
        // agree with.
        if board.keys.is_empty() {
            continue;
        }
        let Some(bezel) = board.bezel else { continue };
        let keys = footer_of(bezel);

        assert_eq!(
            board.keys.len(),
            keys.len(),
            "{}: its row describes {} keys and its footer has {}",
            board.name,
            board.keys.len(),
            keys.len()
        );

        checked += 1;
        for (index, (row_key, key)) in board.keys.iter().zip(&keys).enumerate() {
            assert_eq!(
                row_key,
                names(key.action),
                "{}: slot {} paints {:?} over a key that does {:?}",
                board.name,
                index,
                row_key,
                key.action
            );
        }
    }

    // Five boards paint a hint bar: the X3, the X4, and all three Pimoroni
    // boards. The X4 Pro and the Sticky take Back from a touchscreen and have
    // none. Counted by name rather than by family, because "readers" and
    // "badges" split these five two different ways depending on who is asked.
    assert_eq!(
        checked, 5,
        "the loop skipped a board it should have checked"
    );
}

/// Every board reports the pair its bezel actually carries.
///
/// Not a restatement of `has_left_right_keys`: the expected answer is written
/// out per board, because the answers are not the ones anybody guesses. The
/// obvious split is "readers yes, badges no" and it is wrong twice — the X4 Pro
/// is a reader that says no, and the Inky Frame is a badge that says yes.
///
/// The pair is what lets a value be nudged where it stands. A board without one
/// needs a control that can be entered and left instead, which is why anything
/// reading this must read it rather than assume.
#[test]
fn every_board_reports_the_pair_its_bezel_carries() {
    // Why each is what it is, so a bezel change that moves one of these has to
    // be an argued edit rather than a re-blessed number.
    let expected = |slug: &str| match slug {
        // The readers' shared footer sends Left and Right on its third and
        // fourth keys, whatever the labels above them read.
        "x3" | "x4" => Some(true),
        // A reader with no footer at all: it takes back, confirm, left and
        // right from the touchscreen, and the keys it does wire turn pages and
        // sleep it.
        "x4pro" => Some(false),
        // Three keys: one confirms, and the pair below it turns pages rather
        // than moving a value.
        "sticky" => Some(false),
        // Five keys, and the two beside the panel are Up and Down — busy
        // walking the list.
        "badger2040" | "tufty2040" => Some(false),
        // Five along the footer, and the third and fourth are the pair.
        "inkyframe" => Some(true),
        _ => None,
    };

    // Driven from `ALL`, not from the table: a board added without an
    // answer fails here rather than being skipped, and a board dropped from the
    // table cannot be hidden by another one being listed twice.
    let mut with_pair = 0;
    for board in ALL {
        let wanted = expected(board.slug).unwrap_or_else(|| {
            panic!(
                "{} was added without deciding whether it has the pair",
                board.name
            )
        });
        assert_eq!(
            board.has_left_right_keys(),
            wanted,
            "{} reports the wrong pair",
            board.name
        );
        if wanted {
            with_pair += 1;
        }
    }

    assert_eq!(
        with_pair, 3,
        "three boards carry the pair — the X3, the X4 and the Inky Frame"
    );
}

/// No board names the same job on two keys, and every board can be left.
///
/// A row is written by hand, and a duplicated entry is the kind of typo that
/// paints plausibly — two keys both labelled `Back`, one of which does
/// nothing. A row with no `Back` at all is legitimate (a three-key badge
/// spends its keys elsewhere and reaches Back by a double press), so that is
/// asserted separately where it is true.
#[test]
fn a_row_gives_each_job_to_at_most_one_key() {
    for board in ALL {
        let mut seen: Vec<RowKey> = Vec::new();
        for key in board.keys.iter() {
            if key == RowKey::Unassigned {
                continue;
            }
            assert!(
                !seen.contains(&key),
                "{}: {:?} is on two keys of the same row",
                board.name,
                key
            );
            seen.push(key);
        }
    }
}

/// Every board with a bottom row keeps Back and Confirm reachable from it.
///
/// The five-key boards here all have a key to spare, so spending one on Back
/// costs nothing — and a board that can be entered but not left is the fault
/// this pins. A genuine three-key badge would be the exception, and there is
/// not one in `ALL`; if one is added, this test is the conversation.
#[test]
fn every_row_can_be_entered_and_left() {
    for board in ALL {
        if board.touch {
            continue;
        }
        assert!(
            board.keys.contains(RowKey::Back),
            "{}: its row has no Back key",
            board.name
        );
        assert!(
            board.keys.contains(RowKey::Confirm),
            "{}: its row has no Confirm key",
            board.name
        );
    }
}

// -- the shapes the keys are in --------------------------------------------
//
// Every body here is described as a row along the footer and a column down an
// edge, with the centres derived rather than written down. These say what
// "derived" has to come out as, so a plan that spaced a run by hand — or by
// arithmetic that is subtly wrong — is caught the way the misplaced panels
// were.

/// The keys below the panel, left to right: a board's footer row.
fn footer_of(bezel: Bezel) -> Vec<PhysicalButton> {
    let (_, y, _, height) = bezel.panel_rect();
    let mut keys: Vec<PhysicalButton> = bezel
        .buttons
        .iter()
        .copied()
        .filter(|key| key.centre.1 > y + height)
        .collect();
    keys.sort_by_key(|key| key.centre.0);
    keys
}

/// The keys beside the panel on one side, top to bottom: a board's edge column.
fn column_of(bezel: Bezel, on_the_right: bool) -> Vec<PhysicalButton> {
    let (x, y, width, height) = bezel.panel_rect();
    let mut keys: Vec<PhysicalButton> = bezel
        .buttons
        .iter()
        .copied()
        .filter(|key| key.centre.1 < y + height)
        .filter(|key| {
            let beside = key.centre.0 < x || key.centre.0 > x + width;
            beside && (key.centre.0 > x) == on_the_right
        })
        .collect();
    keys.sort_by_key(|key| key.centre.1);
    keys
}

/// The shell between one key and the next, along a run.
///
/// Between the *edges* rather than between the centres: a sleep key is shorter
/// than the page keys beside it, so even centres would be uneven shell — and
/// the shell is what a thumb feels.
fn gaps(along: impl Iterator<Item = (i32, i32)>) -> Vec<i32> {
    let mut gaps = Vec::new();
    let mut previous: Option<i32> = None;
    for (centre, size) in along {
        if let Some(end) = previous {
            gaps.push(centre - size / 2 - end);
        }
        previous = Some(centre + size / 2);
    }
    gaps
}

/// A gap of one tenth of a millimetre either way, which is integer division
/// dividing a body by five rather than anything anybody would see.
fn evenly_spaced(gaps: &[i32]) -> bool {
    match (gaps.iter().min(), gaps.iter().max()) {
        (Some(least), Some(most)) => most - least <= 1,
        _ => true,
    }
}

/// A footer of three and a footer of five differ by a number, not by a table of
/// positions — so the shell between one key and the next is the same all the
/// way along, and the same at both ends of the row.
#[test]
fn a_row_of_keys_is_evenly_spaced() {
    let mut rows = 0;
    for (board, bezel) in bezels() {
        let keys = footer_of(bezel);
        if keys.len() < 2 {
            continue;
        }
        rows += 1;

        let between = gaps(keys.iter().map(|key| (key.centre.0, key.size.0)));
        assert!(
            evenly_spaced(&between),
            "{}: the keys along the footer are spaced {between:?}",
            board.name
        );

        let first = &keys[0];
        let last = &keys[keys.len() - 1];
        let (before, after) = (
            first.centre.0 - first.size.0 / 2,
            bezel.body.0 - (last.centre.0 + last.size.0 / 2),
        );
        assert!(
            (before - after).abs() <= 1,
            "{}: {before} of shell before the row and {after} after it — the \
             row is not centred on the body",
            board.name
        );

        for key in &keys {
            assert_eq!(
                key.size, first.size,
                "{}: {:?} is not the size of the rest of the row",
                board.name, key.label
            );
        }
    }
    assert!(rows >= 4, "only {rows} boards have a row to check");
}

/// The same, down an edge — and a column's keys share a width even when a sleep
/// key makes them differ in height.
#[test]
fn a_column_of_keys_is_evenly_spaced() {
    let mut columns = 0;
    for (board, bezel) in bezels() {
        for on_the_right in [false, true] {
            let keys = column_of(bezel, on_the_right);
            if keys.len() < 2 {
                continue;
            }
            columns += 1;

            let between = gaps(keys.iter().map(|key| (key.centre.1, key.size.1)));
            assert!(
                evenly_spaced(&between),
                "{}: the keys down its {} edge are spaced {between:?}",
                board.name,
                if on_the_right { "right" } else { "left" }
            );

            for key in &keys {
                assert_eq!(
                    key.size.0, keys[0].size.0,
                    "{}: {:?} is not the width of the rest of the column",
                    board.name, key.label
                );
                assert_eq!(
                    key.centre.0, keys[0].centre.0,
                    "{}: {:?} is out of the column",
                    board.name, key.label
                );
            }
        }
    }
    assert!(columns >= 4, "only {columns} boards have a column to check");
}

/// Which hint names a key, given what pressing it sends.
///
/// `Previous` and `Next` are `Left` and `Right`: a reader's footer calls them
/// Up and Down because that is what they do to a list, and the pins are named
/// for the direction. Anything the hint vocabulary has no word for — a Power
/// key, an unassigned one — is [`RowKey::Unassigned`], which draws nothing.
fn names(action: KeyAction) -> RowKey {
    match action {
        KeyAction::Press(Button::Back) => RowKey::Back,
        KeyAction::Press(Button::Confirm) => RowKey::Confirm,
        KeyAction::Press(Button::Left) => RowKey::Previous,
        KeyAction::Press(Button::Right) => RowKey::Next,
        // Every remaining action, spelled out. A catch-all here reads the same
        // and lets half the fault through: a key that gains a job the bar has
        // no word for still maps to `Unassigned`, so the row keeps painting
        // nothing over a key that now does something, and this test — whose
        // whole purpose is to notice that — stays green.
        KeyAction::Press(Button::Up)
        | KeyAction::Press(Button::Down)
        | KeyAction::Press(Button::Power)
        | KeyAction::Press(Button::PageBack)
        | KeyAction::Press(Button::PageForward)
        | KeyAction::Press(Button::NavNext)
        | KeyAction::Press(Button::NavPrevious)
        | KeyAction::Press(Button::ScreenLeft)
        | KeyAction::Press(Button::ScreenRight)
        | KeyAction::Press(Button::ScreenUp)
        | KeyAction::Press(Button::ScreenDown)
        | KeyAction::Home
        | KeyAction::Unassigned => RowKey::Unassigned,
    }
}

/// The slugs the error message lists are the boards `--board` accepts.
///
/// [`gallery::boards::slugs`] has one caller: the message printed when
/// `--board` is given something it does not recognise. Nothing else reads it,
/// so nothing else would notice it listing a board that cannot be opened, or
/// omitting one that can — which is a person typing exactly what they were
/// told and being refused.
#[test]
fn every_slug_offered_is_a_slug_that_opens() {
    let offered: Vec<&str> = gallery::boards::slugs().collect();
    assert_eq!(offered.len(), ALL.len(), "one slug per board, and no more");

    for slug in offered {
        assert!(
            gallery::boards::from_slug(slug).is_some(),
            "{slug} is offered and opens nothing"
        );
    }
}

/// Each vendor answers for its own boards and declines the others'.
///
/// The composed lookup asks the three in turn, so a vendor that answered for a
/// slug it does not own would shadow the crate that does — and the board that
/// opened would be the wrong one, with the right name on the command line.
#[test]
fn a_vendor_answers_only_for_its_own() {
    for board in ALL {
        let owners = [
            xteink::from_slug(board.slug),
            seeed::from_slug(board.slug),
            pimoroni::from_slug(board.slug),
        ];
        let claimed: Vec<Board> = owners.into_iter().flatten().collect();
        assert_eq!(
            claimed,
            vec![board],
            "{} is claimed by {} vendor(s)",
            board.slug,
            claimed.len()
        );
    }
}
