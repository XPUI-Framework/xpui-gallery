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

# Host only. Both firmwares depend on this crate and each builds it for its own
# device; nothing here is device code, so there is no bare-metal run.
HOST_WORKSPACE=1

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
    rust_format_check
    cpp_format_check
    gates
    printf '\nChecks passed.\n'
    ;;
  fix)
    rust_format_fix
    cpp_format_fix
    gates
    printf '\nFormatted and checked.\n'
    ;;
  *)
    echo "usage: ./build-and-test.sh [check|fix]" >&2
    exit 2
    ;;
esac
