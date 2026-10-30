#!/usr/bin/env bash
# Build cargo-build-sbf from an anza-xyz/cargo-build-sbf PR (default #106)
# into .bin/cargo-build-sbf, the binary the Makefile tests.
#
#   scripts/build-cargo-build-sbf.sh [PR]
set -euo pipefail

root=$(realpath "$(dirname "$0")/..")
pr=${1:-106}
src=$root/.cbs/cargo-build-sbf
bin=$root/.bin/cargo-build-sbf

[[ -d $src ]] || git clone --quiet https://github.com/anza-xyz/cargo-build-sbf "$src"
git -C "$src" fetch --quiet origin "+pull/$pr/head:refs/pr/$pr"
git -C "$src" checkout --quiet --detach "refs/pr/$pr"

# build.rs refuses to build when the Solana CLI's cargo-build-sbf shadows
# ~/.cargo/bin on PATH.
PATH=$HOME/.cargo/bin:$PATH cargo build --release --quiet \
    --manifest-path "$src/Cargo.toml" --bin cargo-build-sbf

install -D -m 755 "$src/target/release/cargo-build-sbf" "$bin"
echo "$bin: PR #$pr at $(git -C "$src" rev-parse --short HEAD)"
