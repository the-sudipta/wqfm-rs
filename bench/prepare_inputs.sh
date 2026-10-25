#!/bin/bash
# Prepares the benchmark inputs in $INPUTS (run in WSL). S1 is not written
# here: its inputs are made by the job runner just before they are used and
# deleted afterwards (they are deterministic and up to 2 GB each).
#
# Usage (WSL): bash bench/prepare_inputs.sh [S2] [S3] [G1] [G2] [G3]
# (no argument: all). Each gene-tree replicate becomes one file of gene
# trees, in the numeric order of the gene folders.

set -eu
source "$(dirname "$0")/config.sh"
parts=${*:-"S2 S3 G1 G2 G3"}
DS=$BENCH/datasets
mkdir -p "$INPUTS"

# Gene trees of one replicate folder, one per line, in numeric gene order.
collect() { # replicate folder, output file
    find "$1" -name 'RAxML_bipartitions.final.f200' | sort -V |
        while read -r f; do tr -d '\r\n' < "$f"; echo; done > "$2"
}

for part in $parts; do
    case $part in
    S2) # wQFM's own weighted-quartet files (commit 7bfdf8e).
        mkdir -p "$INPUTS/S2"
        for f in "$WQFM_SRC"/WQFM/input_files/wqrts_* "$WQFM_SRC"/WQFM/input_files/weighted_quartets_* \
            "$WQFM_SRC"/WQFM/input_files/weird_dataset_1; do
            tr -d '\r' < "$f" > "$INPUTS/S2/$(basename "${f%.txt}").wqrts"
        done ;;
    S3) # Q-MAWS v1.0.0 W2c quartets of Fish mtDNA (qmaws wqfm-export --seed 1).
        mkdir -p "$INPUTS/S3"
        cp "$REPO/crates/wqfm/tests/data/fish_no_strand.wqrts" "$INPUTS/S3/fish_no_strand.wqrts"
        cp /mnt/d/_external/wqfm_work/phase3/inputs/real_00.wqrts "$INPUTS/S3/fish_strand.wqrts" ;;
    G1 | G2) # Mirarab et al. (2014): every model condition, replicates 1 to 10.
        # Each archive is unpacked once into a temporary folder on D:.
        name=$([ "$part" = G1 ] && echo mammalian || echo avian)
        for level in 0.5X 1X 2X; do
            archive=$DS/mirarab2014/$name-$level-estimated-genetrees.tar.bz2
            tmp=$BENCH/unpack_tmp
            rm -rf "$tmp" && mkdir -p "$tmp"
            tar -xjf "$archive" -C "$tmp"
            for cond in $(ls "$tmp"); do
                for r in $(seq 1 10); do
                    out=$INPUTS/$part/$cond/R$r
                    [ -d "$tmp/$cond/R$r" ] || continue
                    mkdir -p "$out"
                    collect "$tmp/$cond/R$r" "$out/genetrees.tre"
                    cp "$DS/mirarab2014/$name-model-species.tre" "$out/true.tre"
                done
            done
            rm -rf "$tmp"
        done ;;
    G3) # ASTRAL-II (Dryad doi:10.6076/D10C7C): the layout is read from its
        # README once the files are downloaded; until then this part stops.
        if [ ! -s "$DS/astral2/estimated-gene-trees.tar.bz" ]; then
            echo "G3: $DS/astral2/estimated-gene-trees.tar.bz is missing (download it from Dryad)" >&2
            exit 1
        fi
        echo "G3: write the extraction from README.md before running" >&2
        exit 1 ;;
    *) echo "unknown part $part" >&2; exit 1 ;;
    esac
    echo "prepared $part"
done
