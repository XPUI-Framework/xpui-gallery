//! Opens the gallery in a window.
//!
//! ```bash
//! cargo run -p xpui-gallery
//! cargo run -p xpui-gallery -- --frames 60     # stop after 60 frames
//! ```
//!
//! Arrows move focus, Enter opens, Backspace goes back, H is the home gesture,
//! Q or Escape quits. Clicking is a tap and the scroll wheel is a swipe, so the
//! touch paths work too.
//!
//! `--frames` exists so the loop can be *tested*. Without it the only way out
//! is a person closing the window, which means CI — and any check that the
//! simulator still starts — hangs until something kills it.

use gallery::Menu;
use gallery::chord::Badge;
use xpui_simulator::{Board, Panel, Simulator};

fn main() {
    let mut board = Board::X4;
    let mut frames: Option<u32> = None;

    // Deliberately hand-parsed. Two optional flags do not justify a
    // command-line dependency in an example whose point is the UI.
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--frames" => match args.next().and_then(|n| n.parse().ok()) {
                Some(count) => frames = Some(count),
                None => fail("--frames needs a number"),
            },
            "--board" => match args.next().as_deref().and_then(Board::from_slug) {
                Some(chosen) => board = chosen,
                None => fail(&format!(
                    "--board must be one of: {}",
                    Board::ALL
                        .iter()
                        .map(|b| b.slug)
                        .collect::<Vec<_>>()
                        .join(", ")
                )),
            },
            other => fail(&format!("unknown argument: {other}")),
        }
    }

    // The firmware's own reading of its keys. A board with a Back key of its
    // own is untouched by it; one with three keys along the bottom takes its
    // Back from a double press of the first, which is the arrangement those
    // badges' own examples use. See `gallery::chord`.
    let mut simulator = Simulator::new(Panel::of(board))
        .title(format!("xpui — {}", board.name))
        .keys(Badge::default());
    if let Some(count) = frames {
        simulator = simulator.frames(count);
    }
    simulator.run(Menu::new());
}

fn fail(message: &str) -> ! {
    eprintln!("{message}");
    eprintln!("usage: gallery [--board SLUG] [--frames N]");
    std::process::exit(2);
}
