//! What the tutorial's screen actually paints.
//!
//! Separate from `screen.rs` on purpose: this installs the `embedded_graphics`
//! backend as the global host, and `xpui::testing::install` is idempotent — it
//! will not take the global back off another backend. Sharing a process, a
//! behaviour test running after one of these would silently be asserting
//! against the wrong host. Cargo gives each test file its own process.

use std::sync::{Mutex, MutexGuard};

use tutorial::{Message, SleepTimer};
use xpui::App;
use xpui::screen::Screen;
use xpui_eg::{Backend, Framebuffer, Palette, assert_screenshot};

const WIDTH: i32 = 480;
const HEIGHT: i32 = 800;

static SERIAL: Mutex<()> = Mutex::new(());

fn serial() -> MutexGuard<'static, ()> {
    SERIAL
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn shoot(name: &str, screen: SleepTimer) -> &'static Backend<Framebuffer> {
    let backend = Backend::leak(Framebuffer::new(WIDTH, HEIGHT), Palette::INK_IS_ON);
    // Safety: serialised by `SERIAL`; nothing has rendered on this backend.
    unsafe { xpui::host::install(backend) };

    let mut app = App::new(screen);
    app.render();

    backend.with_display(|frame| assert_screenshot(name, frame));
    backend
}

#[test]
fn the_finished_screen() {
    let _guard = serial();
    let backend = shoot("tutorial", SleepTimer::new());

    assert!(
        backend.with_display(|f| f.ink_in(0, 0, WIDTH, 56)) > 0,
        "it has a header"
    );
    assert!(
        backend.with_display(|f| f.ink_in(0, HEIGHT - 40, WIDTH, 40)) > 0,
        "and button hints"
    );
}

#[test]
fn the_finished_screen_with_the_picker_open() {
    let _guard = serial();
    let mut screen = SleepTimer::new();
    screen.update(Message::OpenPresets);
    shoot("tutorial_picker", screen);
}
