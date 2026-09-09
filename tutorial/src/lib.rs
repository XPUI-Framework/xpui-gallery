//! The screen the tutorial builds, finished.
//!
//! [`docs/tutorial.md`](https://github.com/XPUI-Framework/xpui-framework/blob/main/docs/tutorial.md) walks
//! from an empty file to this, one step at a time. Every snippet there is a
//! doctest but one — proven by `xpui` itself, which needs no backend to run a
//! screen. The exception is the block that opens a window: `xpui` cannot see a
//! simulator, so those three calls are checked against this crate's `main`
//! instead, which the gate compiles.
//!
//! Screenshot-tested like any other screen, with a `main` that opens it in a
//! window.

use xpui::screen::Screen;
use xpui::{
    Hint, List, ListRow, Modal, NavigationScreen, Point, Scrim, Stepper, Text, Toggle, View,
    finish_screen, hstack, vstack,
};

/// The preset lengths the picker offers.
const PRESETS: [&str; 4] = ["5 minutes", "15 minutes", "30 minutes", "1 hour"];
/// The same presets, in minutes.
const PRESET_MINUTES: [i32; 4] = [5, 15, 30, 60];

/// Everything this screen can be told.
///
/// One enum, matched exhaustively in [`Screen::update`]. Adding a control
/// means adding a variant, and the compiler then insists you handle it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Message {
    /// An absolute value, from dragging the stepper's track.
    SetMinutes(i32),
    /// A nudge of -1 or +1, from its end glyphs or the Left/Right keys.
    StepMinutes(i32),
    /// The state the toggle is moving *to*, so this screen never writes
    /// `!self.something` — the bug that makes a toggle flip twice per press.
    SetSleepOnClose(bool),
    OpenPresets,
    ChoosePreset(usize),
    DismissPresets,
    Done,
}

/// A sleep timer: how long before the device sleeps, and what to do on close.
pub struct SleepTimer {
    minutes: i32,
    sleep_on_close: bool,
    picking: bool,
    /// The minutes, already formatted.
    ///
    /// `body()` runs on every paint *and* on every frame carrying input, so a
    /// `format!` in there allocates several times a second and pulls
    /// `core::fmt` into the binary. Formatting in `update` costs one
    /// allocation per actual change.
    label: String,
}

impl Default for SleepTimer {
    fn default() -> Self {
        SleepTimer::new()
    }
}

impl SleepTimer {
    pub fn new() -> Self {
        SleepTimer {
            minutes: 15,
            sleep_on_close: true,
            picking: false,
            label: label_for(15),
        }
    }

    pub fn minutes(&self) -> i32 {
        self.minutes
    }

    pub fn sleeps_on_close(&self) -> bool {
        self.sleep_on_close
    }

    pub fn is_picking(&self) -> bool {
        self.picking
    }
}

fn label_for(minutes: i32) -> String {
    format!("{minutes} min")
}

impl Screen for SleepTimer {
    type Message = Message;

    fn body(&self) -> impl View<Self::Message> {
        NavigationScreen::new(vstack![14;
            hstack![8;
                Text::new("Sleep after"),
                xpui::Spacer::new(),
                Text::new(&self.label).bold(),
            ],

            // One focus stop, three touch targets: the two glyphs nudge, the
            // track sets an absolute value.
            Stepper::ranged(self.minutes, 120)
                .on_change(Message::SetMinutes)
                .on_step(Message::StepMinutes),

            List::new().push(
                ListRow::new("Presets")
                    .value(PRESETS[nearest_preset(self.minutes)])
                    .on_tap(Message::OpenPresets),
            ),

            Toggle::new("Sleep when closed", self.sleep_on_close, "Yes", "No")
                .on_change(Message::SetSleepOnClose),
        ])
        .title("Sleep timer")
        .hints(Hint::Standard, Hint::text("Save"), Hint::None, Hint::None)
        // A dialog captures input: nothing behind it is reachable while it is
        // in the tree. This screen decides only whether it is there.
        .overlay_if(
            self.picking,
            Modal::picker("Sleep after", PRESETS)
                .selected(nearest_preset(self.minutes))
                .on_select(Message::ChoosePreset)
                .scrim(Scrim::Dim),
        )
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Message::SetMinutes(value) => self.minutes = value.clamp(1, 120),
            Message::StepMinutes(delta) => self.minutes = (self.minutes + delta).clamp(1, 120),
            Message::SetSleepOnClose(next) => self.sleep_on_close = next,
            Message::OpenPresets => self.picking = true,
            Message::ChoosePreset(index) => {
                self.minutes = PRESET_MINUTES[index.min(PRESET_MINUTES.len() - 1)];
                self.picking = false;
            }
            Message::DismissPresets => self.picking = false,
            Message::Done => finish_screen(),
        }
        self.label = label_for(self.minutes);
    }

    /// A touch that no control claimed. The dialog uses it to close when the
    /// dimmed area around it is tapped.
    fn on_background_tap(&self, _at: Point) -> Option<Self::Message> {
        self.picking.then_some(Message::DismissPresets)
    }

    fn title(&self) -> Option<&'static str> {
        Some("Sleep timer")
    }
}

/// Which preset the current value is closest to, so the row and the dialog
/// agree about what is selected.
fn nearest_preset(minutes: i32) -> usize {
    PRESET_MINUTES
        .iter()
        .enumerate()
        .min_by_key(|(_, preset)| (**preset - minutes).abs())
        .map(|(index, _)| index)
        .unwrap_or(0)
}
