//! What every reference-picture test shares: a host on the X3, a screen around
//! a view, a crop, and a way to report every picture that failed at once.
//!
//! Each `reference_*.rs` file is its own test binary with one test, because
//! `xpui::host::install` writes a process-wide static.

// Each test file uses part of this, and the rest reads as dead to it.
#![allow(dead_code)]

use gallery::{metrics_for, wire};
use xpui::{App, NavigationScreen, Rect, Screen, Theme, ThemeMetric, View};
use xpui_boards_core::Board;
use xpui_eg::{Backend, Palette};
use xpui_screenshot::{Framebuffer, check_screenshot};

/// One board for every picture, so the pages share a scale.
pub const BOARD: Board = xpui_boards_xteink::X3;

/// A fresh host the size of the X3, installed.
pub fn install() -> &'static Backend<Framebuffer> {
    let backend = wire(
        Framebuffer::new(BOARD.width, BOARD.height),
        BOARD,
        Palette::INK_IS_ON,
    )
    .leaked();
    // Safety: every file using this holds one test, so nothing else installs
    // a host or renders while this one is in use.
    unsafe { xpui::host::install(backend) };
    backend
}

/// A screen whose content is one view, under a header reading "Settings".
pub struct Body<M: 'static>(pub fn() -> Box<dyn View<M>>);

impl<M: Clone + 'static> Screen for Body<M> {
    type Message = M;

    fn body(&self) -> impl View<M> {
        NavigationScreen::new((self.0)())
    }

    fn update(&mut self, _message: M) {}

    fn title(&self) -> Option<&'static str> {
        Some("Settings")
    }
}

/// A screen that builds its own root, titled `.0`.
pub struct Root<M: 'static>(pub &'static str, pub fn() -> NavigationScreen<M>);

impl<M: Clone + 'static> Screen for Root<M> {
    type Message = M;

    fn body(&self) -> impl View<M> {
        (self.1)()
    }

    fn update(&mut self, _message: M) {}

    fn title(&self) -> Option<&'static str> {
        Some(self.0)
    }
}

/// The part of `panel` inside `rect`, clamped to the panel.
pub fn crop(panel: &Framebuffer, rect: Rect) -> Framebuffer {
    let x = rect.x().max(0);
    let y = rect.y().max(0);
    let width = rect.width().min(BOARD.width - x);
    let height = rect.height().min(BOARD.height - y);
    let mut out = Framebuffer::new(width, height);
    for row in 0..height {
        for column in 0..width {
            out.set(column, row, panel.get(x + column, y + row));
        }
    }
    out
}

/// `frame` against `tests/screenshots/reference/<name>.png`.
pub fn check(name: &str, frame: &Framebuffer) -> Result<(), String> {
    check_screenshot(&format!("reference/{name}"), frame)
}

/// The whole panel as `backend` last painted it.
pub fn whole(backend: &'static Backend<Framebuffer>, name: &str) -> Result<(), String> {
    backend.with_display(|panel| check(name, panel))
}

/// The hint bar along the bottom of the panel.
pub fn hint_bar(backend: &'static Backend<Framebuffer>, name: &str) -> Result<(), String> {
    let height = metrics_for(BOARD).button_hints_height;
    let bar = Rect::new(0, BOARD.height - height, BOARD.width, height);
    backend.with_display(|panel| check(name, &crop(panel, bar)))
}

/// Paints `content` as a screen's body, and keeps the part of the panel it
/// covers, with the theme's small spacing around it.
pub fn component<M: Clone + 'static>(
    name: &str,
    content: fn() -> Box<dyn View<M>>,
) -> Result<(), String> {
    let backend = install();
    App::new(Body(content)).render();

    let band = Theme::content_area();
    let mut view = content();
    View::<M>::measure(&mut view, band.size);
    let size = View::<M>::size(&view);
    let pad = Theme::metric(ThemeMetric::SpacingSmall);
    let rect = Rect::new(
        band.x() - pad,
        band.y() - pad,
        size.width + 2 * pad,
        size.height + 2 * pad,
    );
    let frame = backend.with_display(|panel| crop(panel, rect));
    check(name, &frame)
}

/// Every picture that failed, reported together so one run blesses them all.
#[derive(Default)]
pub struct Failures(Vec<String>);

impl Failures {
    pub fn note(&mut self, outcome: Result<(), String>) {
        if let Err(why) = outcome {
            self.0.push(why);
        }
    }

    pub fn finish(self) {
        assert!(self.0.is_empty(), "{}", self.0.join("\n\n"));
    }
}
