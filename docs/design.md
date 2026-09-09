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
Badger and the Tufty each have an up/down pair and spend the first key on Back.

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
RP2040's flash and affordable there, impossible on a part with 256 K. That is
why the backend ships one family and takes whatever else it is given — a board
that cannot spare the space shortens `FAMILIES` and pays for nothing it does
not use, because the two families are reachable from nowhere else and dropping
them from that list drops their bitmaps from the binary.
