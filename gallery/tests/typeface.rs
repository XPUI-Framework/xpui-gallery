//! More than one typeface, and what changes when you pick another.
//!
//! The families here are the *application's* — assembled in
//! `gallery::fonts` out of faces the backend has never heard of. That is the
//! point of the arrangement: a firmware brings the type it wants and pays for
//! that and nothing else.

use std::sync::{Mutex, MutexGuard};

use gallery::Menu;
use gallery::fonts::{CENTURY, COURIER, FAMILIES};
use gallery::typeface::Typefaces;
use xpui::host::{Canvas, FontRole, FontStyle, TextMetrics};
use xpui::{App, Button, Screen};
use xpui_eg::{
    Backend, Family, FontRenderer, Fonts, Framebuffer, HELVETICA, Palette, Piece, Tier,
    assert_screenshot, clear_chosen_family, font_tier, u8g2,
};
use xpui_simulator::{Board, Control, Panel, Session};

const WIDTH: i32 = 480;
const HEIGHT: i32 = 800;

/// `xpui::host::install` writes a static, and so does the family request.
static SERIAL: Mutex<()> = Mutex::new(());

fn serial() -> MutexGuard<'static, ()> {
    let guard = SERIAL
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    // A family one test chose would still be in force inside the next.
    clear_chosen_family();
    guard
}

fn install() -> &'static Backend<Framebuffer> {
    let backend = Backend::leak(Framebuffer::new(WIDTH, HEIGHT), Palette::INK_IS_ON);
    // Safety: serialised by `SERIAL`, and nothing has rendered on this one.
    unsafe { xpui::host::install(backend) };
    backend
}

// -- the families themselves ----------------------------------------------

/// The same rule the backend's own family is held to. A declared band above
/// the tallest face wastes a row; one below it overprints the row beneath.
#[test]
fn every_registered_tier_declares_the_band_its_tallest_style_needs() {
    for family in FAMILIES {
        for tier in family.tiers {
            let tallest = [tier.regular, tier.bold]
                .into_iter()
                .chain(tier.italic)
                .chain(tier.bold_italic)
                .map(|face| face.get_default_line_height() as i32)
                .max()
                .unwrap_or(0);
            assert_eq!(
                tier.line_height, tallest,
                "{}: a tier says it is {}px and its tallest face is {tallest}px \
                 — the chrome hands out a band of the first and paints the second \
                 into it",
                family.name, tier.line_height
            );
        }
    }
}

/// Three families, and no two of them the same bytes. Two entries in a picker
/// that render identically is a picker that does nothing.
#[test]
fn the_registered_families_are_actually_different() {
    let mut seen = std::collections::HashSet::new();
    for family in FAMILIES {
        for tier in family.tiers {
            assert!(
                seen.insert(tier.id.0),
                "{} carries a tier whose bytes another family already had — an \
                 id is a hash of the faces, so this is the same font twice",
                family.name
            );
        }
    }
}

// -- swapping --------------------------------------------------------------

/// **The id must move with the family.** Every consumer that caches anything
/// against a `FontId` — a page layout, a measured column — is invalidated by
/// the id changing and by nothing else. Hand out a stable id over changed
/// bytes and those caches go stale with no way to notice.
#[test]
fn choosing_a_family_changes_every_id_it_hands_out() {
    let _guard = serial();
    let backend = install();

    let before: Vec<_> = [FontRole::Ui, FontRole::UiSmall, FontRole::Reader]
        .map(|role| backend.font(role))
        .into();

    backend.set_family(&COURIER);

    let after: Vec<_> = [FontRole::Ui, FontRole::UiSmall, FontRole::Reader]
        .map(|role| backend.font(role))
        .into();

    for (role, (was, now)) in [FontRole::Ui, FontRole::UiSmall, FontRole::Reader]
        .into_iter()
        .zip(before.iter().zip(after.iter()))
    {
        assert_ne!(
            was, now,
            "{role:?} reports the same id in Courier as in Helvetica, so \
             anything caching against it never learns the type changed"
        );
    }
}

/// Changing the type has to reach the panel, not only the metrics.
///
/// There are two repaint flags and they answer to different readers:
/// `Backend::dirty` is for a caller driving its own loop, and the framework's
/// own is what `App` consults. Setting only the first gives a screen measured
/// in the new face and painted in the old — and on e-ink, painted that way
/// until something else happens to change.
#[test]
fn changing_the_family_asks_for_the_panel_to_be_repainted() {
    let _guard = serial();
    let backend = install();
    let mut app = App::new(Menu::new());
    app.render();
    assert!(!app.render_if_dirty(), "nothing has changed yet");

    backend.set_family(&COURIER);

    assert!(
        app.render_if_dirty(),
        "the type changed and the app was not asked to paint again"
    );
}

/// A picker asks between frames; the backend applies it at the top of the
/// next one. Applied mid-frame it would leave a screen measured in one face
/// and painted in another.
#[test]
fn a_request_lands_on_the_next_frame_and_not_before() {
    let _guard = serial();
    let backend = install();
    let helvetica = backend.font(FontRole::Ui);

    let mut screen = Typefaces::new(FAMILIES);
    screen.update(1); // Courier

    assert_eq!(
        backend.font(FontRole::Ui),
        helvetica,
        "the request was made, not applied — this frame is still the one that \
         was measured in Helvetica"
    );

    backend.begin_frame(1);

    assert_ne!(
        backend.font(FontRole::Ui),
        helvetica,
        "the next frame opens in the family that was asked for"
    );
    assert_eq!(backend.fonts().family.name, "Courier");
}

/// The sizes come from the chrome and the family answers with what it was cut
/// in. A family that carried its own heights across would over-set the rows a
/// board laid out.
#[test]
fn a_swap_re_derives_the_sizes_from_the_chrome() {
    let _guard = serial();
    // Built with sizes that are *not* what this chrome asks for, so carrying
    // them across and re-deriving them give different answers. A backend left
    // on the default `Fonts` cannot tell the two apart, which is what made an
    // earlier version of this test pass whichever way `set_family` behaved.
    let backend: &'static Backend<Framebuffer> = Box::leak(Box::new(
        Backend::new(Framebuffer::new(WIDTH, HEIGHT), Palette::INK_IS_ON).with_fonts(Fonts::SMALL),
    ));
    // Safety: serialised by `SERIAL`, and nothing has rendered on this one.
    unsafe { xpui::host::install(backend) };

    let smuggled = backend.line_height(backend.font(FontRole::Ui));
    backend.set_family(&HELVETICA);
    let derived = backend.line_height(backend.font(FontRole::Ui));
    let wanted = Fonts::for_tokens(backend.tokens()).ui;

    assert_ne!(
        smuggled, derived,
        "the sizes it was built with survived the swap, so a family cut at \
         other heights would land in rows that were never measured for it"
    );
    assert_eq!(
        derived, wanted,
        "a swap asks the chrome what height it has room for, and the family \
         answers with the cut it has"
    );

    for family in FAMILIES {
        backend.set_family(family);
        let row = backend.tokens().list_row_height;
        let band = backend.line_height(backend.font(FontRole::Ui));
        assert!(
            band < row,
            "{}: a {row}px row was given a {band}px band",
            family.name
        );
    }
}

/// A choice is the application's, not one backend's.
///
/// The simulator keeps a backend per board and switches between them. A choice
/// consumed by the first backend to see it is undone by pressing `B`, with the
/// picker still showing "In use" beside a family nothing is set in.
#[test]
fn the_chosen_family_survives_moving_to_another_board() {
    let _guard = serial();
    let mut session = Session::new(Panel::of(Board::BADGER_2040));

    let mut screen = Typefaces::new(FAMILIES);
    screen.update(1); // Courier
    session.backend().begin_frame(1);
    assert_eq!(session.backend().fonts().family.name, "Courier");

    // Onto a board whose backend was built before any of this happened.
    while session.board() == Board::BADGER_2040 {
        assert!(session.apply(Control::NextBoard), "the cycle moves on");
    }
    session.backend().begin_frame(2);

    assert_eq!(
        session.backend().fonts().family.name,
        "Courier",
        "the board switched and the type reverted, while the picker still \
         says Courier is in use"
    );
}

// -- the fallback chain ----------------------------------------------------

/// A family cut for ASCII only, falling back to the full set.
///
/// `_tr` is u8g2's reduced repertoire — 32 to 127 and nothing else — which is
/// exactly the case a fallback exists for: a board short of flash ships the
/// reduced face and reaches for the full one on the rare accent.
const ASCII_18: Tier = font_tier!(30, u8g2::u8g2_font_helvR18_tr, u8g2::u8g2_font_helvB18_tr);

static ASCII_ONLY: Family = Family {
    name: "ASCII only",
    tiers: &[ASCII_18],
    fallback: Some(&HELVETICA),
};

static NO_FALLBACK: Family = Family {
    name: "ASCII only, alone",
    tiers: &[ASCII_18],
    fallback: None,
};

/// What one string resolves to, as `(face, text)` pairs.
fn resolved(family: &'static Family, text: &str) -> Vec<(*const FontRenderer, String)> {
    let font = xpui_eg::Face {
        family,
        tier: &family.tiers[0],
        style: FontStyle::Regular,
    };
    let mut found = Vec::new();
    xpui_eg::pieces(font, text, |piece| match piece {
        Piece::Run { face, text } => found.push((face as *const FontRenderer, text.to_string())),
        Piece::Marker { width, height } => {
            found.push((std::ptr::null(), format!("marker {width}x{height}")))
        }
    });
    found
}

/// Two families over one tier, differing only in what they fall back to.
///
/// They draw the same string differently — one reaches the accent, the other
/// draws a box — so they must not report the same id for it. An id is what a
/// consumer keys a cache on; two fonts sharing one is a cache that answers a
/// question it was never asked.
#[test]
fn two_families_over_one_tier_do_not_share_an_id() {
    let _guard = serial();
    let backend = install();

    backend.set_family(&ASCII_ONLY);
    let with_fallback = backend.font(FontRole::Ui);
    let reaches = backend.text_width(with_fallback, "caf\u{e9}", FontStyle::Regular);

    backend.set_family(&NO_FALLBACK);
    let alone = backend.font(FontRole::Ui);
    let boxed = backend.text_width(alone, "caf\u{e9}", FontStyle::Regular);

    assert_ne!(
        reaches, boxed,
        "these two are supposed to render the accent differently; if they do \
         not, this test proves nothing about ids"
    );
    assert_ne!(
        with_fallback, alone,
        "the same tier reached through two families reported one id, while \
         drawing the same string two ways"
    );
}

/// A chain that never ends is walked a bounded number of times.
///
/// `Family` has public fields and the README documents assembling one, so two
/// naming each other compiles. Unbounded, this hangs inside a text
/// measurement, mid-render, with no panic to point at.
#[test]
fn a_fallback_chain_that_loops_still_returns() {
    static LOOP_A: Family = Family {
        name: "loop A",
        tiers: &[ASCII_18],
        fallback: Some(&LOOP_B),
    };
    static LOOP_B: Family = Family {
        name: "loop B",
        tiers: &[ASCII_18],
        fallback: Some(&LOOP_A),
    };

    // A character neither has, so the walk goes all the way round.
    let pieces = resolved(&LOOP_A, "a\u{2603}b");
    assert_eq!(pieces.len(), 3, "a run, a marker and a run: {pieces:?}");
    assert!(
        pieces[1].0.is_null(),
        "a chain that runs out draws a marker"
    );
}

#[test]
fn a_glyph_the_primary_lacks_comes_from_the_fallback() {
    let ascii = ASCII_ONLY.tiers[0].regular as *const FontRenderer;
    let full = HELVETICA.tier_for(30).regular as *const FontRenderer;

    // Precomposed `é`, U+00E9: inside the full face's 8-bit repertoire and
    // outside the reduced one's 32-to-127. A combining accent would be
    // outside *both* and would prove nothing about the fallback.
    let pieces = resolved(&ASCII_ONLY, "caf\u{e9}");
    assert_eq!(
        pieces.len(),
        2,
        "expected an ASCII run and a fallback run, got {pieces:?}"
    );
    assert_eq!(pieces[0].0, ascii, "the ASCII part stayed in the primary");
    assert_eq!(
        pieces[1].0, full,
        "the accent came from the fallback family, at the size of the text \
         around it"
    );
}

#[test]
fn a_glyph_nobody_has_draws_a_marker() {
    let pieces = resolved(&NO_FALLBACK, "a\u{2603}b");
    assert_eq!(pieces.len(), 3, "a run, a marker and a run: {pieces:?}");
    assert!(
        pieces[1].0.is_null(),
        "a snowman is in no face here, and a label that silently drops it \
         reads as the wrong word"
    );
}

/// The concrete symptom the chain was built for.
#[test]
fn an_ellipsis_nobody_has_is_three_dots_wide_and_three_dots_drawn() {
    let _guard = serial();
    let backend = install();
    let font = backend.font(FontRole::Ui);

    let ellipsis = backend.text_width(font, "…", FontStyle::Regular);
    let dots = backend.text_width(font, "...", FontStyle::Regular);

    assert!(
        ellipsis > 0,
        "a truncated label ended in nothing at all: no width and no ink"
    );
    assert_eq!(
        ellipsis, dots,
        "measured as something other than what is drawn for it"
    );

    let pieces = resolved(&HELVETICA, "…");
    assert_eq!(pieces.len(), 1);
    assert_eq!(
        pieces[0].1, "...",
        "the substitution is made in the walk both measuring and drawing use, \
         so the two cannot disagree about it"
    );
}

/// Measuring and drawing take the same walk, so a string is as wide as its
/// parts however it is asked.
#[test]
fn measuring_a_string_agrees_with_measuring_its_pieces() {
    let _guard = serial();
    let backend = install();
    let font = backend.font(FontRole::Ui);

    for text in ["Hamburgefonstiv", "café…", "a\u{2603}b", "", "…"] {
        let whole = backend.text_width(font, text, FontStyle::Regular);
        let summed: i32 = {
            let resolved = backend.fonts().face(font, FontStyle::Regular);
            let mut total = 0;
            xpui_eg::pieces(resolved, text, |piece| total += xpui_eg::advance(piece));
            total
        };
        assert_eq!(
            whole, summed,
            "{text:?} measures {whole} whole and {summed} in pieces — drawing \
             advances by the pieces, so the difference is what lands outside \
             what was reserved"
        );
    }
}

/// Which columns of the panel have ink on them, between `from` and `to`.
fn inked_columns(backend: &'static Backend<Framebuffer>, from: i32, to: i32) -> Vec<i32> {
    backend.with_display(|frame| {
        (from..to)
            .filter(|x| (0..HEIGHT).any(|y| frame.get(*x, y)))
            .collect()
    })
}

/// Two markers in a row have to read as two. Butted against each other they
/// are one wide box with a line down it, which is a different character
/// altogether — so a marker occupies its box **and a column of air**, and the
/// measurement has to account for that column or the next glyph sits in it.
///
/// Asserted on pixels because that is the only place it shows: measuring
/// agrees with itself whatever the advance is.
#[test]
fn two_markers_in_a_row_do_not_merge_into_one() {
    let _guard = serial();
    let backend = install();
    let font = backend.font(FontRole::Ui);
    let width = backend.text_width(font, "\u{2603}\u{2603}", FontStyle::Regular);

    backend.clear();
    backend.draw_text(
        xpui::Point::new(10, 40),
        "\u{2603}\u{2603}",
        font,
        FontStyle::Regular,
    );

    let inked = inked_columns(backend, 10, 10 + width);
    let gaps: Vec<i32> = (10..10 + width).filter(|x| !inked.contains(x)).collect();

    assert!(
        !gaps.is_empty(),
        "two markers painted edge to edge across {width}px with no column of \
         panel between them: one box with a divider, not two characters"
    );
}

/// Ink stays inside what was reserved. A string that measures narrower than it
/// paints overruns whatever is laid out beside it, and a list is laid out
/// entirely from these numbers.
#[test]
fn nothing_paints_wider_than_it_measured() {
    let _guard = serial();
    let backend = install();
    let font = backend.font(FontRole::Ui);

    for text in [
        "Hamburgefonstiv",
        "caf\u{e9}\u{2026}",
        "a\u{2603}b",
        "\u{2026}",
    ] {
        let width = backend.text_width(font, text, FontStyle::Regular);
        backend.clear();
        backend.draw_text(xpui::Point::new(10, 40), text, font, FontStyle::Regular);

        let overrun = inked_columns(backend, 10 + width, WIDTH);
        assert!(
            overrun.is_empty(),
            "{text:?} measured {width}px and left ink at {overrun:?}, past \
             where anything beside it would have been placed"
        );
    }
}

// -- what it looks like ----------------------------------------------------

/// The same menu in each family, as pixels.
///
/// The proof that a swap reaches the screen rather than only the metrics: a
/// serif and a mono cannot render as the same image, and if a golden ever
/// stops differing from another, something has stopped applying.
#[test]
fn the_menu_in_every_family() {
    let _guard = serial();
    for (name, family) in [
        ("typeface_helvetica", &HELVETICA),
        ("typeface_courier", &COURIER),
        ("typeface_century", &CENTURY),
    ] {
        let backend = install();
        backend.set_family(family);
        let mut app = App::new(Menu::new());
        app.render();
        backend.with_display(|frame| assert_screenshot(name, frame));
    }
}

/// Opening the picker and choosing a row sets the type for everything, not
/// only for the screen that did the choosing.
#[test]
fn choosing_from_the_picker_sets_the_type_underneath_it() {
    let _guard = serial();
    let backend = install();
    let mut app = App::new(Menu::new());
    app.render();
    let opened_in = backend.font(FontRole::Ui);

    app.push(Typefaces::new(FAMILIES));
    app.render();

    // Down to Courier, then confirm.
    backend.begin_frame(1);
    backend.press(Button::Down);
    app.tick();
    backend.begin_frame(2);
    backend.press(Button::Confirm);
    app.tick();

    backend.begin_frame(3);
    assert_ne!(
        backend.font(FontRole::Ui),
        opened_in,
        "the picker changed the type for the whole app, not for its own rows"
    );
}
