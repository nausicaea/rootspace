FROM docker.io/library/rust:latest
VOLUME ["/src"]
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked cargo install --locked naga-cli
WORKDIR /src
COPY --chmod=0755 <<-"EOF" /usr/local/bin/naga-wrapper
    #!/bin/bash
    set -xe
    exec /usr/local/cargo/bin/naga "$@"
EOF
ENTRYPOINT ["/bin/bash", "/usr/local/bin/naga-wrapper"]
