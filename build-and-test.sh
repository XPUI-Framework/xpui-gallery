#!/usr/bin/env bash

# Everything CI checks in this repository, in one command.
#
#   ./build-and-test.sh          format, lint, test, the seven boards, and every snippet
#   ./build-and-test.sh check    the same thing; the name CI uses
#   ./build-and-test.sh fix      format Rust and C++ in place first
#
# **Half of what runs is in `bin/gate-common.sh`**, of which every repository
# in the organisation carries a byte-identical copy. This file is what this
# repository configures, what only it checks, and the order they run in.
# `xpui-dev` compares the nine copies and runs all nine gates.

set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "${PROJECT_DIR}"

# The reference application, and the screen the tutorial builds.
SOURCE_ROOTS=(gallery tutorial)

# The gallery's tests install the fake host to assert what was painted.
TEST_FEATURES="xpui/testing"

# **There is device code here**, and it is linted for both bare-metal targets
# because nothing else does. `gallery/src/lib.rs` is
# `cfg_attr(target_os = "none", no_std)` and the simulator dependency is gated
# on `cfg(not(target_os = "none"))`, so the configuration both firmwares
# actually link is a different compilation from the host one. Before this line
# it was reached only by `./build-and-test.sh all` in the two firmware
# repositories, against `branch = "main"` from git rather than the local tree —
# so a `no_std` break here was invisible until somebody built a firmware.
#
# `tutorial` is host-only and named out: it exists to snapshot the screen the
# framework's tutorial builds, which needs a window.
HOST_WORKSPACE=1
LINT_TARGETS=("riscv32imc-unknown-none-elf" "thumbv6m-none-eabi?")
# `--lib` because the binary is the simulator entry point and needs `std`;
# what a firmware links is the library, and that is what has to compile here.
LINT_TARGET_CRATES=(-p xpui-gallery --lib)

. bin/gate-common.sh

gates() {
  file_sizes
  every_check_runs
  readmes_warn
  prose_is_compiled
  doc_paths
  cpp_snippets_compile
  lint
  test_suite
  doc_tests
  doc_links
}

case "${1:-check}" in
  check)
    run_all "${FORMAT_CHECK[@]}"
    gates
    printf '\nChecks passed.\n'
    ;;
  fix)
    run_all "${FORMAT_FIX[@]}"
    gates
    printf '\nFormatted and checked.\n'
    ;;
  *)
    echo "usage: ./build-and-test.sh [check|fix]" >&2
    exit 2
    ;;
esac
