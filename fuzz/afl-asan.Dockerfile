# syntax=docker/dockerfile:1.7-labs

FROM docker.io/rustlang/rust:nightly AS build
RUN rustup component add rust-src --toolchain nightly-aarch64-unknown-linux-gnu
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked cargo install --locked cargo-afl
WORKDIR /src
COPY --exclude=target . .
WORKDIR /src/fuzz
ENV RUSTFLAGS="-Zsanitizer=address"
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked cargo afl build -Zbuild-std --target aarch64-unknown-linux-gnu --locked

FROM docker.io/library/debian:stable-slim
WORKDIR /afl
COPY --from=build /root/.local/share/afl.rs/rustc-1.78.0-9b00956/afl.rs-0.15.5/afl/ ./
WORKDIR /fuzz
COPY --from=build /src/fuzz/target/debug/parse_ply ./
VOLUME ["/in", "/out"]
ENV AFL_USE_ASAN=1
ENTRYPOINT ["/afl/bin/afl-fuzz", "-i", "/in", "-o", "/out"]
