//! Back, on a board with no key for it.

use std::sync::{Mutex, MutexGuard};

use gallery::Menu;
use gallery::chord::{Badge, Chord, DOUBLE_PRESS_MS, Doubles, back_stands_on};
use xpui::{App, Button};
use xpui_chrome::RowKey;
use xpui_simulator::{Board, Keypad, Panel, Session, open_frame};

/// `Session::new` installs the process-wide host, so one test at a time.
static SERIAL: Mutex<()> = Mutex::new(());

fn serial() -> MutexGuard<'static, ()> {
    SERIAL
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// A row without a Back key borrows the first one; a row with its own does not.
///
/// Asked of the row, not of its length. A three-key row that spends a key on
/// Back — which is what both Pimoroni boards now do — must not be charged the
/// double-press delay for a key it has.
#[test]
fn only_a_row_without_back_borrows_a_key_for_it() {
    use RowKey::{Back, Confirm, Unassigned};

    assert_eq!(
        back_stands_on(NO_BACK_KEY),
        Some(Button::Confirm),
        "no key of its own for Back, so the first one carries it"
    );
    assert_eq!(
        back_stands_on(HAS_BACK_KEY),
        None,
        "a reader's row has a Back key"
    );
    assert_eq!(
        back_stands_on(&[Back, Confirm, Unassigned]),
        None,
        "three keys, but one of them is Back — no stand-in needed"
    );
    assert_eq!(
        back_stands_on(&[]),
        None,
        "a touch board takes Back from the glass"
    );
}

/// The row of a badge with no key to spare for Back.
const NO_BACK_KEY: &[RowKey] = &[RowKey::Confirm, RowKey::Previous, RowKey::Next];

/// A reader's row, which has one.
const HAS_BACK_KEY: &[RowKey] = &[
    RowKey::Back,
    RowKey::Confirm,
    RowKey::Previous,
    RowKey::Next,
];

/// A three-key badge: a, b and c along the bottom and nothing down the edges,
/// so there is no key to spare for Back and it is borrowed from the first.
///
/// Nothing in `Board::ALL` is arranged this way any more — the Badger and the
/// Tufty both have an up/down pair and give their first key to Back. The
/// arrangement is still what this module exists for, so the tests name it
/// outright rather than borrowing a board that has since grown out of it.
fn three_key_badge() -> Board {
    let mut board = Board::BADGER_2040;
    board.tokens = board
        .tokens
        .with_row(&[RowKey::Confirm, RowKey::Previous, RowKey::Next]);
    board
}

#[test]
fn two_quick_presses_are_back() {
    let mut doubles = Doubles::default();
    let back = back_stands_on(NO_BACK_KEY);

    assert_eq!(
        doubles.press(Button::Confirm, 0, back),
        Chord::Single(Button::Confirm)
    );
    assert_eq!(doubles.press(Button::Confirm, 200, back), Chord::Back);
}

/// Two confirmations far apart are two confirmations, not a Back.
#[test]
fn two_slow_presses_are_two_presses() {
    let mut doubles = Doubles::default();
    let back = back_stands_on(NO_BACK_KEY);

    doubles.press(Button::Confirm, 0, back);
    assert_eq!(
        doubles.press(Button::Confirm, DOUBLE_PRESS_MS + 1, back),
        Chord::Single(Button::Confirm),
        "past the window it is a second confirmation"
    );
}

/// Three presses are a Back and then a fresh single, not two overlapping
/// doubles.
#[test]
fn a_third_press_starts_again() {
    let mut doubles = Doubles::default();
    let back = back_stands_on(NO_BACK_KEY);

    doubles.press(Button::Confirm, 0, back);
    assert_eq!(doubles.press(Button::Confirm, 100, back), Chord::Back);
    assert_eq!(
        doubles.press(Button::Confirm, 200, back),
        Chord::Single(Button::Confirm)
    );
}

/// Another key in between breaks the pair — you meant two different things.
#[test]
fn a_different_key_between_them_breaks_the_pair() {
    let mut doubles = Doubles::default();
    let back = back_stands_on(NO_BACK_KEY);

    doubles.press(Button::Confirm, 0, back);
    doubles.press(Button::Left, 50, back);
    assert_eq!(
        doubles.press(Button::Confirm, 100, back),
        Chord::Single(Button::Confirm),
        "the run was interrupted"
    );
}

/// Every other key passes straight through, however fast you press it.
#[test]
fn only_the_borrowed_key_doubles() {
    let mut doubles = Doubles::default();
    let back = back_stands_on(NO_BACK_KEY);

    assert_eq!(
        doubles.press(Button::Left, 0, back),
        Chord::Single(Button::Left)
    );
    assert_eq!(
        doubles.press(Button::Left, 50, back),
        Chord::Single(Button::Left)
    );
}

/// A reader has a Back key, so double-pressing Select must stay two Selects.
#[test]
fn a_board_with_a_back_key_never_borrows_one() {
    let mut doubles = Doubles::default();
    let back = back_stands_on(Board::X3.tokens.row);

    assert_eq!(
        doubles.press(Button::Confirm, 0, back),
        Chord::Single(Button::Confirm)
    );
    assert_eq!(
        doubles.press(Button::Confirm, 50, back),
        Chord::Single(Button::Confirm),
        "the X3 has four keys along the bottom, one of which is Back"
    );
}

// -- holding the select back ----------------------------------------------
//
// Recognising the double press is only half of it. The other half is not
// acting on the first press until it is certain there was no second one —
// otherwise Back arrives after something has already opened, and going back
// from a screen is not the same as never having opened it.

/// The badge's key: swallowed on the way down, delivered when the window shuts.
#[test]
fn the_borrowed_key_waits_out_its_window() {
    let mut badge = Badge::default();
    let back = back_stands_on(NO_BACK_KEY);

    assert_eq!(
        badge.pressed(Button::Confirm, 1_000, back),
        None,
        "it might yet turn out to be half of a Back"
    );
    assert_eq!(
        badge.due(1_000 + DOUBLE_PRESS_MS - 1),
        None,
        "one millisecond short, a second press could still arrive"
    );
    assert_eq!(
        badge.due(1_000 + DOUBLE_PRESS_MS),
        Some(Button::Confirm),
        "the window closed with nothing behind it, so it was a select"
    );
}

/// Once delivered, it is gone. A press handed over twice opens a screen and
/// then opens whatever the first row of that screen was.
#[test]
fn a_held_press_is_delivered_once() {
    let mut badge = Badge::default();
    let back = back_stands_on(NO_BACK_KEY);

    badge.pressed(Button::Confirm, 0, back);
    assert_eq!(badge.due(DOUBLE_PRESS_MS), Some(Button::Confirm));
    assert_eq!(
        badge.due(DOUBLE_PRESS_MS + 1),
        None,
        "there is no second one"
    );
    assert_eq!(badge.due(10_000), None, "nor a thousand frames later");
}

/// The second press supersedes the first, which must not then arrive late.
#[test]
fn a_second_press_cancels_the_one_being_held() {
    let mut badge = Badge::default();
    let back = back_stands_on(NO_BACK_KEY);

    assert_eq!(badge.pressed(Button::Confirm, 0, back), None);
    assert_eq!(
        badge.pressed(Button::Confirm, 100, back),
        Some(Button::Back),
        "two inside the window are a Back, and that one is not delayed"
    );
    assert_eq!(
        badge.due(10_000),
        None,
        "the select it was holding was superseded, not merely postponed — \
         delivering it later would go back and then open something"
    );
}

/// Every other key acts on the frame it was pressed. Only the borrowed one
/// pays for the arrangement.
#[test]
fn the_other_keys_are_not_delayed() {
    let mut badge = Badge::default();
    let back = back_stands_on(NO_BACK_KEY);

    assert_eq!(badge.pressed(Button::Left, 0, back), Some(Button::Left));
    assert_eq!(badge.pressed(Button::Right, 10, back), Some(Button::Right));
    assert_eq!(badge.due(10_000), None, "neither was held back");
}

/// A board with four keys never enters any of this.
#[test]
fn a_four_key_board_confirms_at_once() {
    let mut badge = Badge::default();
    let back = back_stands_on(Board::X4.tokens.row);

    assert_eq!(
        badge.pressed(Button::Confirm, 0, back),
        Some(Button::Confirm),
        "the X4 has a Back key, so Confirm means Confirm immediately"
    );
    assert_eq!(badge.due(10_000), None, "nothing was ever held back");
}

// -- the whole path --------------------------------------------------------
//
// The three tests above are the state machine on its own. These drive it
// through the simulator's own [`Keypad`] into a real screen stack, which is
// what the arrangement is actually for: what a person sees when they press the
// key on the badge.

/// One frame: the loop's own frame opening, then the presses that arrived.
///
/// [`open_frame`] rather than a copy of what it does — a hand-written version
/// of the ordering is a version that can drift from the loop's without
/// anything noticing, and the ordering is the whole subject here.
fn frame(session: &Session, keypad: &mut Keypad, app: &mut App, now: u32, press: Option<Button>) {
    let board = session.board();
    open_frame(keypad, session.backend(), now);
    if let Some(button) = press {
        keypad.down(session.backend(), button, now, board);
        keypad.up(session.backend(), button);
    }
    app.tick();
    app.render_if_dirty();
}

#[test]
fn on_a_badge_one_press_opens_a_screen_only_once_its_window_has_shut() {
    let _guard = serial();
    let session = Session::new(Panel::of(three_key_badge()));
    let mut keypad = Keypad::new(Box::new(Badge::default()));
    let mut app = App::new(Menu::new());
    app.render();
    assert_eq!(app.depth(), 1, "the menu, and nothing on top of it");

    frame(&session, &mut keypad, &mut app, 0, Some(Button::Confirm));
    assert_eq!(
        app.depth(),
        1,
        "the press has not been delivered yet — opening here is the fault the \
         whole arrangement exists to avoid, because the second press would \
         then arrive on a screen that was not there when it started"
    );

    frame(&session, &mut keypad, &mut app, DOUBLE_PRESS_MS - 1, None);
    assert_eq!(app.depth(), 1, "still inside the window");

    frame(&session, &mut keypad, &mut app, DOUBLE_PRESS_MS, None);
    assert_eq!(
        app.depth(),
        2,
        "the window shut with no second press, so the select was delivered"
    );
}

/// The window's edge, from both sides at once.
///
/// [`Doubles`] takes a second press *strictly inside* the window and
/// [`Badge::due`] fires *at* its end. If both were inclusive the millisecond
/// they share would be read twice — delivered as a select and then as a Back,
/// which opens a screen and immediately leaves it.
#[test]
fn the_windows_two_ends_meet_exactly_once() {
    let mut badge = Badge::default();
    let back = back_stands_on(NO_BACK_KEY);

    badge.pressed(Button::Confirm, 0, back);
    assert_eq!(
        badge.pressed(Button::Confirm, DOUBLE_PRESS_MS - 1, back),
        Some(Button::Back),
        "one millisecond inside the window is still a double"
    );

    let mut badge = Badge::default();
    badge.pressed(Button::Confirm, 0, back);
    assert_eq!(
        badge.due(DOUBLE_PRESS_MS),
        Some(Button::Confirm),
        "and the window's own end belongs to the select"
    );

    let mut badge = Badge::default();
    badge.pressed(Button::Confirm, 0, back);
    assert_eq!(
        badge.pressed(Button::Confirm, DOUBLE_PRESS_MS, back),
        None,
        "a press landing exactly on the end is a fresh single on the stand-in \
         key, so it is held back in its turn rather than read as a Back"
    );
    assert_eq!(
        badge.due(DOUBLE_PRESS_MS),
        Some(Button::Confirm),
        "and the first press, whose window shut on that same millisecond, is \
         still owed — a press a person made must not be dropped because \
         another arrived before anybody asked for it"
    );
}

/// Offering two presses without asking for anything due between them must not
/// lose the first. The loop always asks, so this is about not resting on that.
#[test]
fn a_press_is_never_overwritten_by_the_one_after_it() {
    let mut badge = Badge::default();
    let back = back_stands_on(NO_BACK_KEY);

    assert_eq!(badge.pressed(Button::Confirm, 0, back), None);
    assert_eq!(
        badge.pressed(Button::Confirm, DOUBLE_PRESS_MS + 2, back),
        None
    );

    assert_eq!(
        badge.due(DOUBLE_PRESS_MS + 2),
        Some(Button::Confirm),
        "the first press, ripe and never collected"
    );
    assert_eq!(
        badge.due(2 * DOUBLE_PRESS_MS + 2),
        Some(Button::Confirm),
        "and the second in its own turn — two presses, two selects"
    );
}

/// A press arriving just past the window must not take the select with it.
///
/// The loop reads events *after* delivering anything due, so a press one
/// millisecond too late finds the select already gone. Delivered the other way
/// round it overwrites a select that was ripe, and that press is lost outright
/// — two deliberate presses, one screen, hundreds of milliseconds late on a
/// panel whose frame is a 900ms refresh.
#[test]
fn a_press_just_past_the_window_does_not_swallow_the_select() {
    let _guard = serial();
    let session = Session::new(Panel::of(three_key_badge()));
    let mut keypad = Keypad::new(Box::new(Badge::default()));
    let mut app = App::new(Menu::new());
    app.render();

    frame(&session, &mut keypad, &mut app, 0, Some(Button::Confirm));
    assert_eq!(app.depth(), 1, "held back, as it should be");

    // Two milliseconds past the window: too late to be a double.
    frame(
        &session,
        &mut keypad,
        &mut app,
        DOUBLE_PRESS_MS + 2,
        Some(Button::Confirm),
    );
    assert_eq!(
        app.depth(),
        2,
        "the first press acted — it was ripe before the second arrived, and a \
         second press cannot cancel a select that had already come due"
    );

    // What the second press then does is the example screen's business; that
    // it is still owed at all is `a_press_is_never_overwritten_by_the_one_
    // after_it`, which asks the state machine directly rather than through a
    // screen that may or may not push anything.
}

#[test]
fn on_a_badge_two_presses_go_back() {
    let _guard = serial();
    let session = Session::new(Panel::of(three_key_badge()));
    let mut keypad = Keypad::new(Box::new(Badge::default()));
    let mut app = App::new(Menu::new());
    app.push(Menu::new());
    app.render();
    assert_eq!(app.depth(), 2, "something to go back from");

    frame(&session, &mut keypad, &mut app, 0, Some(Button::Confirm));
    frame(&session, &mut keypad, &mut app, 100, Some(Button::Confirm));

    assert_eq!(app.depth(), 1, "two presses inside the window went back");

    // And the select they superseded does not arrive after the fact.
    for now in [DOUBLE_PRESS_MS, DOUBLE_PRESS_MS + 100, 10_000] {
        frame(&session, &mut keypad, &mut app, now, None);
    }
    assert_eq!(
        app.depth(),
        1,
        "the first press was cancelled by the second, not queued behind it"
    );
}

#[test]
fn on_a_reader_the_select_is_immediate() {
    let _guard = serial();
    let session = Session::new(Panel::of(Board::X4));
    let mut keypad = Keypad::new(Box::new(Badge::default()));
    let mut app = App::new(Menu::new());
    app.render();

    frame(&session, &mut keypad, &mut app, 0, Some(Button::Confirm));

    assert_eq!(
        app.depth(),
        2,
        "a board with a Back key of its own waits for nothing"
    );
}
