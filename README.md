# Test Bed for `cargo-build-sbf --toolchain`

Sample programs for testing `cargo-build-sbf --toolchain <TOOLCHAIN>`, where
`TOOLCHAIN` can be either Anza's platform-tools or Blueshift's upstream
compiler.

Serves as a sandbox for the new `--toolchain` feature. Tests could be widened
to Blueshift's [sbpf-compiler-tests], but this repo is designed to test
specifically `cargo-build-sbf`.

[sbpf-compiler-tests]: https://github.com/blueshift-gg/sbpf-compiler-tests

## Setup

Build the `cargo-build-sbf` under test:

```sh
scripts/build-cargo-build-sbf.sh         # PR #106
scripts/build-cargo-build-sbf.sh <PR>    # any other anza-xyz/cargo-build-sbf PR
```

Install Blueshift's sbpf-linker:

```
cargo install sbpf-linker
```

## Build

```sh
make build_sbf         # platform-tools, into out/sbf
make build_upstream    # --toolchain upstream, into out/upstream
make test              # both builds, then run each pair in mollusk and compare
make clean
```

## Requirements

What a program needs in order to build with `--toolchain upstream`.

The program must be `no_std`. Upstream targets `bpfel-unknown-none`, which is
a Tier 3 target with no `std`.

```rust
#![no_std]
```

## Additional Requirements

LLVM lowers some operations to calls to helper routines (ie. `memcpy`).
platform-tools provides these but upstream doesn't, so programs that need them
must add `solana-compiler-builtins`:

```toml
[target.'cfg(target_arch = "bpf")'.dependencies]
solana-compiler-builtins = "0.3"
```

```rust
#[cfg(target_arch = "bpf")]
use solana_compiler_builtins as _;
```

`[profile.release]` configurations can be used to tune the resulting binary.
`lto = "fat"` and `codegen-units = 1` combined can shrink the binary size
significantly.

```toml
[profile.release]
lto = "fat"
codegen-units = 1
```
