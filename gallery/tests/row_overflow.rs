//! A row must paint inside the bounds it was given.
//!
//! `draw_list` skips a row that does not fit, which is only a real guarantee if
//! `row_height` tells the truth about how tall a row will be. If it
//! under-reports — a subtitle drawn below the height it claimed — the list
//! passes its own fit check and paints over whatever is beneath it, which on a
//! small panel is the button hint bar.

use gallery::{metrics_for, wire};
use xpui::host::RowField;
use xpui::{Rect, Renderer};
use xpui_boards_core::Board;
use xpui_chrome::draw_list;
use xpui_eg::Palette;
use xpui_screenshot::Framebuffer;

fn probe(board: Board) {
    let backend = wire(
        Framebuffer::new(board.width, board.height),
        board,
        Palette::INK_IS_ON,
    )
    .leaked();
    // Safety: this file is its own process and nothing else installs a host.
    unsafe { xpui::host::install(backend) };

    let metrics = metrics_for(board);
    let top = metrics.content_top();
    let band = board.height - top - metrics.button_hints_height;
    let rect = Rect::new(0, top, board.width, band);

    Renderer::clear();
    draw_list(&metrics, rect, 8, 0, &|index, field| match field {
        RowField::Title => Some(["Controls", "Lists", "Dialogs", "Scrolling"][index % 4]),
        RowField::Subtitle => Some("Slider, stepper, toggle, progress"),
        _ => None,
    });

    // The rect is too generous to be the assertion. `draw_list` stops before a
    // row that does not fit, so on a 480x800 panel it leaves 224 pixels of
    // slack inside its own rect — a row could overflow its bounds by that much
    // and still land nowhere near the edge. The floor is the bottom of the last
    // row it actually painted.
    let cells = |index: usize, field: RowField| match field {
        RowField::Title => Some(["Controls", "Lists", "Dialogs", "Scrolling"][index % 4]),
        RowField::Subtitle => Some("Slider, stepper, toggle, progress"),
        _ => None,
    };
    let painted = xpui_chrome::rows_that_fit(&metrics, rect, 8, &cells);
    let stride = xpui_chrome::row_height(&metrics, 8, &cells) + metrics.list_row_gap;
    let floor = top + (painted as i32) * stride;

    let below = backend.with_display(|f| f.ink_in(0, floor, board.width, board.height - floor));
    assert_eq!(
        below, 0,
        "{}: {} pixels of ink below the last row that fits — {painted} rows of \
         {stride}px from y={top}, so anything past y={floor} is a row that \
         overflowed the height `row_height` promised",
        board.name, below
    );
}

#[test]
fn a_row_never_paints_below_the_rect_it_was_given() {
    for board in gallery::boards::ALL {
        probe(board);
    }
}
