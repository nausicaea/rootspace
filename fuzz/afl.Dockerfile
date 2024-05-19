# syntax=docker/dockerfile:1

FROM docker.io/library/rust:1.78.0 AS build
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked cargo install --locked cargo-afl
WORKDIR /src
COPY . .
WORKDIR /src/fuzz
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked cargo fetch
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked --network=none cargo afl build --frozen --offline

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
    exec /afl/bin/afl-fuzz -i /in -o /out "$@"
EOF
ENTRYPOINT ["/bin/bash", "/bin/afl-fuzz"]
