# The gallery

Example screens built on [`xpui`](../../crates/xpui/), in a window.

```bash
cargo run -p xpui-gallery
cargo run -p xpui-gallery -- --board badger2040    # a 296x128 e-ink strip
cargo run -p xpui-gallery -- --board tufty2040     # a 320x240 colour LCD
```

Arrows move focus, Enter opens, Backspace goes back, H is the home gesture, Q
or Escape quits. Clicking is a tap and the scroll wheel is a swipe.

## Three keys, and a spare

The badges have three keys along the bottom edge — `a`, `b`, `c` — and a
dedicated up/down pair beside the panel. Five keys for four jobs, so **`a` is
Back and `b` confirms**, in the order the framework's own row has always been:
a key means the same thing here as on a reader. `c` has nothing on it.

That was not always true. While a badge was taken to be three keys and nothing
else, Back had nowhere of its own and was folded into a double press of `a` —
which cost every select a wait to find out whether a second press was coming.
No board here is arranged that way now, and none pays that wait.

`src/chord.rs` still implements it, because a board with three keys and no
spare is a real shape and this is where a firmware would read two presses as
one meaning — the simulator delivers what the hardware sent, and what it
*means* is the firmware's decision. See it before choosing that arrangement for
a board of your own, and note what it costs.

## The same screens, on every panel

`--board` changes the panel size, the chrome sized for it, and the window
scale — and nothing else. The screens are untouched, which is the claim the
framework makes and the one worth checking by eye.

It is worth checking early. A Badger 2040 has a 90-pixel content band; a
screen that looks spacious at 480 × 800 may have nowhere to put its third row.
`tests/screenshots.rs` renders the menu on every board and asserts the content
band is not empty, because with the *default* chrome on that panel it would
be — 28 pixels, and a list refuses to paint a row that does not fit.

## What is in it

| Example | Shows |
|---|---|
| Controls | Slider, stepper, toggle, progress bar |
| Lists | Rows, subtitles, values, and how a subtitle changes row height |
| Dialogs | A picker over content, capturing input, dimming what is behind |
| Scrolling | More rows than fit, and the runtime keeping focus visible |
| Text | Font roles, weights, and truncation on a character boundary |
| Typeface | Three families, and changing the one everything is set in |
| Developers | Everything at once on one screen, ported from CrossPoint |

## Why it is a library and a binary

The screens live in `src/`, and `main.rs` does nothing but read its two flags
and open a window around them. That is so the tests can drive every screen
without one:
`tests/gallery.rs` walks the menu, opens each example, presses buttons and
checks what came back; `tests/screenshots.rs` renders the same screens to a
framebuffer and compares each one against a PNG committed in
`tests/screenshots/`, pixel for pixel.

```bash
cargo test -p xpui-gallery
UPDATE_SNAPSHOTS=1 cargo test -p xpui-gallery   # accept intended changes
open examples/gallery/tests/screenshots/        # then look at them
open target/diff/                               # after a failure
```

## It is also the framework's dogfood

If a screen here needs a workaround, `xpui` has a gap. Two things in these
files exist because of that:

- **`format!` is not in `body()`.** `body()` runs on every paint *and* every
  frame carrying input, so `Controls` formats its percentages in `update()` and
  keeps the strings. A `format!` in `body()` allocates several times a second
  and drags `core::fmt` into the binary.
- **Every screen wraps itself in a `NavigationScreen`.** Without one there is
  no header and no button hints, which looks broken the moment you open it from
  the menu rather than on its own.
