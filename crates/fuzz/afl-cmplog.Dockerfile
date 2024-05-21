# syntax=docker/dockerfile:1

FROM docker.io/rustlang/rust:nightly AS build
ARG FUZZ_TARGET
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked cargo install --locked cargo-afl
WORKDIR /src
COPY . .
WORKDIR /src/crates/fuzz
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked cargo afl build --locked --bin $FUZZ_TARGET
RUN --network=none mv /src/crates/fuzz/target/debug/$FUZZ_TARGET /src/crates/fuzz/target/debug/$FUZZ_TARGET.regular
ENV AFL_LLVM_CMPLOG=1
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked cargo afl build --locked --bin $FUZZ_TARGET
RUN --network=none mv /src/crates/fuzz/target/debug/$FUZZ_TARGET /src/crates/fuzz/target/debug/$FUZZ_TARGET.cmplog

FROM docker.io/library/debian:stable-slim
ARG FUZZ_TARGET
WORKDIR /afl
COPY --from=build /root/.local/share/afl.rs/rustc-*/afl.rs-*/afl/ ./
WORKDIR /fuzz
COPY --from=build /src/crates/fuzz/target/debug/$FUZZ_TARGET.regular ./$FUZZ_TARGET
COPY --from=build /src/crates/fuzz/target/debug/$FUZZ_TARGET.cmplog ./$FUZZ_TARGET.cmplog
VOLUME ["/in", "/out"]
COPY --chmod=0755 <<-"EOF" /bin/afl-fuzz
    #!/bin/bash
    set -ex
    exec /afl/bin/afl-fuzz -S $HOSTNAME -i /in -o /out "$@"
EOF
ENTRYPOINT ["/bin/bash", "/bin/afl-fuzz"]
