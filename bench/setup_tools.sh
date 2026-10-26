#!/bin/bash
# Unpacks and builds every method of the benchmark into $TOOLS (run in WSL,
# after the Q-MAWS benchmark: building ASTER uses the CPU for a few minutes).
# Writes $TOOLS/versions.tsv with the SHA-256 of every program used.
#
# Usage (WSL): bash bench/setup_tools.sh <wqfm-rs Linux binary>
# The Linux binary is built on Windows with
#   RUSTFLAGS="-C linker=rust-lld" cargo build --release --locked \
#     --bin wqfm-rs --example make_inputs --target x86_64-unknown-linux-musl

set -eu
source "$(dirname "$0")/config.sh"
DL=${WQFM_DOWNLOADS:-$BENCH/tools}
mkdir -p "$TOOLS"
cd "$TOOLS"

# wQFM-rs (and its input generator, built next to it).
install -m 755 "$1" wqfm-rs
gen="$(dirname "$1")/examples/make_inputs"
[ -f "$gen" ] && install -m 755 "$gen" make_inputs

# wQFM v1.4 jar and its pipeline script with the helper binary.
mkdir -p wqfm-pipeline
cp "$WQFM_SRC/wQFM-v1.4.jar" "$WQFM_SRC/generate_wqrts.py" "$WQFM_SRC/triplets.soda2103" wqfm-pipeline/
cp -r "$WQFM_SRC/WQFM/lib" wqfm-pipeline/ 2> /dev/null || true
chmod u+x wqfm-pipeline/triplets.soda2103

# wQFM-TREE v1.1 (folder wQFM-TREE-v1.0.1, with PAUP* for its consensus tree).
rm -rf wQFM-TREE-v1.0.1 && unzip -q "$DL/wQFM-TREE-v1.1.zip"
chmod u+x wQFM-TREE-v1.0.1/paup
# The bundled paup is a 32-bit program, which WSL1 cannot run; the 64-bit
# PAUP* 4.0a169 for Ubuntu (phylosolutions.com/paup-test) replaces it when
# downloaded (Amendment 1).
# It needs libgfortran.so.4 and libpython2.7, which Ubuntu 24.04 no longer
# ships; they come from the official Ubuntu 18.04 packages in
# $DL/ubuntu18-libs, unpacked here and seen by PAUP* only (a wrapper sets
# LD_LIBRARY_PATH; nothing is installed system-wide).
if [ -f "$DL/paup4a169_ubuntu64.gz" ]; then
    gunzip -c "$DL/paup4a169_ubuntu64.gz" > wQFM-TREE-v1.0.1/paup.bin
    chmod u+x wQFM-TREE-v1.0.1/paup.bin
    rm -rf paup-libs && mkdir paup-libs
    for d in "$DL"/ubuntu18-libs/*.deb; do dpkg-deb -x "$d" paup-libs; done
    # PAUP* 4.0a169 also writes the majority-rule frequencies as labels of
    # inner nodes, which the bundled version did not and wQFM-TREE cannot
    # read; after PAUP* exits, the wrapper removes those labels from the
    # tree files named in the command file (the topology is unchanged).
    cat > wQFM-TREE-v1.0.1/paup <<WRAPPER
#!/bin/bash
LD_LIBRARY_PATH="$TOOLS/paup-libs/usr/lib/x86_64-linux-gnu" "$TOOLS/wQFM-TREE-v1.0.1/paup.bin" "\$@"
code=\$?
cmd="\${@: -1}"
if [ -f "\$cmd" ]; then
    for f in \$(grep -o 'treefile = [^ ]*' "\$cmd" | cut -d ' ' -f 3); do
        [ -f "\$f" ] && sed -i -E '/^[[:space:]]*tree /s/\)[0-9.]+/)/g' "\$f"
    done
fi
exit \$code
WRAPPER
    chmod u+x wQFM-TREE-v1.0.1/paup
fi

# ASTRAL-III 5.7.8.
rm -rf Astral && unzip -q "$DL/Astral.5.7.8.zip"

# ASTER v1.25: ASTRAL-IV (bin/astral4) and wASTRAL (bin/wastral).
rm -rf ASTER-1.25 && tar -xzf "$DL/ASTER-v1.25.tar.gz"
# (SKIP_ASTER=1 skips the build, for a quick test of the scripts.)
[ "${SKIP_ASTER:-0}" = 1 ] || make -C ASTER-1.25 astral wastral > ASTER-build.log 2>&1

# TREE-QMC v3.0.4 (released Linux binary).
install -m 755 "$DL/tree-qmc-v3.0.4-b5bfe82-linux64" tree-qmc

{
    printf 'program\tsha256\n'
    for f in wqfm-rs make_inputs wqfm-pipeline/wQFM-v1.4.jar wqfm-pipeline/generate_wqrts.py \
        wqfm-pipeline/triplets.soda2103 wQFM-TREE-v1.0.1/wQFM-TREE-v1.0.1.jar \
        wQFM-TREE-v1.0.1/paup Astral/astral.5.7.8.jar ASTER-1.25/bin/astral4 \
        ASTER-1.25/bin/wastral tree-qmc; do
        [ -f "$f" ] && printf '%s\t%s\n' "$f" "$(sha256sum "$f" | cut -d ' ' -f 1)"
    done
    printf 'java\t%s\n' "$(java -version 2>&1 | head -n 1)"
    printf 'g++\t%s\n' "$(g++ --version | head -n 1)"
    printf 'python3\t%s\n' "$(python3 --version)"
} > versions.tsv
cat versions.tsv
