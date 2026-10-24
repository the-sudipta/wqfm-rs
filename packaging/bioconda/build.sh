#!/bin/bash
# Bioconda build script (draft): installs the wqfm-rs binary into $PREFIX.
set -euo pipefail
cargo-bundle-licenses --format yaml --output THIRDPARTY.yml
cargo install --locked --no-track --root "$PREFIX" --path crates/wqfm-cli
