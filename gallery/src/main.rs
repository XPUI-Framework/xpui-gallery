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
use gallery::boards;
use gallery::chord::Badge;
use xpui_boards_xteink as xteink;
use xpui_simulator::{Panel, Simulator};

fn main() {
    // The one it opens on when nothing is named. A real device rather than a
    // placeholder, so what opens is something that exists.
    let mut board = xteink::X4;
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
            "--board" => match args.next().as_deref().and_then(boards::from_slug) {
                Some(chosen) => board = chosen,
                None => fail(&format!(
                    "--board must be one of: {}",
                    boards::slugs().collect::<Vec<_>>().join(", ")
                )),
            },
            other => fail(&format!("unknown argument: {other}")),
        }
    }

    // The firmware's own reading of its keys. Every board here has a Back key
    // of its own, so this passes them all straight through; it earns its keep
    // on a board with three keys and no spare, which folds Back into a double
    // press. See `gallery::chord`.
    let simulator = Simulator::new(Panel::of(board))
        // The seven this example is built for. The simulator has no device
        // list of its own — it walks whatever it is handed, in this order —
        // so this is where the gallery says which panels it claims to fit.
        .boards(&boards::ALL)
        .title(format!("xpui — {}", board.name))
        .keys(Badge::default());

    // Said at startup, the way the RP2040 firmware says which board it booted
    // on: a window that opens on the wrong panel, or a `B` key that walks a
    // list nobody meant, is otherwise a thing you notice by pressing it.
    println!(
        "xpui: {} {}x{}, cycle {}",
        board.name,
        board.width,
        board.height,
        simulator
            .session()
            .boards()
            .iter()
            .map(|b| b.slug)
            .collect::<Vec<_>>()
            .join(" ")
    );

    let mut simulator = simulator;
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
