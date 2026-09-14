# Design decisions

Where a comment in the code says what was decided, this is why. One section
per decision, named for the item that carries the sentence.

## A double press stands in for Back (`gallery::chord`)

A board with three keys along its bottom edge and no spare has nowhere to put
Back, so `chord::Badge` reads two quick presses of the first key as one. A key
that might yet turn out to be half of a double press cannot act until it is
certain it was not, so **every select on the stand-in key waits out
`DOUBLE_PRESS_MS`**. On a panel that takes most of a second to refresh the wait
is lost in it; on an immediate LCD it is a third of a second of nothing.

Only the stand-in key pays it. Every other key on the row acts on the frame it
was pressed, and a board with a Back key of its own never enters the
arrangement at all — which is why `back_stands_on` asks the row whether it
*has* a Back key rather than counting its keys: a board with three keys along
the bottom and an up/down pair elsewhere gives one to Back, and counting would
still charge it the delay.

Before choosing this for a board, check whether it has a key to spare. A delay
on the one action a person takes most often is a poor trade for a button that
was already there. No board in `gallery::boards::ALL` is arranged this way; the
[Badger](https://shop.pimoroni.com/products/badger-2040) and the [Tufty](https://shop.pimoroni.com/products/tufty-2040) each have an up/down pair and spend the first key on Back.

## Two extra families cost 99.8 KB (`gallery::fonts`)

Each family in `gallery::fonts` is twelve faces of bitmaps in `.rodata`. Built
for a Badger 2040 and measured with `arm-none-eabi-size` on the linked
`badger2040` binary, shortening `FAMILIES` and rebuilding:

| what `FAMILIES` holds | firmware |
|---|---|
| `&[&HELVETICA]` — the backend's own, alone | 121 KB |
| all three | 221 KB |

The absolute figures move with every change to the firmware; the difference
between the rows is the number that means something, and it has held at
99.8 KB (the rows are rounded to the kilobyte). The two extra families nearly double the firmware: 5% of a 2 MB
[RP2040](https://www.raspberrypi.com/products/rp2040/)'s flash and affordable there, impossible on a part with 256 K. That is
why the backend ships one family and takes whatever else it is given — a board
that cannot spare the space shortens `FAMILIES` and pays for nothing it does
not use, because the two families are reachable from nowhere else and dropping
them from that list drops their bitmaps from the binary.

## The eight-board suite is here, not in a backend (`gallery::boards`)

A per-board regression is the easiest kind to ship and the hardest to see:
the suite is green, the board you looked at is right, and two of the other
seven are broken. That has happened — a change to what the [Pimoroni](https://shop.pimoroni.com/) boards
*paint* left what they *send* alone, so every hint label sat one key off and
no key produced Back, on two boards, with 169 tests passing.

The suite lives here because this is where a board and a backend actually
meet. `gallery::boards::ALL` is composed here from the three vendor crates —
there is deliberately none across vendors below this level — and a `const`
assertion fails if a vendor gains or loses a board and this list does not
follow.

## A library and a binary (`gallery`)

The screens live in `src/`, and `main.rs` does nothing but read its two flags
and open a window around them. That is so the tests can drive every screen
without one: `tests/gallery.rs` walks the menu, opens each example, presses
buttons and checks what came back; `tests/screenshots.rs` renders the same
screens to a framebuffer, once per board, and compares each against a
committed PNG. It is also what lets two firmwares link the screens without
dragging a window in.

## The framework's dogfood (`gallery::screens`, `::typeface`, `::developers`)

If a screen here needs a workaround, `xpui` has a gap. Two things in these
files exist because of that:

- **`format!` belongs out of `body()`.** `body()` runs on every paint *and*
  every frame carrying input, so a string is built when its value changes and
  kept: `Typefaces` builds its size labels in `new`, the tutorial's
  `SleepTimer` rebuilds one in `update`, and a control that shows its own
  number, like `Stepper`, takes `.readout("%")` and formats nothing. A
  `format!` in `body()` allocates several times a second and drags
  `core::fmt` into the binary.

  **`DevelopersScreen` breaks this rule and is the reason it is written
  down.** Its five memory rows call `Units::format` from inside `body()`, so
  opening Developers formats five values and allocates about fifteen times on
  every paint. On a laptop nothing shows; on a Badger it is the screen that
  would. Nothing catches it — `xpui`'s counting allocator is `cfg(test)`
  inside `xpui`, so no test here can assert it.
- **Every screen wraps itself in a `NavigationScreen`.** Without one there is
  no header and no button hints, which looks broken the moment you open it
  from the menu rather than on its own.
