# syntax=docker/dockerfile:1.7-labs

FROM docker.io/rustlang/rust:nightly AS build
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked cargo install --locked cargo-afl
WORKDIR /src
COPY --exclude=target . .
WORKDIR /src/fuzz
ENV AFL_USE_ASAN=1
ENV AFL_USE_MSAN=1
ENV AFL_USE_TSAN=1
ENV AFL_USE_CFISAN=1
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked cargo afl build --locked

FROM docker.io/library/debian:stable-slim
WORKDIR /afl
COPY --from=build /root/.local/share/afl.rs/rustc-*/afl.rs-*/afl/ ./
WORKDIR /fuzz
COPY --from=build /src/fuzz/target/debug/parse_ply ./
VOLUME ["/in", "/out"]
ENTRYPOINT ["/afl/bin/afl-fuzz", "-i", "/in", "-o", "/out"]
