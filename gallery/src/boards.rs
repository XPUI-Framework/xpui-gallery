//! The eight panels this gallery claims to fit.
//!
//! There is no such thing as "every board" below this file. `xpui-boards-core`
//! is the vocabulary and describes no device; each vendor crate describes its
//! own and knows nothing of the other two. Assembling a device list is the
//! application's job, and this is the application.
//!
//! A firmware names the one board it flashes and needs none of this. A project
//! shipping against two vendors writes its own eight-line version.

use xpui_boards_core::Board;

/// Every board the gallery is built for, in vendor order.
///
/// The order is what `B` walks in the simulator, so it is the order the
/// screenshots are taken in and the order the slugs are listed in on a bad
/// `--board`. Grouped by vendor rather than by size, because that is how
/// somebody looking for their device thinks about it.
pub const ALL: [Board; 8] = [
    xpui_boards_xteink::X3,
    xpui_boards_xteink::X4,
    xpui_boards_xteink::X4_CLASSIC,
    xpui_boards_xteink::X4_PRO,
    xpui_boards_seeed::STICKY,
    xpui_boards_pimoroni::BADGER_2040,
    xpui_boards_pimoroni::TUFTY_2040,
    xpui_boards_pimoroni::INKY_FRAME,
];

/// Every vendor's boards are here, and nothing else is.
///
/// `ALL` is written out by hand, so a vendor could gain a board and this list
/// not notice. A `const` assertion rather than a test because it fires at
/// compile time: a board added to a vendor crate stops this building until
/// somebody decides whether the gallery ships it.
const _: () = assert!(
    ALL.len()
        == xpui_boards_xteink::ALL.len()
            + xpui_boards_seeed::ALL.len()
            + xpui_boards_pimoroni::ALL.len(),
    "a vendor gained or lost a board and this list did not follow"
);

/// Looks a board up by its short name, for `--board`.
///
/// Each vendor is asked in turn and answers for its own boards only, which is
/// what keeps the aliases — `badger` and `tufty`, because that is what people
/// say — with the crate that owns those devices rather than in a table here
/// that would fall behind it.
pub fn from_slug(slug: &str) -> Option<Board> {
    xpui_boards_xteink::from_slug(slug)
        .or_else(|| xpui_boards_seeed::from_slug(slug))
        .or_else(|| xpui_boards_pimoroni::from_slug(slug))
}

/// The slugs `--board` accepts, for the message printed when it does not.
pub fn slugs() -> impl Iterator<Item = &'static str> {
    ALL.iter().map(|board| board.slug)
}
