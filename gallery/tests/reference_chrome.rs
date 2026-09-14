//! The pictures on `xpui-chrome`'s reference pages.
//!
//! Every picture is one of the crate's paint functions called directly, on the
//! X3 like every other reference picture, with the metrics and words the
//! gallery wires that board with. Nothing here goes through a screen: these
//! are what a backend gets when it calls the function itself.
//!
//! ```bash
//! UPDATE_SNAPSHOTS=1 cargo test -p xpui-gallery --test reference_chrome
//! open gallery/tests/screenshots/reference/
//! ```
//!
//! **Open every image before committing it.** A golden here is shown to every
//! reader of the page, not only compared.

mod support;

use gallery::metrics_for;
use support::{BOARD, Failures, check, crop, hint_bar, install, whole};
use xpui::host::{ControlState, Hint, IconRef, KeyRow, RowField, RowKey};
use xpui::{Point, Rect, Renderer};
use xpui_chrome::{
    Icon, Labels, Metrics, draw_button_hints, draw_header, draw_icon, draw_list, draw_option_popup,
    draw_progress_bar, draw_scroll_indicator, draw_slider, draw_sub_header,
};

/// One row: a title, a subtitle and a value.
type Cells = (&'static str, Option<&'static str>, Option<&'static str>);

const SETTINGS: [Cells; 3] = [
    ("Frontlight", None, Some("On")),
    ("Sleep after", None, Some("5 min")),
    ("Free heap", None, Some("182 KB")),
];

const NETWORKS: [Cells; 2] = [
    ("Wi-Fi", Some("Home network"), Some("On")),
    ("Bluetooth", Some("No devices"), Some("Off")),
];

const LIBRARY: [Cells; 5] = [
    ("Middlemarch", None, None),
    ("The Odyssey", None, None),
    ("Walden", None, None),
    ("Moby-Dick", None, None),
    ("Persuasion", None, None),
];

const DELAYS: [&str; 4] = ["1 min", "5 min", "15 min", "Never"];

const LANGUAGES: [&str; 24] = [
    "Afrikaans",
    "Bahasa Indonesia",
    "Bahasa Melayu",
    "Català",
    "Cymraeg",
    "Dansk",
    "Deutsch",
    "Eesti",
    "English",
    "Español",
    "Euskara",
    "Français",
    "Gaeilge",
    "Galego",
    "Íslenska",
    "Italiano",
    "Kiswahili",
    "Magyar",
    "Nederlands",
    "Norsk",
    "Polski",
    "Português",
    "Suomi",
    "Svenska",
];

/// A row callback over `cells`, as `draw_list` asks for one.
fn rows(cells: &'static [Cells]) -> impl Fn(usize, RowField) -> Option<&'static str> {
    move |index, field| {
        let (title, subtitle, value) = *cells.get(index)?;
        match field {
            RowField::Title => Some(title),
            RowField::Subtitle => subtitle,
            RowField::Value => value,
        }
    }
}

/// The metrics the gallery wires the X3 with.
fn metrics() -> Metrics {
    metrics_for(BOARD)
}

fn labels() -> Labels {
    Labels::for_panel(BOARD.width, BOARD.height)
}

/// Paints with `body` on a fresh panel, and keeps `rect` of it.
fn paint(name: &str, rect: Rect, body: impl FnOnce(&Metrics)) -> Result<(), String> {
    let backend = install();
    body(&metrics());
    let frame = backend.with_display(|panel| crop(panel, rect));
    check(name, &frame)
}

/// The hint bar, painted with `keys`, `labels` and four hints.
fn hints(name: &str, keys: &KeyRow, labels: &Labels, given: [Hint; 4]) -> Result<(), String> {
    let backend = install();
    let [back, confirm, previous, next] = &given;
    draw_button_hints(&metrics(), labels, keys, back, confirm, previous, next);
    hint_bar(backend, name)
}

/// A whole panel of chrome: a header, two groups, and the hint bar.
fn overview(m: &Metrics) {
    let side = m.content_side_padding;
    let width = BOARD.width - side * 2;
    let top = m.content_top();

    draw_header(m, Some("Settings"), Some("12:04"));

    draw_sub_header(
        m,
        Rect::new(side, top, width, m.sub_header_height),
        "Display",
        None,
    );
    let list_top = top + m.sub_header_height + m.spacing_small;
    let list_height = 3 * (m.list_row_height + m.list_row_gap) - m.list_row_gap;
    draw_list(
        m,
        Rect::new(side, list_top, width, list_height),
        3,
        0,
        &rows(&SETTINGS),
    );

    let reading = list_top + list_height + m.vertical_spacing;
    draw_sub_header(
        m,
        Rect::new(side, reading, width, m.sub_header_height),
        "Reading",
        Some("Chapter 3"),
    );
    let slider = reading + m.sub_header_height + m.spacing_small;
    draw_slider(
        m,
        Rect::new(side, slider, width, 40),
        25,
        100,
        ControlState::Idle,
    );
    let bar = slider + 40 + m.vertical_spacing;
    draw_progress_bar(
        m,
        Rect::new(side, bar, width, m.progress_bar_height),
        120,
        340,
    );

    let band = BOARD.height - m.button_hints_height - top;
    draw_scroll_indicator(m, Rect::new(0, top, BOARD.width, band), band * 2, band, 0);

    draw_button_hints(
        m,
        &labels(),
        &BOARD.keys,
        &Hint::Standard,
        &Hint::Standard,
        &Hint::Standard,
        &Hint::Standard,
    );
}

#[test]
fn the_pictures_on_the_chrome_reference_pages() {
    let mut failures = Failures::default();
    let m = metrics();
    let side = m.content_side_padding;
    let width = BOARD.width - side * 2;
    let top = m.content_top();
    let pad = m.spacing_small;
    // A crop of `height` pixels of content at the top of the content band.
    let band = |height: i32| Rect::new(side - pad, top - pad, width + 2 * pad, height + 2 * pad);

    // docs/reference/painting.md
    let backend = install();
    overview(&m);
    failures.note(whole(backend, "chrome_overview"));

    let header = Rect::new(0, 0, BOARD.width, m.top_padding + m.header_height + 1 + pad);
    failures.note(paint("chrome_header", header, |m| {
        draw_header(m, Some("Settings"), Some("12:04"));
    }));

    failures.note(paint("chrome_sub_header", band(m.sub_header_height), |m| {
        let rect = Rect::new(side, top, width, m.sub_header_height);
        draw_sub_header(m, rect, "Display", Some("3 settings"));
    }));

    let three = 3 * (m.list_row_height + m.list_row_gap) - m.list_row_gap;
    failures.note(paint("chrome_list", band(three), |m| {
        draw_list(
            m,
            Rect::new(side, top, width, three),
            3,
            1,
            &rows(&SETTINGS),
        );
    }));

    let two_tall = 2 * (m.list_row_height_with_subtitle + m.list_row_gap) - m.list_row_gap;
    failures.note(paint("chrome_list_subtitle", band(two_tall), |m| {
        draw_list(
            m,
            Rect::new(side, top, width, two_tall),
            2,
            0,
            &rows(&NETWORKS),
        );
    }));

    // Room for two rows and most of a third, outlined so the space the list
    // leaves shows.
    let short = 2 * (m.list_row_height + m.list_row_gap) + m.list_row_height / 2;
    failures.note(paint("chrome_list_fit", band(short), |m| {
        let rect = Rect::new(side, top, width, short);
        draw_list(m, rect, LIBRARY.len(), -1, &rows(&LIBRARY));
        Renderer::stroke_rect(rect);
    }));

    let backend = install();
    draw_header(&m, Some("Settings"), None);
    draw_list(
        &m,
        Rect::new(side, top, width, three),
        3,
        1,
        &rows(&SETTINGS),
    );
    draw_button_hints(
        &m,
        &labels(),
        &BOARD.keys,
        &Hint::Standard,
        &Hint::Standard,
        &Hint::Standard,
        &Hint::Standard,
    );
    draw_option_popup(
        &m,
        "Sleep after",
        &|index| DELAYS.get(index).copied(),
        DELAYS.len(),
        1,
    );
    failures.note(whole(backend, "chrome_option_popup"));

    let backend = install();
    draw_option_popup(
        &m,
        "Language",
        &|index| LANGUAGES.get(index).copied(),
        LANGUAGES.len(),
        4,
    );
    failures.note(whole(backend, "chrome_popup_clamped"));

    let gap = 2 * m.spacing_small;
    let sliders = 3 * 40 + 2 * gap;
    failures.note(paint("chrome_slider", band(sliders), |m| {
        let states = [
            ControlState::Idle,
            ControlState::Focused,
            ControlState::Editing,
        ];
        for (index, state) in states.into_iter().enumerate() {
            let y = top + index as i32 * (40 + gap);
            draw_slider(m, Rect::new(side, y, width, 40), 40, 100, state);
        }
    }));

    let bar = m.progress_bar_height;
    let bars = 3 * bar + 16 + 3 * gap;
    failures.note(paint("chrome_progress_bar", band(bars), |m| {
        let mut y = top;
        for (current, height) in [(0, bar), (120, bar), (340, bar), (120, 16)] {
            draw_progress_bar(m, Rect::new(side, y, width, height), current, 340);
            y += height + gap;
        }
    }));

    let window = 4 * (m.list_row_height + m.list_row_gap) - m.list_row_gap;
    failures.note(paint("chrome_scroll_indicator", band(window), |m| {
        let beside = m.scrollbar_width + m.scrollbar_inset * 2;
        let list = Rect::new(side, top, width - beside, window);
        draw_list(m, list, LIBRARY.len(), 0, &rows(&LIBRARY));
        let stride = m.list_row_height + m.list_row_gap;
        let content = LIBRARY.len() as i32 * stride * 2;
        draw_scroll_indicator(
            m,
            Rect::new(side, top, width, window),
            content,
            window,
            stride,
        );
    }));

    let standard = [
        Hint::Standard,
        Hint::Standard,
        Hint::Standard,
        Hint::Standard,
    ];
    failures.note(hints(
        "chrome_button_hints",
        &BOARD.keys,
        &labels(),
        standard,
    ));
    failures.note(hints(
        "chrome_button_hints_open",
        &BOARD.keys,
        &labels(),
        [Hint::Cancel, Hint::Done, Hint::Standard, Hint::Standard],
    ));
    let three_keys = KeyRow::new(&[RowKey::Back, RowKey::Confirm, RowKey::Unassigned]);
    failures.note(hints(
        "chrome_button_hints_keys",
        &three_keys,
        &Labels::ENGLISH_NARROW,
        [
            Hint::Standard,
            Hint::Standard,
            Hint::Standard,
            Hint::Standard,
        ],
    ));

    // docs/reference/icons.md
    let icons = Rect::new(
        side - pad,
        top - pad,
        12 * 40 + 2 * pad - 8,
        2 * 40 + 2 * pad - 8,
    );
    failures.note(paint("chrome_icons", icons, |_| {
        for (column, icon) in Icon::ALL.into_iter().enumerate() {
            for (row, variant) in [0, 1].into_iter().enumerate() {
                let origin = Point::new(side + column as i32 * 40, top + row as i32 * 40);
                draw_icon(
                    origin,
                    IconRef {
                        variant,
                        ..icon.into()
                    },
                );
            }
        }
    }));

    let sizes = [8, 16, 24, 32, 48];
    let row = 48;
    let across: i32 = sizes.iter().map(|s| s + 16).sum::<i32>() - 16;
    let sized = Rect::new(
        side - pad,
        top - pad,
        across + 2 * pad,
        2 * row + 16 + 2 * pad,
    );
    failures.note(paint("chrome_icon_sizes", sized, |_| {
        for (line, icon) in [Icon::Book, Icon::Sun].into_iter().enumerate() {
            let bottom = top + row + line as i32 * (row + 16);
            let mut x = side;
            for size in sizes {
                let reference = IconRef {
                    size,
                    ..icon.into()
                };
                draw_icon(Point::new(x, bottom - size), reference);
                x += size + 16;
            }
        }
    }));

    failures.finish();
}
