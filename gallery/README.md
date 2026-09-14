[![CI](https://github.com/XPUI-Framework/xpui-gallery/actions/workflows/ci.yml/badge.svg)](https://github.com/XPUI-Framework/xpui-gallery/actions/workflows/ci.yml) [![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](../LICENSE)

# The gallery

Example screens built on [`xpui`](https://github.com/XPUI-Framework/xpui-framework),
in a window, and the library both firmwares flash.

## Using it

```bash
cargo run -p xpui-gallery
cargo run -p xpui-gallery -- --board badger2040    # a 296x128 e-ink strip
cargo run -p xpui-gallery -- --board tufty2040     # a 320x240 colour LCD
```

Arrows move focus, Enter opens, Backspace goes back, H is the home gesture,
`B` walks the eight boards, Q or Escape quits. Clicking is a tap and the
scroll wheel is a swipe.

| Example | Shows |
|---|---|
| Controls | Slider, stepper, toggle, progress bar |
| Lists | Rows, subtitles, values, and how a subtitle changes row height |
| Dialogs | A picker over content, capturing input, dimming what is behind |
| Scrolling | More rows than fit, and the runtime keeping focus visible |
| Text | Font roles and weights, and a line running off the panel — `Text` paints what it is given |
| Typeface | Three families, and changing the one everything is set in |
| Developers | Everything at once on one screen: heap figures, a widget bench, a scrolling section |

The badges have three keys along the bottom edge — `a`, `b`, `c` — and a
dedicated up/down pair beside the panel, so **`a` is Back and `b` confirms**,
in the order the framework's own row has: a key means the same thing here as
on a reader. `c` has nothing on it. `src/chord.rs` reads two presses as one
meaning for a board with three keys and no spare — a real shape no board here
has — and [`../docs/design.md`](../docs/design.md) says what that costs.

## Checking it

The gate is the repository's; run `./build-and-test.sh` from the root. This
crate's `tests/` are the eight-board conformance suite;
[`../docs/conformance.md`](../docs/conformance.md) is how to read a failure
and re-bless.

## Where next

| | |
|---|---|
| [`../docs/conformance.md`](../docs/conformance.md) | every screen on every board, and the rule that a bless is an assertion you have made |
| [`../docs/design.md`](../docs/design.md) | why the suite is here, why this is a library, the double press, the cost of two typefaces |

## License

MIT — see [LICENSE](../LICENSE).
