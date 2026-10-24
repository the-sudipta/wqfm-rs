# Releasing wQFM-rs

Each step marked **(owner)** is public or hard to undo and is done only with
the owner's explicit approval.

1. All Phase 3 checks pass; CI is green on the commit to release.
2. Set the version in `Cargo.toml` (workspace), `CITATION.cff` and the
   dependency line of `crates/wqfm-cli/Cargo.toml`; move the CHANGELOG
   entries under the new version with the date.
3. **(owner)** Tag and push: `git tag -a v1.0.0 -m "wQFM-rs 1.0.0"` and
   `git push origin v1.0.0`. The Release workflow builds five binaries with
   SHA-256 sums into a **draft** release.
4. **(owner)** Check the draft release (download one binary, run
   `wqfm-rs --version`), then publish it.
5. **(owner)** crates.io: `cargo publish -p wqfm`, then
   `cargo publish -p wqfm-rs`.
6. **(owner)** Zenodo: archive the GitHub release to get a DOI; add it to
   `CITATION.cff` and the README.
7. **(owner)** Bioconda: copy `packaging/bioconda/` into a fork of
   bioconda-recipes as `recipes/wqfm-rs/`, set the version and the sha256 of
   the release archive, open the pull request.
8. Docker (optional): `docker build -t wqfm-rs .` from the tag.
