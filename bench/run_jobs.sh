#!/bin/bash
# Runs the benchmark jobs one at a time (run in WSL, nothing else running).
# Restartable: a job with a result is skipped. Every run is timed with GNU
# time (wall seconds, peak resident memory) under the limits of config.sh.
#
# Usage (WSL): bash bench/run_jobs.sh <jobs.tsv>
# Results: $RESULTS/results.tsv (one row per job) and $RESULTS/runs/<job>/
# (output tree, log, time). Status: ok, time limit, file limit, memory
# limit, error, or "not run: limit reached on a smaller input".

set -u
source "$(dirname "$0")/config.sh"
JOBS=$1
RS=$TOOLS/wqfm-rs
PIPE=$TOOLS/wqfm-pipeline
mkdir -p "$RESULTS/runs"
OUTTSV=$RESULTS/results.tsv
[ -s "$OUTTSV" ] || printf 'job\tstatus\texit\tseconds\tmax_rss_kb\toutput_sha256\tquartets_sha256\tscore\ttotal_weight\tquartets\n' > "$OUTTSV"
GEN=$BENCH/s1_tmp
current_gen=""

done_already() { awk -F'\t' -v j="$1" '$1 == j { found = 1 } END { exit !found }' "$OUTTSV"; }
# Amendment 1: a failed limit on a smaller input of the same family stops
# the method on larger inputs.
stopped() { # method family size (only S1 by kind and G3, as the amendment says)
    [[ $2 == S1-* || $2 == G3 ]] || return 1
    awk -F'\t' -v m="$1" -v f="$2" -v s="$3" '
        NR == FNR { if ($2 ~ /limit$/) failed[$1] = 1; next }
        FNR > 1 && ($1 in failed) && $5 == m && $2 == f && $3 + 0 < s + 0 { hit = 1 }
        END { exit !hit }' "$OUTTSV" "$JOBS"
}

command_for() { # method input output workdir
    local m=$1 in=$2 out=$3 w=$4
    case $m in
    wqfm-rs) echo "'$RS' amalgamate -i '$in' -o '$out'" ;;
    wqfm-jar) echo "cd '$PIPE' && java -Xmx$JAVA_HEAP -jar wQFM-v1.4.jar -i '$in' -o '$out'" ;;
    wqfm-pipeline-java) echo "cp '$PIPE/generate_wqrts.py' '$PIPE/triplets.soda2103' '$w/' && cd '$w' && python3 generate_wqrts.py '$in' '$w/quartets.txt' && cd '$PIPE' && java -Xmx$JAVA_HEAP -jar wQFM-v1.4.jar -i '$w/quartets.txt' -o '$out'" ;;
    wqfm-pipeline-rs) echo "'$RS' gt2wq -t 12 -i '$in' -o '$w/quartets.txt' && '$RS' amalgamate -i '$w/quartets.txt' -o '$out'" ;;
    wqfm-pipeline-rs-1t) echo "'$RS' gt2wq -t 1 -i '$in' -o '$w/quartets.txt' && '$RS' amalgamate -i '$w/quartets.txt' -o '$out'" ;;
    wqfm-tree) echo "cp '$in' '$w/gt.tre' && cd '$TOOLS/wQFM-TREE-v1.0.1' && bash run.sh '$w/gt.tre' '$out'" ;;
    astral3) echo "java -Xmx$JAVA_HEAP -jar '$TOOLS/Astral/astral.5.7.8.jar' -i '$in' -o '$out'" ;;
    astral4) echo "'$TOOLS/ASTER-1.25/bin/astral4' -i '$in' -o '$out'" ;;
    wastral) echo "'$TOOLS/ASTER-1.25/bin/wastral' -i '$in' -o '$out'" ;;
    tree-qmc) echo "'$TOOLS/tree-qmc' -i '$in' -o '$out'" ;;
    *) echo "false" ;;
    esac
}

tail -n +2 "$JOBS" | while IFS=$'\t' read -r job family size input method repeat truth; do
    done_already "$job" && continue
    if stopped "$method" "$family" "$size"; then
        printf '%s\tnot run: limit reached on a smaller input\t\t\t\t\t\t\t\t\n' "$job" >> "$OUTTSV"
        continue
    fi
    # S1 inputs are made here (not timed) and kept only while they are used.
    if [[ $input == gen:* ]]; then
        IFS=: read -r _ kind taxa seed <<< "$input"
        file=$(printf '%s/%s_m%03d_s%s.wqrts' "$GEN" "$kind" "$taxa" "$seed")
        if [ "$current_gen" != "$file" ]; then
            rm -rf "$GEN" && mkdir -p "$GEN"
            "$TOOLS/make_inputs" "$GEN" "$taxa" "$seed" "$kind" > /dev/null
            current_gen=$file
        fi
        input=$file
    fi
    w=$RESULTS/runs/$job
    rm -rf "$w" && mkdir -p "$w"
    out=$w/tree.nwk
    cmd=$(command_for "$method" "$input" "$out" "$w")
    echo "$(date -u +%FT%TZ) $job"
    /usr/bin/time -f '%e\t%M' -o "$w/time.txt" \
        timeout "$TIME_LIMIT" bash -c "ulimit -f $FILE_LIMIT_KB; $cmd" > "$w/log.txt" 2>&1
    code=$?
    secs=$(cut -f 1 "$w/time.txt" | tail -n 1)
    rss=$(cut -f 2 "$w/time.txt" | tail -n 1)
    if [ $code -eq 0 ] && [ -s "$out" ]; then
        status=ok
    elif [ $code -eq 124 ]; then
        status="time limit"
    elif [ $code -eq 153 ] || grep -q "File size limit" "$w/log.txt"; then
        status="file limit"
    elif [ $code -eq 137 ] || grep -q -i "OutOfMemoryError\|memory allocation\|Cannot allocate" "$w/log.txt"; then
        status="memory limit"
    else
        status=error
    fi
    osha=""
    [ -s "$out" ] && osha=$(sha256sum "$out" | cut -d ' ' -f 1)
    qsha=""
    if [ -s "$w/quartets.txt" ]; then
        qsha=$(sha256sum "$w/quartets.txt" | cut -d ' ' -f 1)
        rm -f "$w/quartets.txt"
    fi
    rm -f "$w/gt.tre"* "$w/generate_wqrts.py" "$w/triplets.soda2103" "$w/TEMP_FILE_PYTHON_FOR_EMBEDDED_QUARTETS"
    # Quartet score of the output on quartet inputs (not timed).
    score="" total="" quartets=""
    if [[ $family == S* ]]; then
        quartets=$(grep -c . "$input")
        [ "$status" = ok ] && read -r score total _ < <("$RS" score -i "$input" -t "$out")
    fi
    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$job" "$status" "$code" "$secs" "$rss" \
        "$osha" "$qsha" "$score" "$total" "$quartets" >> "$OUTTSV"
done
rm -rf "$GEN"
echo "all jobs done: $OUTTSV"
