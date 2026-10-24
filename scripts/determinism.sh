#!/bin/bash
# Writes the SHA-256 of wqfm-rs outputs on a fixed set of inputs, one line
# per file, sorted. The same list on every OS means identical results.
#
# Usage: bash scripts/determinism.sh [work folder]   (prints the list)
# CI compares the list with validation/determinism.sha256.

set -eu
root=$(cd "$(dirname "$0")/.." && pwd)
work=${1:-"$root/target/determinism"}
bin="$root/target/release"
rs="$bin/wqfm-rs"
gen="$bin/examples/make_inputs"
[ -x "$rs" ] || rs="$rs.exe"
[ -x "$gen" ] || gen="$gen.exe"
if command -v sha256sum > /dev/null; then sha() { sha256sum "$@"; }; else sha() { shasum -a 256 "$@"; }; fi

rm -rf "$work"
mkdir -p "$work/in" "$work/out"
"$gen" "$work/in" 6,10,20,30 1,2 > /dev/null
for f in "$work"/in/*.wqrts; do
    name=$(basename "$f" .wqrts)
    "$rs" amalgamate -i "$f" -o "$work/out/$name.tre" 2> /dev/null
done
for f in "$root"/crates/wqfm/tests/data/*.tre; do
    case "$f" in *.jar.tre) continue ;; esac
    name=$(basename "$f" .tre)
    "$rs" gt2wq -i "$f" -o "$work/out/$name.gt2wq" 2> /dev/null
    "$rs" gt2wq -i "$f" -o "$work/out/$name.keep.gt2wq" --keep-unresolved 2> /dev/null
done
cp "$work"/in/*.wqrts "$work/out/"
cd "$work/out"
for f in *; do
    printf '%s  %s\n' "$(sha < "$f" | cut -d ' ' -f 1)" "$f"
done | LC_ALL=C sort -k 2
