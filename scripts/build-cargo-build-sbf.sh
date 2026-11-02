#!/usr/bin/env bash
# Build cargo-build-sbf from an anza-xyz/cargo-build-sbf PR or commit into
# .bin/cargo-build-sbf, the binary the Makefile tests.
#
#   scripts/build-cargo-build-sbf.sh [PR|COMMIT]
set -euo pipefail

root=$(realpath "$(dirname "$0")/..")
src=$root/.cbs/cargo-build-sbf
bin=$root/.bin/cargo-build-sbf

usage() {
    cat <<EOF
Usage: ${0##*/} [PR|COMMIT]
       ${0##*/} --pr PR
       ${0##*/} --commit COMMIT

Build anza-xyz/cargo-build-sbf from a pull request (default: 106) or a full
40-character commit hash.
EOF
}

if (( $# == 0 )); then
    kind='pr'
    revision='106'
elif (( $# == 1 )); then
    case $1 in
        -h|--help)
            usage
            exit 0
            ;;
        *[!0-9a-fA-F]*|'')
            echo "Invalid PR or commit: $1" >&2
            usage >&2
            exit 2
            ;;
        ????????????????????????????????????????)
            kind='commit'
            revision=$1
            ;;
        *[!0-9]*)
            echo "Commit hashes must contain 40 hexadecimal characters: $1" >&2
            exit 2
            ;;
        *)
            kind='pr'
            revision=$1
            ;;
    esac
elif (( $# == 2 )) && [[ $1 == --pr || $1 == --commit ]]; then
    kind=${1#--}
    revision=$2
else
    usage >&2
    exit 2
fi

if [[ $kind == pr && ! $revision =~ ^[0-9]+$ ]]; then
    echo "Invalid PR number: $revision" >&2
    exit 2
elif [[ $kind == commit && ! $revision =~ ^[0-9a-fA-F]{40}$ ]]; then
    echo "Invalid commit hash (expected 40 hexadecimal characters): $revision" >&2
    exit 2
fi

[[ -d $src ]] || git clone --quiet https://github.com/anza-xyz/cargo-build-sbf "$src"
if [[ $kind == pr ]]; then
    git -C "$src" fetch --quiet origin "+pull/$revision/head:refs/pr/$revision"
    git -C "$src" checkout --quiet --detach "refs/pr/$revision"
    source="PR #$revision"
else
    git -C "$src" fetch --quiet origin "$revision"
    git -C "$src" checkout --quiet --detach FETCH_HEAD
    source="commit $revision"
fi

# build.rs refuses to build when the Solana CLI's cargo-build-sbf shadows
# ~/.cargo/bin on PATH.
PATH=$HOME/.cargo/bin:$PATH cargo build --release --quiet \
    --manifest-path "$src/Cargo.toml" --bin cargo-build-sbf

install -D -m 755 "$src/target/release/cargo-build-sbf" "$bin"
echo "$bin: $source at $(git -C "$src" rev-parse --short HEAD)"
