# syntax=docker/dockerfile:1.7-labs

FROM docker.io/rustlang/rust:nightly AS build
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked cargo install --locked cargo-afl
WORKDIR /src
COPY --exclude=target . .
WORKDIR /src/fuzz
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked cargo afl build --locked
RUN --network=none mv /src/fuzz/target/debug/parse_ply /src/fuzz/target/debug/parse_ply.regular
ENV AFL_LLVM_CMPLOG=1
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked cargo afl build --locked
RUN --network=none mv /src/fuzz/target/debug/parse_ply /src/fuzz/target/debug/parse_ply.cmplog

FROM docker.io/library/debian:stable-slim
WORKDIR /afl
COPY --from=build /root/.local/share/afl.rs/rustc-*/afl.rs-*/afl/ ./
WORKDIR /fuzz
COPY --from=build /src/fuzz/target/debug/parse_ply.regular ./parse_ply
COPY --from=build /src/fuzz/target/debug/parse_ply.cmplog ./parse_ply.cmplog
VOLUME ["/in", "/out"]
ENTRYPOINT ["/afl/bin/afl-fuzz", "-i", "/in", "-o", "/out"]
