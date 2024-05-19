# syntax=docker/dockerfile:1

FROM docker.io/rustlang/rust:nightly AS build
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked cargo install --locked cargo-afl
WORKDIR /src
COPY . .
WORKDIR /src/fuzz
ENV AFL_USE_ASAN=1
ENV AFL_USE_MSAN=1
ENV AFL_USE_TSAN=1
ENV AFL_USE_CFISAN=1
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked cargo afl build --locked

FROM docker.io/library/debian:stable-slim
ARG FUZZ_TARGET
WORKDIR /afl
COPY --from=build /root/.local/share/afl.rs/rustc-*/afl.rs-*/afl/ ./
WORKDIR /fuzz
COPY --from=build /src/fuzz/target/debug/$FUZZ_TARGET ./
VOLUME ["/in", "/out"]
COPY --chmod=0755 <<-"EOF" /bin/afl-fuzz
    #!/bin/bash
    set -ex
    exec /afl/bin/afl-fuzz -S $HOSTNAME -i /in -o /out "$@"
EOF
ENTRYPOINT ["/bin/bash", "/bin/afl-fuzz"]
