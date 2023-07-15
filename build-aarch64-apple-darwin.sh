#!/bin/bash
set -euo pipefail
IFS=$'\n\t'

version="${1:-}"
if [[ -z "$version" ]]; then
    echo "usage: $0 version"
    exit 1
fi

# build release binaries
cargo build --release

staging="mech3ax-$version"
target_dir='target/release'
ext='dylib'
target_arch='aarch64-apple-darwin'
archive="$staging-$target_arch.tar.gz"

mkdir "$staging"
# copy supporting files
cp {README.md,LICENSE,mechlib2blend.py,gamez2blend.py} "$staging/"
# copy build artifacts and compress
cp "$target_dir/unzbd" "$target_dir/rezbd" "$staging/"
cp "$target_dir/libmech3ax.$ext" "$staging/lib$staging.$ext"
ls -1 "$staging"
tar czvf "$archive" "$staging"
tar tf "$archive"
