//! What the tutorial's screen actually paints.
//!
//! Separate from `screen.rs` on purpose: this installs the `embedded_graphics`
//! backend as the global host, and `xpui::testing::install` is idempotent — it
//! will not take the global back off another backend. Sharing a process, a
//! behaviour test running after one of these would silently be asserting
//! against the wrong host. Cargo gives each test file its own process.

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use tutorial::{Message, SleepTimer};
use xpui::App;
use xpui::screen::Screen;
use xpui::testing::assert_text_snapshot;
use xpui_eg::{Backend, Framebuffer, Palette};

const WIDTH: i32 = 480;
const HEIGHT: i32 = 800;

static SERIAL: Mutex<()> = Mutex::new(());

fn serial() -> MutexGuard<'static, ()> {
    SERIAL
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn screenshots() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("../screenshots")
}

fn shoot(name: &str, screen: SleepTimer) -> &'static Backend<Framebuffer> {
    let backend = Backend::leak(Framebuffer::new(WIDTH, HEIGHT), Palette::INK_IS_ON);
    // Safety: serialised by `SERIAL`; nothing has rendered on this backend.
    unsafe { xpui::host::install(backend) };

    let mut app = App::new(screen);
    app.render();

    let thumbnail = backend.with_display(|frame| {
        frame.write_bmp_in(screenshots(), name);
        frame.thumbnail(60)
    });
    assert_text_snapshot(name, &thumbnail);
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
