//! Two presses of one key, meaning something a second key would have meant.
//!
//! A board with three keys along its bottom edge and no spare has nowhere to
//! put Back. Pressing the first key twice in quick succession stands in for it
//! — the same trick a mouse plays with its one button.
//!
//! **No board in `Board::ALL` is arranged this way.** The Badger and the Tufty
//! each have three keys *and* an up/down pair, so they spend the first on Back
//! and never enter this. It is kept because that shape is real, and because
//! reading two presses as one meaning is the firmware's job wherever it
//! happens.
//!
//! Driven by `(key, timestamp)` so it can be tested without a window.
//!
//! This lives in the example, not the simulator. A simulator stands in for the
//! hardware, and hardware sends raw presses — what two of them close together
//! *mean* is the firmware's decision, and here the example is the firmware.
//!
//! # What it costs
//!
//! A key that might yet turn out to be half of a double press cannot act
//! until it is certain it was not, so **every select on a three-key board waits
//! out [`DOUBLE_PRESS_MS`]**. That is the price of the arrangement, and it is
//! not free:
//!
//! On a panel that takes most of a second to refresh the wait is lost in it; on
//! an immediate LCD it is a third of a second of nothing.
//!
//! Only the stand-in key pays it. Every other key on the row acts on the
//! frame it was pressed, and a board with four keys has a Back of its own and
//! never enters this at all.
//!
//! Before choosing this for a board, check whether it has a key to spare. A
//! delay on the one action a person takes most often is a poor trade for a
//! button that was already there.

use xpui::Button;
use xpui_chrome::RowKey;

/// How long a second press has to arrive to count as part of the first.
///
/// Long enough not to need a deliberate double-tap, short enough that two
/// separate confirmations are not read as one Back. The firmware uses four
/// hundred milliseconds to separate a click from a hold, and this sits under
/// that so the two do not fight.
pub const DOUBLE_PRESS_MS: u32 = 350;

/// What a press turned out to mean.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Chord {
    /// The key on its own.
    Single(Button),
    /// Two presses of the key that stands in for Back.
    Back,
}

/// Watches one key for a second press.
#[derive(Copy, Clone, Debug, Default)]
pub struct Doubles {
    /// The key a double press means Back on, and when it was last pressed.
    last: Option<(Button, u32)>,
}

impl Doubles {
    /// What pressing `button` at `now` means.
    ///
    /// `stands_in_for_back` is the key that carries Back on this board — the
    /// first of a three-key row. Every other key passes straight through, so a
    /// board with a Back key of its own is unaffected.
    ///
    /// A press that completes a double clears the record, so three presses read
    /// as a Back and then a fresh single rather than as two overlapping
    /// doubles.
    pub fn press(&mut self, button: Button, now: u32, stands_in_for_back: Option<Button>) -> Chord {
        if Some(button) != stands_in_for_back {
            self.last = None;
            return Chord::Single(button);
        }

        let doubled = match self.last {
            // Strictly inside. `Badge::due` fires *at* the window's end, and
            // if both were inclusive a press landing on that exact
            // millisecond would be delivered as a select and read as a Back —
            // opening a screen and immediately leaving it. Between them the
            // two cover every instant exactly once.
            Some((previous, at)) => previous == button && now.saturating_sub(at) < DOUBLE_PRESS_MS,
            None => false,
        };

        if doubled {
            self.last = None;
            return Chord::Back;
        }

        self.last = Some((button, now));
        Chord::Single(button)
    }
}

/// The key that carries Back on a board whose bottom row has no Back key.
///
/// Asked of the row itself rather than of how long it is. Those were the same
/// question while every three-key row spent its keys on Confirm, Previous and
/// Next — but a board with three keys along the bottom *and* an up/down pair
/// elsewhere has one to spare, gives it to Back, and needs no stand-in at all.
/// Counting keys would still charge it the double-press delay for a key it
/// has.
pub fn back_stands_on(row: &[RowKey]) -> Option<Button> {
    // An empty row is a board with no keys along the bottom at all: Back comes
    // from its touchscreen, and there is no key here to borrow. "Has no Back
    // key" is true of it and means the opposite of what it means for a badge,
    // which is the one place asking the row rather than its length needs help.
    (!row.is_empty() && !row.contains(&RowKey::Back)).then_some(Button::Confirm)
}

/// The whole arrangement: recognise the double press, and hold the select back
/// far enough to know there was not one.
///
/// [`Doubles`] can only answer once both presses have arrived. That is too
/// late on its own — by then the first press has already opened something, and
/// going back from it is not the same as never having gone. So the stand-in
/// key is **swallowed on the way down** and re-issued when its window closes
/// with no second press behind it. See the module docs for what that costs.
///
/// Portable on purpose. The simulator is where it is exercised, but the board
/// it is for is a real one, and its firmware has the same three keys and the
/// same missing fourth.
#[derive(Copy, Clone, Debug, Default)]
pub struct Badge {
    doubles: Doubles,
    /// A press held back, and the reading of the clock it is released at.
    pending: Option<(Button, u32)>,
    /// A press whose window shut before anybody asked for it.
    ///
    /// The loop asks for anything due at the top of each frame, so this is
    /// normally empty. It exists because "normally" is not a guarantee a state
    /// machine should rest on: a caller that offers two presses without asking
    /// between them would otherwise have the first silently overwritten, and a
    /// press that a person made and the device dropped is the worst outcome
    /// available here.
    ripe: Option<Button>,
}

impl Badge {
    /// What pressing `button` at `now` should deliver, or `None` for nothing
    /// yet.
    ///
    /// `stands_in_for_back` is [`back_stands_on`] for the board the press came
    /// from. `None` — a board with a Back key of its own — passes everything
    /// through on the frame it arrived, so nothing here delays a device that
    /// does not need it.
    pub fn pressed(
        &mut self,
        button: Button,
        now: u32,
        stands_in_for_back: Option<Button>,
    ) -> Option<Button> {
        // A press whose window has already shut is owed to whoever made it,
        // whether or not anybody has asked yet. Set aside before this press
        // can overwrite it.
        if let Some((held, at)) = self.pending
            && now >= at
        {
            self.pending = None;
            self.ripe = Some(held);
        }

        match self.doubles.press(button, now, stands_in_for_back) {
            // The second press. Whatever the first one was holding is what this
            // one supersedes, so it is dropped rather than delivered late.
            Chord::Back => {
                self.pending = None;
                Some(Button::Back)
            }
            // The first press of the key that carries Back: it cannot be
            // delivered until the window closes on it.
            Chord::Single(held) if Some(held) == stands_in_for_back => {
                self.pending = Some((held, now.saturating_add(DOUBLE_PRESS_MS)));
                None
            }
            Chord::Single(other) => Some(other),
        }
    }

    /// The held press, once its window has closed.
    ///
    /// Asked every frame. Returns at most once per press — a press delivered
    /// twice would open a screen and then open whatever the first row of it
    /// was.
    pub fn due(&mut self, now: u32) -> Option<Button> {
        if let Some(button) = self.ripe.take() {
            return Some(button);
        }
        let (button, at) = self.pending?;
        if now < at {
            return None;
        }
        self.pending = None;
        Some(button)
    }
}

// The simulator links SDL, so it is a desktop dependency and the device build
// of these screens does not have it. The state machine above is the part worth
// sharing with a firmware; this is only the plug.
#[cfg(not(target_os = "none"))]
impl xpui_simulator::Keys for Badge {
    fn translate(&mut self, press: xpui_simulator::Press) -> Option<Button> {
        self.pressed(
            press.button,
            press.now,
            back_stands_on(press.board.tokens.row),
        )
    }

    fn due(&mut self, now: u32) -> Option<Button> {
        Badge::due(self, now)
    }

    fn reset(&mut self) {
        *self = Badge::default();
    }
}
