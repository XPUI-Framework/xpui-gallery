//! Example screens built on [`xpui`].
//!
//! A menu of examples, each one opening a screen that demonstrates one part of
//! the framework. It is deliberately shaped like a real e-reader's settings
//! app rather than like a widget catalogue, because that is the thing the
//! framework is for and the thing worth knowing works.
//!
//! The screens live in a library rather than in `main.rs` so the tests can
//! drive them without opening a window — and so a firmware can depend on them
//! without dragging a window in.
//!
//! `no_std` on a device, `std` on a desktop: these exact screens are what the
//! RP2040 binaries in [`examples/rp2040`](../../rp2040/) flash to a Badger
//! 2040 and a Tufty 2040. One source, two targets, differing only in a
//! [`Board`] and an entry point.

#![cfg_attr(target_os = "none", no_std)]

extern crate alloc;

pub mod boards;
pub mod chord;
pub mod developers;
pub mod fonts;
pub mod heap;
pub mod menu;
pub mod screens;
pub mod typeface;
mod units;

use xpui_boards_core::Board;
use xpui_chrome::{Labels, Metrics};
use xpui_eg::{Backend, DrawTarget, Fonts, Palette};

pub use developers::DevelopersScreen;
pub use menu::Menu;
pub use typeface::Typefaces;
pub use units::Units;

/// The measurements a board's panel gets.
///
/// Chrome derives these from a size and a scale and knows nothing about
/// boards; deciding which board gets which is the application's call, and this
/// is the one place the gallery makes it. [`wire`] paints through it and
/// `tests/physical.rs` measures through it, so a screenshot and a millimetre
/// figure cannot disagree about what was on the glass.
///
/// `!board.touch` is the hint band: a board driven by a finger has no keys to
/// label, so the band would be a strip of words naming keys nobody has.
pub fn metrics_for(board: Board) -> Metrics {
    Metrics::for_device(
        board.width,
        board.height,
        board.ui_scale_percent,
        !board.touch,
    )
}

/// A backend wired for a board, as a firmware would wire one.
///
/// The framework does not know what a board is any more: `Metrics` comes from
/// the panel's size and scale, `Labels` from its size, `KeyRow` and the
/// Left/Right pair from the hardware. Composing those is the application's
/// job, and this is the application — so this is what a firmware's frame loop
/// looks like, and what the conformance suite drives.
pub fn wire<D>(display: D, board: Board, palette: Palette<D::Color>) -> Backend<D>
where
    D: DrawTarget,
{
    let metrics = metrics_for(board);
    Backend::new(display, palette)
        .with_metrics(metrics)
        .with_labels(Labels::for_panel(board.width, board.height))
        .with_keys(board.keys)
        .with_left_right_keys(board.has_left_right_keys())
        .with_fonts(Fonts::for_metrics(&metrics))
}
