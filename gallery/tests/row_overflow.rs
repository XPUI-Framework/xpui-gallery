//! A row must paint inside the bounds it was given.
//!
//! `draw_list` skips a row that does not fit, which is only a real guarantee if
//! `row_height` tells the truth about how tall a row will be. If it
//! under-reports — a subtitle drawn below the height it claimed — the list
//! passes its own fit check and paints over whatever is beneath it, which on a
//! small panel is the button hint bar.

use xpui::host::RowField;
use xpui::{Rect, Renderer};
use xpui_chrome::draw_list;
use xpui_eg::{Backend, Board, Framebuffer, Palette};

fn probe(board: Board) {
    let backend = Backend::leak_for_board(
        Framebuffer::new(board.width, board.height),
        board,
        Palette::INK_IS_ON,
    );
    // Safety: this file is its own process and nothing else installs a host.
    unsafe { xpui::host::install(backend) };

    let top = board.tokens.content_top();
    let band = board.height - top - board.tokens.button_hints_height;
    let rect = Rect::new(0, top, board.width, band);

    Renderer::clear();
    draw_list(&board.tokens, rect, 8, 0, &|index, field| match field {
        RowField::Title => Some(["Controls", "Lists", "Dialogs", "Scrolling"][index % 4]),
        RowField::Subtitle => Some("Slider, stepper, toggle, progress"),
        _ => None,
    });

    let below =
        backend.with_display(|f| f.ink_in(0, top + band, board.width, board.height - top - band));
    assert_eq!(
        below, 0,
        "{}: {} pixels of ink below the list's own rect — a row overflowed the \
         height `row_height` promised, so `draw_list`'s fit check let it through",
        board.name, below
    );
}

#[test]
fn a_row_never_paints_below_the_rect_it_was_given() {
    for board in Board::ALL {
        probe(board);
    }
}
