#!/usr/bin/env bash
set -euo pipefail

if ! command -v cargo >/dev/null 2>&1; then
  if command -v cygpath >/dev/null 2>&1 && [[ -n "${USERPROFILE:-}" ]]; then
    export PATH="$(cygpath -u "$USERPROFILE")/.cargo/bin:$PATH"
  fi
  if [[ -n "${HOME:-}" ]]; then
    export PATH="$HOME/.cargo/bin:$PATH"
  fi
fi

cargo test --locked
node --test --test-concurrency=1 tests/node/*.test.js
