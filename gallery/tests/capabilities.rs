//! What a backend wired for a board answers about that board.
//!
//! The backend stores a `bool` and cannot derive it — see `xpui-backends`'s
//! `embedded_graphics/tests/capabilities.rs`. Deriving it is the
//! application's job, so this is where the wiring is checked: that
//! [`gallery::wire`] asks the board rather than guessing.
//!
//! A wrong answer is a control that cannot be changed at all, or one that
//! promises keys the board does not carry — and neither shows up in a
//! framebuffer.

use gallery::wire;
use xpui::host::InputSource;
use xpui_eg::Palette;
use xpui_screenshot::Framebuffer as TestDisplay;

fn display() -> TestDisplay {
    TestDisplay::new(200, 120)
}

/// The wiring reports what the board carries, board by board.
///
/// Walking `gallery::boards::ALL` rather than a chosen one: this is the per-board
/// regression that is easiest to ship and hardest to see, because the suite is
/// green and the board you looked at is right.
///
/// Deliberately compared against the board's own answer rather than a second
/// hand-written table. What is being checked here is the *wiring* — that the
/// application asks the board — and `xpui-boards` is where the answers
/// themselves are pinned per board and argued for.
#[test]
fn a_backend_wired_for_a_board_answers_for_that_board() {
    let mut with_pair = 0;
    let mut without = 0;

    for board in gallery::boards::ALL {
        let backend = wire(display(), board, Palette::INK_IS_ON);
        assert_eq!(
            backend.has_left_right_keys(),
            board.has_left_right_keys(),
            "{} answers something its board does not",
            board.name
        );
        if board.has_left_right_keys() {
            with_pair += 1;
        } else {
            without += 1;
        }
    }

    // Wiring hard-coded to either constant would satisfy the loop above on
    // whichever boards happened to agree with it. Both answers have to occur,
    // or this test is only checking one of them.
    //
    // How many boards give each answer is `xpui-boards`' to assert, and it
    // does. Repeating the count here would put the census in two crates, which
    // is the drift this method was added to end.
    assert!(
        with_pair > 0 && without > 0,
        "every board now answers the same way, so this no longer proves the \
         wiring asks rather than guesses"
    );
}
