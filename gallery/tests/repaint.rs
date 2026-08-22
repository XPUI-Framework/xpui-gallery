//! A keypress must reach the panel.
//!
//! Every other focus test drives `Runtime` and asserts `focused_index()`, which
//! moves correctly even when nothing is drawn. That is how "arrow keys do
//! nothing" survived the whole suite: the selection moved internally on every
//! press and the framebuffer never changed.
//!
//! So this test asserts the only thing a user can actually see — that the
//! pixels are different afterwards.

use gallery::Menu;
use xpui::{App, Button};
use xpui_eg::{Backend, Board, Palette};
use xpui_screenshot::Framebuffer;

/// Every pixel, so a comparison cannot miss a change the way a coarse
/// thumbnail or an ink count can — two frames can have identical ink totals
/// and look completely different.
fn pixels(backend: &'static Backend<Framebuffer>) -> Vec<bool> {
    backend.with_display(|frame| frame.ink().to_vec())
}

#[test]
fn pressing_down_changes_the_panel() {
    let board = Board::X4;
    let backend = Backend::leak_for_board(
        Framebuffer::new(board.width, board.height),
        board,
        Palette::INK_IS_ON,
    );
    // Safety: this file is its own process and nothing else installs a host.
    unsafe { xpui::host::install(backend) };

    let mut app = App::new(Menu::new());
    app.render();
    let before = pixels(backend);

    backend.press(Button::Down);
    app.tick();

    assert!(
        app.render_if_dirty(),
        "moving the focus must mark the panel stale"
    );

    let after = pixels(backend);
    assert_ne!(
        before, after,
        "the focus moved but every pixel is identical — the selection is \
         changing somewhere the panel never sees, which is exactly what makes \
         the arrow keys look dead"
    );
}
