//! Every gallery screen, on every board, as real pixels.
//!
//! `gallery.rs` asserts behaviour against the draw calls. This renders the
//! same screens through the `embedded_graphics` backend and the `chrome`
//! components, so what it checks is the whole stack a device would run.
//!
//! **Ten screens across seven boards: seventy goldens, about 142 kB.**
//! The chrome is sized from tokens, and a token that lays out comfortably on a
//! 600x448 Inky Frame can leave a 296x128 Badger with a content band of a few
//! dozen pixels — so one panel proves nothing about the other six.
//!
//! The cost is small enough to read in a diff: these are 1-bit panels, and the
//! files average 2,070 bytes. What it buys is measured rather than argued, by
//! moving one token by one pixel and counting:
//!
//! | one pixel added to | moves | on |
//! |---|---|---|
//! | `Tokens::DEFAULT.list_row_height` | 35 captures | 5 boards |
//! | `Tokens::SMALL.list_row_height` | 7 captures | the Badger |
//! | `Tokens::SMALL.header_height` | 10 captures | the Badger |
//!
//! `Tokens::SMALL` is the preset no other board in `Board::ALL` uses. Moving
//! its `list_row_height` was **green across the whole repository** until this
//! file captured more than the menu: the menu's rows carry subtitles, so they
//! are laid out from `list_row_height_with_subtitle`, and the single Badger
//! capture that existed did not move. Moving its `header_height` was already
//! caught by that same capture — one of the two got through, which is the
//! shape of the gap rather than its size.
//!
//! The X4 Pro and the Sticky are the same 480x800 panel at the same preset, so
//! their goldens are byte-identical today. They are kept apart because a
//! preset belongs to a board, and either board's could change alone.
//!
//! ```bash
//! cargo test -p xpui-gallery --test screenshots
//! open examples/gallery/tests/screenshots/
//! open target/diff/                                  # after a failure
//! ```
//!
//! ## What `UPDATE_SNAPSHOTS=1` commits you to
//!
//! ```bash
//! UPDATE_SNAPSHOTS=1 cargo test -p xpui-gallery --test screenshots
//! ```
//!
//! **A blessed golden is an assertion you have made**: that this is what the
//! screen should look like on that panel. Blessing seventy at once makes
//! seventy of them in one keystroke, and a regression blessed is a
//! regression with a test agreeing with it — the one failure mode this whole
//! technique has. Open the directory and read the diff before committing.
//!
//! A board that fails [`chrome_fits`] is not blessed at all: the capture never
//! happens, because the floor check runs first and stops that board. So this
//! flag cannot write a picture of a screen that came back blank.

use std::sync::{Mutex, MutexGuard};

use gallery::fonts::FAMILIES;
use gallery::menu::Example;
use gallery::screens::{Controls, Dialogs, Lists, Scrolling, TextSizes};
use gallery::typeface::Typefaces;
use gallery::{DevelopersScreen, Menu};
use xpui::host::RowField;
use xpui::{App, Button, Rect};
use xpui_eg::{Backend, Board, Palette};
use xpui_screenshot::{Framebuffer, check_screenshot};

/// How many rows the menu offers, which is what [`rows_are_painted`] measures
/// against.
const MENU_ROWS: usize = Example::ALL.len();

/// `xpui::host::install` writes a process-wide static.
static SERIAL: Mutex<()> = Mutex::new(());

fn serial() -> MutexGuard<'static, ()> {
    SERIAL
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// A backend the size of `board`, carrying the chrome that board asks for.
fn install(board: Board) -> &'static Backend<Framebuffer> {
    let backend = Backend::leak_for_board(
        Framebuffer::new(board.width, board.height),
        board,
        Palette::INK_IS_ON,
    );
    // Safety: serialised by `SERIAL`, and nothing has rendered on this one.
    unsafe { xpui::host::install(backend) };
    backend
}

/// Paints one screen on one board, and answers for whatever only that screen
/// can be asked. `Err` is what to say about this board.
type Paint = fn(&'static Backend<Framebuffer>, Board) -> Result<(), String>;

/// Whether the screen writes its own name in the header band.
///
/// Every screen in the gallery names itself, so the only marker in use for a
/// screen's own header is [`Titled`](Header::Titled). One that deliberately
/// had no title would need a variant of its own rather than
/// [`Overlaid`](Header::Overlaid), which records a different reason — a dialog
/// over the band — and would let a missing title through unasked.
#[derive(Copy, Clone)]
enum Header {
    /// The title is painted, so the band holds more than the rule under it.
    Titled,
    /// Something is drawn over the band, so what is in it is not the screen's
    /// answer and **this check is not made**.
    ///
    /// The picker, and only because a dialog is over it. Whatever is left of
    /// the header — dimmed, or hidden under a dialog that fills the panel —
    /// the band has ink in it from something that is not the header, so asking
    /// would pass on every board however the dialog behaved. Measured, not
    /// assumed: with the title suppressed and the picker marked `Titled`, the
    /// check passes on all seven. A check that cannot fail is worse than none,
    /// because it is counted. What covers the picker instead is its own
    /// before-and-after comparison and its seven goldens.
    Overlaid,
}

/// Captures `screen` on all seven boards, and reports **every** one that
/// failed rather than the first.
///
/// A change to a token, a row height or a hint moves several panels at once,
/// and a run that stops at the first hides how far it reached — which is the
/// question these captures exist to answer.
fn on_every_board(screen: &str, header: Header, paint: Paint) {
    let _guard = serial();
    let mut failed: Vec<(Board, String)> = Vec::new();

    for board in Board::ALL {
        let backend = install(board);
        let outcome = paint(backend, board)
            .and_then(|()| chrome_fits(backend, board, header))
            .and_then(|()| check(backend, &format!("{screen}_{}", board.slug)));
        if let Err(why) = outcome {
            failed.push((board, why));
        }
    }

    report(&format!("the {screen} screen"), &failed);
}

/// Panics naming `what` and every board that failed, or returns.
///
/// `what` is the whole subject — "the menu screen", "controls opened from the
/// menu" — because not every caller here is a screen.
fn report(what: &str, failed: &[(Board, String)]) {
    if failed.is_empty() {
        return;
    }

    let boards: Vec<&str> = failed.iter().map(|(board, _)| board.name).collect();
    let detail: Vec<String> = failed
        .iter()
        .map(|(board, why)| {
            format!(
                "---- {what} on the {} ({}x{}) ----\n{why}",
                board.name, board.width, board.height
            )
        })
        .collect();

    panic!(
        "{what} fails on {} of {} boards: {}\n\n{}",
        failed.len(),
        Board::ALL.len(),
        boards.join(", "),
        detail.join("\n\n"),
    );
}

/// The frame against its golden.
fn check(backend: &'static Backend<Framebuffer>, name: &str) -> Result<(), String> {
    backend.with_display(|frame| check_screenshot(name, frame))
}

/// The floor every screen clears on every panel: the header band holds what
/// `header` says it should, the content band is not blank, and the hint bar
/// has something in it **on the boards that have one** — the X4 Pro and the
/// Sticky are touch panels with no hint bar at all, and nothing is asserted
/// about a band of zero height.
///
/// The content band is where the token presets earn their keep. With the
/// default chrome a Badger 2040 has 28 pixels of it, and a list refuses to
/// paint a row that does not fit — so the screen comes back empty, and the
/// golden of it would be a picture of nothing.
///
/// **That the band is not blank is all it proves.** A screen whose section
/// headings paint but whose rows do not still clears it. What the rows do is
/// the golden's job, and on the menu — whose content band is a list and
/// nothing else — [`rows_are_painted`] says so directly.
fn chrome_fits(
    backend: &'static Backend<Framebuffer>,
    board: Board,
    header: Header,
) -> Result<(), String> {
    let (top, band) = content_band(board);
    let hints = board.tokens.button_hints_height;

    // Above the rule the header paints at `top_padding + header_height`, so
    // this is the title itself rather than the chrome that would be there
    // whether or not the screen named itself.
    let title_band = board.tokens.top_padding + board.tokens.header_height;
    let titled = ink(backend, 0, 0, board.width, title_band) > 0;
    // Spelled out rather than left to a wildcard: a variant added later must
    // be given an answer here, which is the whole reason the marker exists.
    match (header, titled) {
        (Header::Overlaid, _) => {}
        (Header::Titled, true) => {}
        (Header::Titled, false) => {
            return Err(format!(
                "nothing in the {title_band}px title band: the header rule is \
                 painted but the screen's name is not"
            ));
        }
    }

    if ink(backend, 0, top, board.width, band) == 0 {
        return Err(format!(
            "the content band is empty: {band}px from y={top}, and the chrome \
             does not fit this panel"
        ));
    }
    if hints > 0 && ink(backend, 0, board.height - hints, board.width, hints) == 0 {
        return Err(format!("nothing in the {hints}px hint bar"));
    }
    Ok(())
}

/// Ink below the first row of a list that fills the content band.
///
/// The check [`chrome_fits`] cannot make for a screen built out of sections:
/// one row painted and the rest dropped leaves the band far from empty.
///
/// How many rows *should* be there is asked of the same function the painter
/// uses, rather than assumed from a token — `every_board_has_room_for_a_list`
/// answers for `list_row_height`, and every row here carries a subtitle, which
/// is a taller row and a different number. A board with room for one row would
/// make this vacuous, so it says so instead of passing.
fn rows_are_painted(backend: &'static Backend<Framebuffer>, board: Board) -> Result<(), String> {
    let (top, band) = content_band(board);
    let rect = Rect::new(0, top, board.width, band);
    // The menu's own rows rather than a copy of them: only whether a subtitle
    // is there changes the arithmetic, so a copy that drifted would keep
    // measuring from a row height the list no longer uses, and say nothing
    // while looking like it had.
    let cells = |index: usize, field: RowField| match field {
        RowField::Title => Some(Example::ALL[index].title()),
        RowField::Subtitle => Some(Example::ALL[index].summary()),
        _ => None,
    };
    let fits = xpui_chrome::rows_that_fit(&board.tokens, rect, MENU_ROWS, &cells);
    if fits < 2 {
        return Err(format!(
            "{band}px of content band from y={top} has room for {fits} of the \
             menu's {MENU_ROWS} rows, so `nothing below the first row` would \
             pass whatever the list painted"
        ));
    }

    let stride =
        xpui_chrome::row_height(&board.tokens, MENU_ROWS, &cells) + board.tokens.list_row_gap;
    // Short of the scroll indicator, which runs the height of the band on the
    // boards the menu does not fit: its dither is ink below the first row on
    // three of the seven, and would answer this question for the list.
    let rows_end = board.width - board.tokens.scrollbar_width - board.tokens.scrollbar_inset;
    if ink(backend, 0, top + stride, rows_end, band - stride) == 0 {
        return Err(format!(
            "nothing below the first row, where {fits} rows of {stride}px fit: \
             the list painted one and stopped"
        ));
    }
    Ok(())
}

fn ink(backend: &'static Backend<Framebuffer>, x: i32, y: i32, width: i32, height: i32) -> usize {
    backend.with_display(|frame| frame.ink_in(x, y, width, height))
}

/// The band a screen's own content is laid out in, between the header and the
/// hints.
fn content_band(board: Board) -> (i32, i32) {
    let top = board.tokens.content_top();
    (top, board.height - top - board.tokens.button_hints_height)
}

// -- the screens -----------------------------------------------------------

/// The list you land on. Seven rows, each with a subtitle, so this is also the
/// board's answer to how many two-line rows a panel holds — and its content
/// band is a list and nothing else, which is what lets it assert that rows
/// were actually painted.
#[test]
fn the_menu_on_every_board() {
    on_every_board("menu", Header::Titled, |backend, board| {
        App::new(Menu::new()).render();
        rows_are_painted(backend, board)
    });
}

/// The screen [`30`](../../../docs/specs/30-editing-a-value-with-one-key.md)
/// rebuilds: a slider, a stepper, a toggle and a progress bar.
#[test]
fn the_controls_example_on_every_board() {
    on_every_board("controls", Header::Titled, |_backend, _board| {
        App::new(Controls::new()).render();
        Ok(())
    });
}

/// The same screen with a value **open**, on every board.
///
/// [`34`](../../../docs/specs/34-a-mode-you-can-see.md)'s third state, and the
/// only capture of it. Without this the suite holds `Idle` and `Focused` on
/// seven panels and `Editing` on none — a state that draws correctly on one
/// panel and not another is exactly what this file exists to notice, and the
/// mode is what that whole spec is about.
///
/// **Three boards never open one**, and that is the assertion for them rather
/// than an exemption: the X3, the X4 and the Inky Frame have a Left/Right pair,
/// so Confirm nudges nothing and opens nothing, and their goldens here are the
/// unopened screen. A change that started opening an edit where the pair exists
/// would move those three.
#[test]
fn a_value_open_for_editing_on_every_board() {
    on_every_board("controls_open", Header::Titled, |backend, board| {
        let mut app = App::new(Controls::new());
        app.render();

        // Down onto the slider, then Confirm — which opens it on a board with
        // no pair and does nothing at all on one that has it.
        for key in [Button::Down, Button::Confirm] {
            backend.begin_frame(0);
            backend.press(key);
            app.tick();
            app.render();
        }

        let open = backend.with_display(|frame| frame.ink().to_vec());

        // The control frame: the same screen with nothing focused and nothing
        // open. Without it "these pixels" is a picture rather than a claim.
        App::new(Controls::new()).render();
        let closed = backend.with_display(|frame| frame.ink().to_vec());

        // Put the state under test back, since that comparison repainted.
        app.render();

        if board.has_left_right_keys() {
            // Confirm neither nudges nor opens where the pair exists, so this
            // board's golden is the unopened screen and has to stay that way.
            return Ok(());
        }
        if open == closed {
            return Err(
                "the panel is pixel-identical with a value open: the mode is \
                 invisible on this board, whatever its golden holds"
                    .into(),
            );
        }
        Ok(())
    });
}

#[test]
fn the_lists_example_on_every_board() {
    on_every_board("lists", Header::Titled, |_backend, _board| {
        App::new(Lists::new()).render();
        Ok(())
    });
}

#[test]
fn the_dialogs_example_on_every_board() {
    on_every_board("dialogs", Header::Titled, |_backend, _board| {
        App::new(Dialogs::new()).render();
        Ok(())
    });
}

/// The picker open, over its own content, dimmed.
///
/// The board that matters here is the smallest: a dialog is a percentage of
/// the panel width with a border and padding inside it, and on a 296x128
/// strip there is a size at which it has no room left for a row. Captured
/// rather than skipped — a dialog that cannot fit is a fact about the chrome,
/// and a picture of it is better evidence than an absent file.
///
/// On a Badger it takes the full height of the panel and 92% of its width,
/// covering the header and the hint bar and leaving twelve dimmed columns down
/// each side. On the other six it is a box with the screen showing through a
/// dither around it, the title still legible in the band above.
#[test]
fn the_picker_on_every_board() {
    on_every_board("picker", Header::Overlaid, |backend, _board| {
        let mut app = App::new(Dialogs::new());
        app.render();
        let closed = backend.with_display(|frame| frame.ink().to_vec());

        backend.begin_frame(0);
        backend.press(Button::Confirm);
        app.tick();
        app.render();

        if backend.with_display(|frame| frame.ink() == closed) {
            return Err(
                "the panel is pixel-identical with the picker open: no dialog \
                 and no scrim, whatever the golden of it holds"
                    .into(),
            );
        }
        Ok(())
    });
}

/// More rows than any panel holds, so every board must draw the indicator.
#[test]
fn the_scrolling_example_on_every_board() {
    on_every_board("scrolling", Header::Titled, |backend, board| {
        App::new(Scrolling::new()).render();

        let (top, band) = content_band(board);
        let strip = board.width - board.tokens.scrollbar_width - board.tokens.scrollbar_inset;
        if ink(backend, strip, top, board.tokens.scrollbar_width, band) == 0 {
            return Err(format!(
                "a page longer than the panel drew no scroll indicator in the \
                 {}px strip at x={strip}",
                board.tokens.scrollbar_width
            ));
        }
        Ok(())
    });
}

/// Font roles and weights, and a line longer than any of these panels.
///
/// What six of the seven goldens show that line doing is running off the
/// right-hand edge: `Text` paints what it is given, so the last glyph is
/// sliced by the framebuffer and there is no ellipsis. On the Badger the line
/// is below the fold and not in the picture at all. `ListRow` truncates and
/// `Text` does not — the Developers screen has both, in one capture.
#[test]
fn the_text_example_on_every_board() {
    on_every_board("text", Header::Titled, |_backend, _board| {
        App::new(TextSizes::new()).render();
        Ok(())
    });
}

/// The family picker, on each panel. The *menu* set in each face is a
/// different axis and lives in `typeface.rs`, as `family_<name>.png`.
#[test]
fn the_typeface_example_on_every_board() {
    on_every_board("typeface", Header::Titled, |_backend, _board| {
        App::new(Typefaces::new(FAMILIES)).render();
        Ok(())
    });
}

/// Everything at once: memory rows, a picker, a slider, a stepper and a long
/// scrolling section on one screen. The composite is where a per-board layout
/// fault shows first, because the widgets have to fit beside each other.
///
/// One thing to know before reading a diff of these seven: the memory figures
/// come from a counter that advances on every reading, so they are stable only
/// as long as the runtime calls `body()` the same number of times per render.
/// A diff in the *figures* rather than in the layout means that count changed,
/// not that the screen did.
#[test]
fn the_developers_example_on_every_board() {
    on_every_board("developers", Header::Titled, |_backend, _board| {
        App::new(DevelopersScreen::new()).render();
        Ok(())
    });
}

// -- and the way in --------------------------------------------------------

/// Opening an example from the menu is the path a person actually takes, and
/// it crosses every layer: focus, navigation, the stack, and a repaint.
///
/// It owns no golden. Arriving from the menu has to land on exactly the screen
/// `the_controls_example_on_every_board` blessed, and the way to say that is
/// to render both and compare the two framebuffers — a second file of the same
/// pixels is a second thing to keep in step, and a second *writer* of the same
/// file is worse: on a board added tomorrow, whichever test ran first would
/// create the golden and the other would pass against a picture nobody had
/// looked at.
#[test]
fn opening_an_example_from_the_menu_on_every_board() {
    let _guard = serial();
    let mut failed: Vec<(Board, String)> = Vec::new();

    for board in Board::ALL {
        let direct = install(board);
        App::new(Controls::new()).render();
        let expected = direct.with_display(|frame| frame.ink().to_vec());

        let arrived = install(board);
        let mut app = App::new(Menu::new());
        app.render();
        arrived.begin_frame(0);
        arrived.press(Button::Confirm);
        app.tick();

        if app.depth() != 2 {
            failed.push((
                board,
                format!(
                    "confirming the first row opened nothing: the stack is {} deep",
                    app.depth()
                ),
            ));
            continue;
        }

        app.render();
        if arrived.with_display(|frame| frame.ink() != expected) {
            failed.push((
                board,
                "Controls opened from the menu does not paint what Controls \
                 opened on its own paints, so one of them is not the screen \
                 `controls_<board>.png` was blessed from"
                    .into(),
            ));
        }
    }

    report("controls opened from the menu", &failed);
}

/// Every board must have room for a usable list. Three rows is the floor: two
/// entries and somewhere to scroll to.
#[test]
fn every_board_has_room_for_a_list() {
    for board in Board::ALL {
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
