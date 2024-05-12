#!/bin/sh

set -e

readonly version="0.20.0"
readonly sha256sum="c8bb83967f74734a5a4b23b0136c26db3fcc81570eb389cffda4d67ea6d8ad9a"
readonly basename="cargo-audit-x86_64-unknown-linux-musl-v$version"
readonly filename="$basename.tgz"

cd .gitlab

echo "$sha256sum  $filename" > cargo-audit.sha256sum
curl -OL "https://github.com/rustsec/rustsec/releases/download/cargo-audit%2Fv$version/$filename"
sha256sum --check cargo-audit.sha256sum
tar --strip-components=1 -xf "$filename" "$basename/cargo-audit"
chmod +x cargo-audit

