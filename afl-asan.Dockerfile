# syntax=docker/dockerfile:1.7-labs

FROM docker.io/rustlang/rust:nightly AS build
RUN cargo install --locked cargo-afl
WORKDIR /src
COPY --exclude=target . .
WORKDIR /src/afl
ENV RUSTFLAGS="-Zsanitizer=address"
RUN cargo fetch
RUN cargo afl build

#FROM docker.io/library/alpine:3.19.1
FROM docker.io/library/debian:stable-slim
WORKDIR /afl
COPY --from=build /root/.local/share/afl.rs/rustc-1.78.0-9b00956/afl.rs-0.15.5/afl/ ./
WORKDIR /fuzz
COPY --from=build /src/afl/target/debug/parse_ply ./
ENV AFL_USE_ASAN=1
VOLUME ["/in", "/out"]
ENTRYPOINT ["/afl/bin/afl-fuzz", "-i", "/in", "-o", "/out"]
