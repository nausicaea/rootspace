#!/bin/sh

set -e

readonly version="0.8.0"
readonly sha256sum="2e0e7df61bc7dcf61fd65c1b345d05cd1f832598a15c6f42e7e21f86b8d39b1f"
readonly filename="sccache-$version-x86_64-unknown-linux-musl"
readonly tarball="$filename.tar.gz"

cd .gitlab

echo "$sha256sum  $tarball" > sccache.sha256sum
curl -OL "https://github.com/mozilla/sccache/releases/download/$version/$tarball"
sha256sum --check sccache.sha256sum
tar -xf "$tarball"
mv "$filename/sccache" .

