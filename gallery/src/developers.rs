//! Diagnostics that are not user features.
//!
//! Ported from the firmware this framework was built inside, where it showed
//! live heap figures. There is no firmware here, so the numbers come from a
//! stand-in that moves between readings.
//!
//! It doubles as the framework's proving ground: the sections below exercise
//! the list, the dialog, the slider and focus navigation on one screen, so a
//! change to any of them can be seen on a device rather than argued about.

use alloc::string::String;
use alloc::vec::Vec;

use xpui::{
    Button, List, ListRow, Modal, NavigationScreen, Screen, ScrollView, Section, Slider, Stepper,
    Theme, ThemeMetric, View, vstack,
};

use crate::heap::Heap;
use crate::units::Units;

/// Labels for the scale picker and the sections below.
///
/// Plain constants rather than translated keys: a firmware would reach these
/// through its own i18n, and standing one up for a single example would be
/// more machinery than the example.
const UNIT_LABELS: [&str; 2] = ["Bytes", "Kilobytes"];
const SCREEN_TITLE: &str = "Developers";
const MEMORY_TITLE: &str = "Memory";
const TOTAL_HEAP: &str = "Total";
const USED_HEAP: &str = "Used";
const FREE_HEAP: &str = "Free";
const LARGEST_BLOCK: &str = "Largest block";
const MIN_FREE_HEAP: &str = "Least ever free";
const BENCH_TITLE: &str = "Widget bench";
const SCROLL_TITLE: &str = "Scrolling";

/// Rows in the scrolling section. Deliberately more than fit on any of the
/// panels, so the scroll indicator and the keep-the-selection-visible logic
/// both have something to do.
const SCROLL_ROWS: usize = 14;

/// Everything this screen can be told.
#[derive(Clone, Copy)]
pub enum Msg {
    /// Any memory row: opens the scale picker.
    PickUnits,
    /// A scale chosen from the picker.
    ChoseUnits(usize),
    /// The picker dismissed without choosing.
    DismissPicker,
    /// The slider dragged or tapped: carries the new value.
    Brightness(i32),
    /// The stepper nudged by a button: carries ±1.
    StepBrightness(i32),
    /// A row in the scrolling section, by index.
    ScrollRow(usize),
    /// A bench row. Carries its own index so a tap is visible, and crucially
    /// is *not* the picker's message.
    BenchRow(usize),
}

#[derive(Default)]
pub struct DevelopersScreen {
    units: Units,
    /// Stands in for the firmware's heap figures. It moves between readings on
    /// purpose: a static number would let a repaint test pass without anything
    /// having been redrawn.
    heap: Heap,
    /// Whether the scale picker is open. The screen owns this; the framework
    /// only takes input over once the dialog is in the tree.
    picking: bool,
    /// Drives both the slider and the stepper, so the two stay in step and a
    /// touch drag and a button press can be compared against each other.
    level: i32,
    /// Last row tapped in the scrolling section, echoed as its value so a tap
    /// is visible without a serial log.
    tapped_row: Option<usize>,
    /// Row labels, built once. `body` runs on every paint and every frame
    /// carrying input, so formatting fourteen of them there would allocate
    /// fourteen strings several times a second.
    row_labels: Vec<String>,
}

impl DevelopersScreen {
    pub fn new() -> Self {
        DevelopersScreen {
            level: 50,
            row_labels: (0..SCROLL_ROWS)
                .map(|index| alloc::format!("Row {}", index + 1))
                .collect(),
            ..DevelopersScreen::default()
        }
    }

    /// Three rows carrying a right-hand value. Tapping any of them opens the
    /// picker, since the scale applies to all three.
    fn memory_usage_view(&self) -> List<Msg> {
        let heap = self.heap.reading();
        let row = |label: &str, bytes: i32| {
            ListRow::new(label)
                .value(self.units.format(bytes))
                .on_tap(Msg::PickUnits)
        };

        List::new()
            // Total first, then what is gone: a free figure alone says nothing
            // about how much room there ever was.
            .push(row(TOTAL_HEAP, heap.total))
            .push(row(USED_HEAP, heap.total - heap.free))
            .push(row(FREE_HEAP, heap.free))
            .push(row(LARGEST_BLOCK, heap.largest_block))
            .push(row(MIN_FREE_HEAP, heap.min_free))
    }

    /// Rows carrying a subtitle. The theme sizes a two-line row differently
    /// from a one-line one, so a section of them is the only way to see that
    /// the list is asking for the right height.
    fn subtitle_rows(&self) -> List<Msg> {
        List::new()
            .push(
                ListRow::new("Touch drag")
                    .subtitle("Slide the control below with a finger")
                    .on_tap(Msg::BenchRow(0)),
            )
            .push(
                ListRow::new("Button step")
                    .subtitle("Left and right nudge the focused row")
                    .on_tap(Msg::BenchRow(1)),
            )
            .push(
                ListRow::new("Long subtitle")
                    .subtitle("A line long enough to test what the row does when the text runs past the width it was given")
                    .on_tap(Msg::BenchRow(4)),
            )
            .push(
                ListRow::new(
                    "A title with no subtitle, long enough to run past the width of the panel and need a second line",
                )
                .on_tap(Msg::BenchRow(3)),
            )
    }

    /// More rows than fit, so the list has to scroll and keep the focused row
    /// on screen — by swipe on a touch panel, by Up/Down on a button one.
    fn scrolling_rows(&self) -> List<Msg> {
        let mut list = List::new();
        for (index, label) in self.row_labels.iter().enumerate() {
            let mut row = ListRow::new(label.as_str()).on_tap(Msg::ScrollRow(index));
            if self.tapped_row == Some(index) {
                row = row.value("tapped");
            }
            list = list.push(row);
        }
        list
    }

    /// The shared brightness value, for a test to read back.
    pub fn level(&self) -> i32 {
        self.level
    }

    /// The scale every memory figure is written in.
    pub fn units(&self) -> Units {
        self.units
    }

    /// Whether the scale picker is up.
    pub fn is_picking(&self) -> bool {
        self.picking
    }

    /// Which label matches the scale in use, so the picker opens on it.
    fn units_index(&self) -> usize {
        match self.units {
            Units::Bytes => 0,
            Units::Kilobytes => 1,
        }
    }
}

impl Screen for DevelopersScreen {
    type Message = Msg;

    fn body(&self) -> impl View<Msg> {
        let spacing = Theme::metric(ThemeMetric::VerticalSpacing);

        NavigationScreen::new(ScrollView::new(vstack![spacing;
                Section::new(MEMORY_TITLE, self.memory_usage_view()),
                Section::new(BENCH_TITLE, self.subtitle_rows()),
                // Same value, two controls: a plain track and a stepper with
                // a glyph at each end. Both are focus stops of their own, so
                // this row shows the difference between them rather than
                // working around either.
                Slider::new(self.level, 100).on_change(Msg::Brightness),
                Stepper::new(self.level)
                    .on_change(Msg::Brightness)
                    .on_step(Msg::StepBrightness),
                Section::new(SCROLL_TITLE, self.scrolling_rows()),
        ]))
        // An overlay, not content: the scroll view must not clip it and it must
        // not scroll away with the rows underneath.
        .overlay_if(
            self.picking,
            Modal::picker(MEMORY_TITLE, UNIT_LABELS)
                .selected(self.units_index())
                .on_select(Msg::ChoseUnits),
        )
    }

    fn title(&self) -> Option<&'static str> {
        Some(SCREEN_TITLE)
    }

    /// Back closes the picker rather than the screen. The runtime does not
    /// assume this, so a screen showing a dialog says so.
    fn on_key(&self, key: Button) -> Option<Msg> {
        match key {
            Button::Back if self.picking => Some(Msg::DismissPicker),
            _ => None,
        }
    }

    fn update(&mut self, message: Msg) {
        match message {
            Msg::PickUnits => self.picking = true,
            Msg::ChoseUnits(index) => {
                self.units = if index == 0 {
                    Units::Bytes
                } else {
                    Units::Kilobytes
                };
                self.picking = false;
            }
            Msg::DismissPicker => self.picking = false,
            Msg::Brightness(value) => self.level = value.clamp(0, 100),
            Msg::StepBrightness(delta) => self.level = (self.level + delta).clamp(0, 100),
            Msg::ScrollRow(index) => self.tapped_row = Some(index),
            Msg::BenchRow(_) => {}
        }
    }
}
