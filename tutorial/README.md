[![CI](https://github.com/XPUI-Framework/xpui-gallery/actions/workflows/ci.yml/badge.svg)](https://github.com/XPUI-Framework/xpui-gallery/actions/workflows/ci.yml) [![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](../LICENSE)

# `xpui-tutorial`

The screen the framework's tutorial builds, finished and running.

If you are following
[**the tutorial**](https://github.com/XPUI-Framework/xpui-framework/blob/main/docs/tutorial.md),
this is where it ends up — and it is here rather than in the framework because
the framework has nothing to draw with. That is the dependency rule: `xpui`
depends on nothing, so the crate that owns the tutorial cannot open a window to
show you the result.

## Using it

```bash
cargo run -p xpui-tutorial            # in a window
cargo test -p xpui-tutorial           # eleven tests, no window
```

**The crate exists so the tutorial cannot rot.** Its eight `rust` blocks are
doctests in the framework, which proves they compile; this crate proves the
finished screen *works* — that its rows respond, its dialog opens, its Back
does what the page says. Eleven tests: nine driving the screen the way a
person does, and two comparing the painted frame against a committed image.
Change the tutorial's API and the framework's doctests fail; change what the
screen *does* and these fail. Neither catches the other.

`src/main.rs` differs from the page in three ways, none of them about the
framework: it keeps the builder in a `let` so a `--frames` flag can drive it
headlessly, gives the window this repository's title rather than the page's,
and passes `SleepTimer::new()` because by step 8 the screen has state.

## Checking it

The gate is the repository's; run `./build-and-test.sh` from the root.

## License

MIT — see [LICENSE](../LICENSE). Copyright (c) 2026 Thiago Holanda.
