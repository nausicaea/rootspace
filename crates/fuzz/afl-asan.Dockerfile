# syntax=docker/dockerfile:1

FROM docker.io/rustlang/rust:nightly AS build
ARG FUZZ_TARGET
ARG HOST_TARGET
RUN rustup component add rust-src --toolchain nightly-aarch64-unknown-linux-gnu
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked cargo install --locked cargo-afl
WORKDIR /src
COPY . .
WORKDIR /src/crates/fuzz
ENV AFL_USE_ASAN=1
COPY --chmod=0755 <<-EOF /bin/cargo-afl-build
    #!/bin/sh
    set -ex
    export RUST_FLAGS="-Z sanitizer=address -C opt-level=0" 
    exec cargo afl build -Z build-std --target $HOST_TARGET --locked --bin $FUZZ_TARGET
EOF
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked /bin/cargo-afl-build

FROM docker.io/library/debian:stable-slim
ARG FUZZ_TARGET
ARG HOST_TARGET
WORKDIR /afl
COPY --from=build /root/.local/share/afl.rs/rustc-*/afl.rs-*/afl/ ./
WORKDIR /fuzz
COPY --from=build /src/crates/fuzz/target/$HOST_TARGET/debug/$FUZZ_TARGET ./
VOLUME ["/in", "/out"]
COPY --chmod=0755 <<-"EOF" /bin/afl-fuzz
    #!/bin/bash
    set -ex
    exec /afl/bin/afl-fuzz -S $HOSTNAME -i /in -o /out "$@"
EOF
ENTRYPOINT ["/bin/bash", "/bin/afl-fuzz"]
