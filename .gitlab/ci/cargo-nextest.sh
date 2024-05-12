#!/bin/sh

set -e

readonly version="0.9.70"
readonly sha256sum="c2d76b2608ce7c92d95aa016889498f273ecacbb3fcffb83db3385f643aa1a9a"
readonly filename="linux"

cd .gitlab

echo "$sha256sum  $filename" > cargo-nextest.sha256sum
curl -L "https://get.nexte.st/$version/$filename"
sha256sum --check cargo-nextest.sha256sum
tar -xf "$filename"
