# `xpui-gallery`

## What this is, and what it may not become

The application layer: the reference screens `xpui` is demonstrated with, in
a window on a desktop and as the library both firmwares flash, and the
seven-board conformance suite that proves the framework paints the same thing
on every panel it describes. It assembles the seven boards from three vendor
crates, because nothing below it may know more than one vendor, and it wires
a backend the way a firmware does.

**It is where screens live and where framework gaps show, never where they
are fixed.** A screen that needs a workaround means `xpui` has a gap; the fix
goes there. Nothing here is device-specific — a board is data handed in, and
the same screens run on all seven — and no framework logic, no chrome and no
backend code belongs in this repository.

## The gate

```bash
./build-and-test.sh          # everything below
./build-and-test.sh fix      # the same, formatting in place first
```

```text
format · file sizes · crates are tested · READMEs warn · prose is compiled · documented paths resolve · rustdoc links resolve · the reference mirrors rustdoc · documented commands resolve · lint · tests · doctests · README sections · AGENTS.md · published crates deny missing_docs · comment blocks · comment narration
```

There is no `all` mode; this list is the whole of it, and a last stage,
`the gate is documented`, compares it to what ran. Run it before saying a
change is done, and read the real exit code.

## What only this repository checks

The seven-board conformance suite, under `tests`: seventy board captures,
three typeface captures and five draw-call snapshots, compared pixel for
pixel and byte for byte. The two bare-metal clippy runs under `lint` compile
the `gallery` library alone (`LINT_CRATES`), the shape a firmware links.
`published crates deny missing_docs` prints `no publishable crates`, and that
is this repository's permanent truth: both crates are `publish = false`, and
`#![deny(missing_docs)]` is on anyway.

## Style that bites here

- **Every board, every time.** A change to layout, chrome or keys is checked
  on all seven; a test about any of them walks `gallery::boards::ALL`.
- **A blessed golden is an assertion you have made.** `UPDATE_SNAPSHOTS=1`
  rewrites every golden the run touched; read the diff before staging, and
  never bless from a documentation change.
- **`format!` belongs out of `body()`.** It runs on every paint and every
  frame carrying input; a string is built when its value changes and kept.
  `DevelopersScreen`'s memory rows break this and are the known exception —
  `docs/design.md` says what it costs.
- **Every screen is a `NavigationScreen`**, or it has no header and no hints
  the moment it is opened from the menu.
- **No test opens a window.** The simulator loop runs headless with
  `--frames`; an agent never opens the window to report what it saw.
- **`no_std` on device.** The `gallery` library builds for bare metal;
  `alloc::` explicitly, and `std` only in `main.rs` and the tests.
- **Every `pub` item is documented**, both crates.
- **A file under `src/` is at most 400 lines.**

## Where the documentation lives, and what proves each piece

| Document | Proven by |
|---|---|
| [`README.md`](README.md) | its `rust` fence is a doctest, mounted by `gallery/src/lib.rs`; its paths and commands resolve |
| [`docs/README.md`](docs/README.md) | its paths resolve; the README-heading check exempts it, because it is the index of `docs/`, not a front page |
| [`gallery/README.md`](gallery/README.md), [`tutorial/README.md`](tutorial/README.md) | their paths and commands resolve; neither carries a `rust` fence |
| [`docs/conformance.md`](docs/conformance.md) | its `rust` fence is a doctest, mounted by `gallery/src/lib.rs`; its commands resolve |
| [`docs/design.md`](docs/design.md) | mounted by `gallery/src/lib.rs`; it carries no `rust` fence, so what is checked is its paths |
| [`docs/contributing.md`](docs/contributing.md) | every path and command it gives resolves; the umbrella command is `xpui-dev`'s |
| `xpui`'s `docs/tutorial.md` | its finished screen is `tutorial/`, driven by eleven tests here |
| `AGENTS.md` | the stage list above is compared to what the gate runs |
| every `///` and `//!` | `rustdoc links resolve`, and the two comment checks |

## Git

Never stage, never commit, never push without being asked, each time. The
index is the reviewer's queue; leave new work unstaged. No self-attribution
in a commit message. Never rewrite a commit that exists; a correction is a new
commit. The rules that apply to all ten repositories, and the five review
steps, are in [`xpui`'s `docs/orientation.md`](https://github.com/XPUI-Framework/xpui-framework/blob/main/docs/orientation.md);
how a change is built and reviewed here is in
[`docs/contributing.md`](docs/contributing.md).
