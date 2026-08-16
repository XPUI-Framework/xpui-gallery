//! Every example, rendered to real pixels.
//!
//! `gallery.rs` asserts behaviour against the draw calls. This renders the
//! same screens through the `embedded_graphics` backend and the `chrome`
//! components, so what it checks is the whole stack a device would run.
//!
//! ```bash
//! UPDATE_SNAPSHOTS=1 cargo test -p xpui-gallery
//! open target/screenshots/
//! ```

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

/// One place for every screenshot, inside the workspace's target directory.
///
/// `CARGO_TARGET_TMPDIR` is a compile-time variable cargo sets for integration
/// tests; the working directory at run time is the crate root, which in a
/// workspace is the wrong place.
fn screenshots() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("../screenshots")
}

use gallery::Menu;
use gallery::screens::{Controls, Dialogs, Lists, Scrolling, TextSizes};
use xpui::screen::Screen;
use xpui::testing::assert_text_snapshot;
use xpui::{App, Button};
use xpui_eg::{Backend, Framebuffer, Palette};

const WIDTH: i32 = 480;
const HEIGHT: i32 = 800;

static SERIAL: Mutex<()> = Mutex::new(());

fn serial() -> MutexGuard<'static, ()> {
    SERIAL
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn install() -> &'static Backend<Framebuffer> {
    let backend = Backend::leak(Framebuffer::new(WIDTH, HEIGHT), Palette::INK_IS_ON);
    // Safety: serialised by `SERIAL`, and nothing has rendered on this one.
    unsafe { xpui::host::install(backend) };
    backend
}

fn shoot<S: Screen + 'static>(name: &str, screen: S) -> &'static Backend<Framebuffer> {
    let backend = install();
    let mut app = App::new(screen);
    app.render();
    capture(backend, name);
    backend
}

fn capture(backend: &'static Backend<Framebuffer>, name: &str) {
    let thumbnail = backend.with_display(|frame| {
        frame.write_bmp_in(screenshots(), name);
        frame.thumbnail(60)
    });
    assert_text_snapshot(name, &thumbnail);
}

// -- the shots -------------------------------------------------------------

#[test]
fn the_menu() {
    let _guard = serial();
    let backend = shoot("gallery_menu", Menu::new());

    assert!(
        backend.with_display(|frame| frame.ink_in(0, 0, WIDTH, 56)) > 0,
        "the menu has a header"
    );
    assert!(
        backend.with_display(|frame| frame.ink_in(0, HEIGHT - 40, WIDTH, 40)) > 0,
        "and button hints"
    );
}

#[test]
fn the_controls_example() {
    let _guard = serial();
    shoot("gallery_controls", Controls::new());
}

#[test]
fn the_lists_example() {
    let _guard = serial();
    shoot("gallery_lists", Lists::new());
}

#[test]
fn the_dialogs_example() {
    let _guard = serial();
    shoot("gallery_dialogs", Dialogs::new());
}

/// The dialog open, over its own content, dimmed.
#[test]
fn the_dialogs_example_with_the_picker_open() {
    let _guard = serial();
    let backend = install();
    let mut app = App::new(Dialogs::new());
    app.render();

    backend.begin_frame(0);
    backend.press(Button::Confirm);
    app.tick();
    app.render();

    capture(backend, "gallery_dialogs_open");
}

#[test]
fn the_scrolling_example() {
    let _guard = serial();
    let backend = shoot("gallery_scrolling", Scrolling::new());

    // The list is longer than the panel, so the indicator must be there.
    assert!(
        backend.with_display(|frame| frame.ink_in(WIDTH - 8, 60, 8, 700)) > 0,
        "a page longer than the panel shows a scroll indicator"
    );
}

#[test]
fn the_text_example() {
    let _guard = serial();
    shoot("gallery_text", TextSizes::new());
}

/// Opening an example from the menu is the path a person actually takes, and
/// it crosses every layer: focus, navigation, the stack, and a repaint.
#[test]
fn opening_an_example_from_the_menu() {
    let _guard = serial();
    let backend = install();
    let mut app = App::new(Menu::new());
    app.render();

    backend.begin_frame(0);
    backend.press(Button::Confirm);
    app.tick();
    assert_eq!(app.depth(), 2, "an example opened");

    app.render();
    capture(backend, "gallery_opened_from_menu");
}

// -- the same screens, on every board --------------------------------------

/// The menu on each supported panel.
///
/// This is what the token presets are for, and the case that motivated them:
/// with the default chrome a Badger 2040's content band is 28 pixels and a
/// list draws **zero** rows, so the screen comes back empty. Each of these
/// asserts that rows actually appear, and writes a picture to be looked at.
#[test]
fn the_menu_on_every_board() {
    let _guard = serial();

    for board in xpui_eg::Board::ALL {
        let backend = Backend::leak_for_board(
            Framebuffer::new(board.width, board.height),
            board,
            Palette::INK_IS_ON,
        );
        // Safety: serialised by `SERIAL`; nothing has rendered on this one.
        unsafe { xpui::host::install(backend) };

        let mut app = App::new(Menu::new());
        app.render();

        let name = format!("board_{}", board.slug());
        let thumbnail = backend.with_display(|frame| {
            frame.write_bmp_in(screenshots(), &name);
            frame.thumbnail(60.min(board.width))
        });
        assert_text_snapshot(&name, &thumbnail);

        // The header band, and content below it.
        let chrome = board.tokens.content_top();
        assert!(
            backend.with_display(|f| f.ink_in(0, 0, board.width, chrome)) > 0,
            "{}: nothing in the header band",
            board.name
        );
        assert!(
            backend.with_display(|f| f.ink_in(
                0,
                chrome,
                board.width,
                board.height - chrome - board.tokens.button_hints_height
            )) > 0,
            "{}: the content band is empty — the chrome does not fit this panel",
            board.name
        );
    }
}

/// Every board must have room for a usable list. Three rows is the floor: two
/// entries and somewhere to scroll to.
#[test]
fn every_board_has_room_for_a_list() {
    for board in xpui_eg::Board::ALL {
        assert!(
            board.list_rows() >= 3,
            "{} ({}x{}) fits only {} list rows",
            board.name,
            board.width,
            board.height,
            board.list_rows()
        );
    }
}
