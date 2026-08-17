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
//! [`Board`](xpui_boards::Board) and an entry point.

#![cfg_attr(target_os = "none", no_std)]

extern crate alloc;

pub mod chord;
pub mod developers;
pub mod fonts;
pub mod heap;
pub mod menu;
pub mod screens;
pub mod typeface;
mod units;

pub use developers::DevelopersScreen;
pub use menu::Menu;
pub use typeface::Typefaces;
pub use units::Units;
