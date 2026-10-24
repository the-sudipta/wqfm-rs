# wqfm-rs in a small container.
#   docker build -t wqfm-rs .
#   docker run --rm -v "$PWD":/data wqfm-rs amalgamate -i /data/quartets.txt -o /data/tree.nwk

FROM rust:1.98.1-slim-bookworm AS build
WORKDIR /src
COPY . .
RUN cargo build --release --locked --bin wqfm-rs

FROM debian:bookworm-slim
COPY --from=build /src/target/release/wqfm-rs /usr/local/bin/wqfm-rs
COPY LICENSE NOTICE /usr/share/doc/wqfm-rs/
ENTRYPOINT ["wqfm-rs"]
