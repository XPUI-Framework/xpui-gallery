# The seven-board conformance suite

`--board` changes the panel size, the chrome sized for it, and the window
scale — and nothing else. The screens are untouched, which is the claim the
framework makes and the one this suite checks, pixel for pixel, on every
panel it describes.

## What it captures

`gallery/tests/screenshots.rs` renders **every screen on every board**: the
seven examples, plus the menu, the picker open over its content, and the
Controls screen with a value open for editing, across the seven panels of
`gallery::boards::ALL` — seventy committed PNGs under
`gallery/tests/screenshots/`, named `<screen>_<board slug>.png`.
`gallery/tests/typeface.rs` adds three more, `family_<name>.png`, in the same
directory, which therefore holds seventy-three files; and
`gallery/tests/gallery.rs` five text snapshots of the draw calls under
`gallery/tests/snapshots/`. The tutorial crate has two screenshots of its
own, under `tutorial/tests/screenshots/`.

Beside each capture the suite asserts that the content band is not blank —
with the *default* chrome a [Badger](https://shop.pimoroni.com/products/badger-2040)'s is 28 pixels and a list refuses to paint
a row that does not fit, so the screen comes back empty — and asks for the
screen's name in the header band and for ink in the hint bar on the boards
that have one. Neither is asked of every capture, and the file says which
and why: the picker's header is under a dialog, so the question would pass
there whatever the dialog did.

The board-by-board captures are what catch a fault only one panel has.
Moving `Metrics::SMALL.list_row_height` by one pixel moves seven of them and
fails no other test in the repository. That file's own module doc counts two
more such mutations and says which boards each one reaches. Goldens are compared by
[`xpui-screenshot`](https://github.com/XPUI-Framework/xpui-backends/tree/main/screenshot),
pixel for pixel with no tolerance.

## Running it

```bash
cargo test --workspace --features xpui/testing
```

No window opens. A mismatch writes `expected`, `actual` and `differences`
side by side into `target/diff/<name>.png`, so a failure — on CI too, where
the workflow uploads that directory — can be looked at rather than guessed
at.

## Reading a failure

Open `target/diff/<name>.png`. Three panels side by side, separated by thin
rules: the committed golden, what was painted, and a mark on every pixel that
differs.
Then ask which of the three changed on purpose: the screen, the chrome, or
the board. A one-pixel shift in one board's capture and no other is a board
fault; the same shift in all seven is a screen or a chrome change.

## Re-blessing

A golden that does not exist yet is written, and then the test fails: nobody
commits a picture they have never looked at. Accept an intended change with:

```bash
UPDATE_SNAPSHOTS=1 cargo test --workspace --features xpui/testing
```

That rewrites **every** golden the run touched — seventy board captures,
three families, five draw-call snapshots, the tutorial's two — in one
keystroke. **A blessed golden is an assertion you have made.** Open
`gallery/tests/screenshots/`, read the diff, and only then stage it.

The comparison is one call, and the name is the file the golden lives under.
The suite calls `check_screenshot`, which returns a mismatch rather than
panicking, so one run names every board that moved; a test with a single
capture calls `assert_screenshot`, which panics on the first:

```rust,no_run
# use xpui_screenshot::Framebuffer;
# fn compare(framebuffer: &Framebuffer) {
// Against `tests/screenshots/menu_x3.png`; on a mismatch, writes
// `target/diff/menu_x3.png` and panics.
xpui_screenshot::assert_screenshot("menu_x3", framebuffer);
# }
```

## Every board, every time

There are seven boards, and a change is not finished until all seven are
checked. Why the suite lives here rather than in a backend, and the
per-board regression that has already shipped once, are in
[design.md](design.md). Any test about layout, chrome or keys belongs in the
shape `screenshots.rs`, `row_overflow.rs` and `boards.rs` already have —
walking `gallery::boards::ALL` — rather than pinned to one board.
