//! Runs the tutorial's screen in a window.
//!
//! ```bash
//! cargo run -p xpui-tutorial
//! cargo run -p xpui-tutorial -- --frames 60
//! ```

use tutorial::SleepTimer;
use xpui_simulator::{Panel, Simulator};

fn main() {
    let mut simulator = Simulator::new(Panel::PORTRAIT).title("xpui — tutorial");

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--frames" => match args.next().and_then(|n| n.parse().ok()) {
                Some(frames) => simulator = simulator.frames(frames),
                None => {
                    eprintln!("--frames needs a number");
                    std::process::exit(2);
                }
            },
            other => {
                eprintln!("unknown argument: {other}");
                std::process::exit(2);
            }
        }
    }

    simulator.run(SleepTimer::new());
}
