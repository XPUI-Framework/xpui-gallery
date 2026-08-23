//! That the simulator actually starts.
//!
//! This exists because it did not. `Window::events()` panics if it is called
//! before the first `update()` — documented, in `embedded-graphics-simulator`
//! — and the loop called it on its very first iteration. The workspace built,
//! every unit test passed, and the binary died on startup every single time.
//!
//! Building is not running. This runs it.

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Long enough for a debug binary to boot and paint thirty frames, short
/// enough that a hang is a failure rather than a coffee break.
const DEADLINE: Duration = Duration::from_secs(60);

/// Runs the gallery headlessly for `frames` frames and returns its output.
///
/// `SDL_VIDEODRIVER=dummy` gives SDL a windowless target, so this works over
/// ssh and on a CI runner with no display.
fn run_headless(frames: u32) -> (Option<i32>, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_gallery"))
        .args(["--frames", &frames.to_string()])
        .env("SDL_VIDEODRIVER", "dummy")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the gallery binary was built");

    let started = Instant::now();
    loop {
        match child.try_wait().expect("can poll the child") {
            Some(status) => {
                let output = child.wait_with_output().expect("can read the output");
                let text = format!(
                    "{}{}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
                return (status.code(), text);
            }
            None if started.elapsed() > DEADLINE => {
                // Killed rather than waited out: a simulator that does not
                // finish a fixed frame budget is stuck, and the useful thing
                // is the failure, not the eventual timeout of the whole suite.
                let _ = child.kill();
                let output = child.wait_with_output().expect("can read the output");
                panic!(
                    "the simulator did not finish {frames} frames in {DEADLINE:?}\n{}{}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            None => std::thread::sleep(Duration::from_millis(50)),
        }
    }
}

#[test]
fn the_simulator_starts_and_runs_a_frame_budget() {
    let (code, output) = run_headless(30);
    assert_eq!(
        code,
        Some(0),
        "the simulator exited with {code:?}:\n{output}"
    );
}

/// One frame is the case that used to panic: the loop reached `events()`
/// before anything had painted.
#[test]
fn the_very_first_frame_does_not_panic() {
    let (code, output) = run_headless(1);
    assert!(
        !output.contains("panicked"),
        "the first frame panicked:\n{output}"
    );
    assert_eq!(code, Some(0), "exited with {code:?}:\n{output}");
}

/// A bad argument should say so and exit, not start a window nobody asked for.
#[test]
fn an_unknown_argument_is_refused() {
    let output = Command::new(env!("CARGO_BIN_EXE_gallery"))
        .arg("--nonsense")
        .env("SDL_VIDEODRIVER", "dummy")
        .output()
        .expect("the gallery binary was built");

    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("unknown argument"),
        "it said why"
    );
}

/// The gallery offers all seven boards to the `B` key.
///
/// The simulator has no device list of its own — `gallery/src/main.rs`
/// passes one — so a deleted `.boards(..)` line ships a window whose `B` key
/// does nothing, on every board, with no test red anywhere. Nothing else here
/// reaches that line: the two tests above assert an exit code, and the
/// simulator's own tests build a `Session` directly and never touch this
/// binary.
///
/// Asserted against the startup line rather than by pressing `B`, because SDL
/// gives no way to inject an event into a running window.
#[test]
fn the_gallery_offers_every_board_to_the_board_key() {
    let (code, output) = run_headless(1);
    assert_eq!(code, Some(0), "exited with {code:?}:\n{output}");

    let cycle = output
        .lines()
        .find_map(|line| line.split_once("cycle "))
        .map(|(_, rest)| rest.trim().to_string())
        .unwrap_or_else(|| panic!("the gallery said nothing about its cycle:\n{output}"));

    // Named rather than counted: a cycle of the right length holding the wrong
    // boards is the fault this is for.
    let want = "x3 x4 x4pro sticky badger2040 tufty2040 inkyframe";
    assert_eq!(cycle, want, "the gallery walks a list it does not claim to");
}
