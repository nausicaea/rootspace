# syntax=docker/dockerfile:1.7-labs

FROM docker.io/library/rust:1.78.0 AS build
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked cargo install --locked cargo-afl
WORKDIR /src
COPY --exclude=target . .
WORKDIR /src/fuzz
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked cargo fetch
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked --network=none cargo afl build --frozen --offline

FROM docker.io/library/debian:stable-slim
WORKDIR /afl
COPY --from=build /root/.local/share/afl.rs/rustc-*/afl.rs-*/afl/ ./
WORKDIR /fuzz
COPY --from=build /src/fuzz/target/debug/parse_ply ./
VOLUME ["/in", "/out"]
ENTRYPOINT ["/afl/bin/afl-fuzz", "-i", "/in", "-o", "/out"]
