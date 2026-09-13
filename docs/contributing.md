# Contributing to `xpui-gallery`

## Building it

SDL2 first: the simulator links it, and the workspace does not build without
it — `brew install sdl2` on macOS, `sudo apt install libsdl2-dev` on Debian and
Ubuntu. Then `rust-toolchain.toml` pins the rest, and every other dependency
is a sibling repository fetched on `main`.

```bash
cargo test --workspace --features xpui/testing   # the suite, headless
./build-and-test.sh                              # everything CI checks
```

## The gate

A change is not finished until `./build-and-test.sh` passes. It is the same
command CI runs, so a green run locally means what a green tick means there.
The checks are listed in [`AGENTS.md`](../AGENTS.md) and implemented in
[`xtask/`](../xtask/); `./build-and-test.sh fix` formats in place first.

## Every board, every time

There are seven boards, and a change is not finished until all seven are
checked — [`docs/design.md`](design.md) says what the regression this catches
looks like, and [`docs/conformance.md`](conformance.md) how to run and read
the suite. Any test about layout, chrome or keys walks `gallery::boards::ALL`
rather than pinning one board.

```bash
cargo run -p xpui-gallery -- --board x3        # and x4, x4pro, sticky,
                                               # badger2040, tufty2040, inkyframe
```

The goldens under `gallery/tests/screenshots/` are assertions. A first run of
a new one writes it and fails; `UPDATE_SNAPSHOTS=1` accepts a change, and
[conformance.md](conformance.md) says how to read a diff before you do. A
documentation change never moves a golden; if one moved, something else did.

## The review

Five steps, in order, none skipped:

1. The gate passes, with the real exit code read.
2. The [code-reviewer](../.claude/agents/code-reviewer.md) agent reviews the
   change — every finding resolved, not noted.
3. The [docs-reviewer](../.claude/agents/docs-reviewer.md) agent reviews the
   prose, last: it runs every command a document gives and resolves every
   snippet against the API.
4. The author opens the window on every board and looks. That is their step;
   an agent never opens the simulator to report what it saw.
5. They say commit.

A test that cannot fail is worse than no test. Before adding one, break the
code on purpose and confirm the test notices. Prefer an assertion that pins a
relationship over one that pins a number.

## Commits

The subject says what was done — imperative, under fifty characters, one
concern. The body says what changed and why, in under about ten lines,
carrying the fact that is not in the diff. Nothing about how the bug was
found. No self-attribution.

## Working across the repositories

Both firmwares depend on `gallery` through a `git` dependency on `main`, and
this repository depends on five siblings the same way. Before pushing a
change, run the umbrella:

```bash
for d in ../xpui*/; do git -C "$d" fetch --quiet --all; done
cd ../xpui-dev && ./build-and-test.sh cross
```

It builds every crate from the sibling checkouts on disk and says which one
broke. `cross` is that repository's gate, not this one's — run it from there,
not here. The fetch first, because its link check resolves every
`github.com/XPUI-Framework/…` URL against each sibling's `origin/main`, and a
stale remote is a stale answer. `xpui`'s [`docs/orientation.md`](https://github.com/XPUI-Framework/xpui-framework/blob/main/docs/orientation.md)
describes the layout it expects.
