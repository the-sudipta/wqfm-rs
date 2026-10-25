#!/bin/bash
# Compares wqfm-rs with the wQFM v1.4 jar on every .wqrts file of a folder.
# The jar runs in WSL (Java) with a memory cap, one input at a time.
#
# Usage: bash scripts/jar_compare.sh <inputs folder> <wQFM-2020 folder> [java heap, default 512m]
# Writes <inputs folder>/comparison.tsv (one row per input) and prints a
# summary. Pass = identical topology (RF 0) and identical score.
# Environment: JAVA = java command inside WSL (default java); TAG = suffix
# of the jar trees and the table, to keep runs with two Java versions apart;
# BETA = partition-score beta passed to both programs (default 1).

set -u
dir=$(cd "$1" && pwd)
wqfm_dir=$(cd "$2" && pwd)
heap=${3:-512m}
java=${JAVA:-java}
tag=${TAG:-}
beta=${BETA:-1}
root=$(cd "$(dirname "$0")/.." && pwd)
rs="$root/target/release/wqfm-rs"
[ -x "$rs" ] || rs="$rs.exe"
to_wsl() { wsl.exe -d Ubuntu-24.04 -- wslpath -a "$(cygpath -m "$1")" | tr -d '\r'; }
wqfm_wsl=$(to_wsl "$wqfm_dir")
dir_wsl=$(to_wsl "$dir")

out="$dir/comparison$tag.tsv"
printf 'name\trs_score\tjar_score\ttotal\trf\tnrf\trs_seconds\tjar_seconds\tresult\n' > "$out"
pass=0
fail=0
for f in "$dir"/*.wqrts; do
    name=$(basename "$f" .wqrts)
    t0=$(date +%s.%N)
    "$rs" amalgamate --beta "$beta" -i "$f" -o "$dir/$name.rs$tag.tre" 2> /dev/null
    t1=$(date +%s.%N)
    wsl.exe -d Ubuntu-24.04 -- bash -c "cd '$wqfm_wsl' && timeout 600 $java -Xmx$heap -jar wQFM-v1.4.jar -beta $beta -i '$dir_wsl/$name.wqrts' -o '$dir_wsl/$name.jar$tag.tre'" > "$dir/$name.jar$tag.log" 2>&1
    t2=$(date +%s.%N)
    rs_s=$(awk -v a="$t0" -v b="$t1" 'BEGIN { printf "%.3f", b - a }')
    jar_s=$(awk -v a="$t1" -v b="$t2" 'BEGIN { printf "%.3f", b - a }')
    if [ ! -s "$dir/$name.jar$tag.tre" ]; then
        printf '%s\t\t\t\t\t\t%s\t%s\tjar failed\n' "$name" "$rs_s" "$jar_s" >> "$out"
        fail=$((fail + 1))
        continue
    fi
    read -r rs_score total _ < <("$rs" score -i "$f" -t "$dir/$name.rs$tag.tre")
    read -r jar_score _ _ < <("$rs" score -i "$f" -t "$dir/$name.jar$tag.tre")
    read -r rf nrf < <("$rs" compare "$dir/$name.rs$tag.tre" "$dir/$name.jar$tag.tre")
    if [ "$rf" = 0 ] && [ "$rs_score" = "$jar_score" ]; then
        result=pass
        pass=$((pass + 1))
    else
        result=FAIL
        fail=$((fail + 1))
    fi
    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$name" "$rs_score" "$jar_score" "$total" "$rf" "$nrf" "$rs_s" "$jar_s" "$result" >> "$out"
done
echo "$pass passed, $fail failed of $((pass + fail)) inputs ($out)"
[ "$fail" = 0 ]
