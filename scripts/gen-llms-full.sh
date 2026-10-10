#!/bin/bash
# Generate llms-full.txt from llms.txt, the README and the examples.
# Run with --check to fail if the committed file is out of date.
set -euo pipefail

cd "$(dirname "$0")/.."

OUT="llms-full.txt"
EXAMPLES=(
    examples/hello_world/src/main.rs
    examples/todo_api.rs
    examples/auth_api.rs
    examples/catalog_api.rs
)

generate() {
    cat llms.txt
    printf '\n---\n\n# README\n\n'
    # Skip the banner/badges block before the first `---`, and the
    # contributor-only sections from `## Development` onwards
    awk '
        !body && /^---$/ { body = 1; next }
        /^## Development$/ { exit }
        body
    ' README.md
    for example in "${EXAMPLES[@]}"; do
        printf '\n---\n\n# %s\n\n```rust\n' "$example"
        cat "$example"
        printf '```\n'
    done
}

if [ "${1:-}" = "--check" ]; then
    if ! diff -u "$OUT" <(generate) >/dev/null; then
        echo "error: $OUT is out of date, run 'just llms' to regenerate it" >&2
        exit 1
    fi
else
    generate >"$OUT"
    echo "Wrote $OUT"
fi
