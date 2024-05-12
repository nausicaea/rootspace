#!/bin/sh

set -e

readonly version="0.30.0"
readonly sha256sum="7adaec5afad826e8b758fffe3b9804d3a371553fa4cdb56ddd9a3592c08a03ac"
readonly filename="cargo-tarpaulin-x86_64-unknown-linux-musl"
readonly tarball="$filename.tar.gz"

cd .gitlab

echo "$sha256sum  $tarball" > tarpaulin.sha256sum
curl -OL "https://github.com/xd009642/tarpaulin/releases/download/$version/$tarball"
sha256sum --check tarpaulin.sha256sum
tar -xf "$tarball"

