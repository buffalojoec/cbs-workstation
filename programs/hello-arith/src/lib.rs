//! Arithmetic whose instructions depend on the eBPF CPU.
//!
//! SBPFv3 aligns with CPU v3. `cargo-build-sbf --toolchain upstream` should
//! target CPU v3. Targeting CPU v4 instead adds instructions SBPFv3 lacks:
//!
//! - `i64` division and remainder:
//!   - platform-tools: calls a software division routine.
//!   - upstream: rejected at CPU v3. At CPU v4, `sdiv`/`smod`, which
//!     sbpf-linker rewrites.
//! - `as i8` casts (sign extension):
//!   - platform-tools: shifts (`lsh64`, `arsh64`).
//!   - upstream: shifts at CPU v3. At CPU v4, `movsx`/`ldsx` unless disabled.
//! - `swap_bytes`:
//!   - platform-tools: `be32`.
//!   - upstream: `be32` at CPU v3. At CPU v4, `bswap` unless disabled.
//! - `u32` math:
//!   - platform-tools: 64-bit instructions.
//!   - upstream: 32-bit ALU instructions, which sBPF v3 sign-extends, so
//!     sbpf-linker adds zero-extensions.
//! - `i128` multiply:
//!   - platform-tools: `__multi3` from its own compiler-builtins.
//!   - upstream: `__multi3` from `solana-compiler-builtins`.

#![no_std]

#[cfg(target_arch = "bpf")]
use solana_compiler_builtins as _;
use {
    pinocchio::{
        error::ProgramError, no_allocator, nostd_panic_handler, program_entrypoint, AccountView,
        Address, ProgramResult,
    },
    solana_program_log::log,
};

program_entrypoint!(process_instruction);
no_allocator!();
nostd_panic_handler!();

fn word(data: &[u8], index: usize) -> Result<i64, ProgramError> {
    data.get(index * 8..(index + 1) * 8)
        .and_then(|bytes| bytes.try_into().ok())
        .map(i64::from_le_bytes)
        .ok_or(ProgramError::InvalidInstructionData)
}

pub fn process_instruction(
    _program_id: &Address,
    _accounts: &mut [AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let (a, b, c) = (
        word(instruction_data, 0)?,
        word(instruction_data, 1)?,
        word(instruction_data, 2)?,
    );

    let quotient = a.checked_div(b).ok_or(ProgramError::ArithmeticOverflow)?;
    let remainder = a.checked_rem(b).ok_or(ProgramError::ArithmeticOverflow)?;
    let sum = (a as u32).wrapping_add(b as u32) as u64;
    let narrowed = (c as i8) as i64 + (instruction_data[0] as i8) as i64;
    let swapped = (c as u32).swap_bytes();
    let product = (a as i128).wrapping_mul(c as i128);

    log!(
        "q={} r={} sum={} narrowed={} swapped={}",
        quotient,
        remainder,
        sum,
        narrowed,
        swapped
    );
    log!(
        "product hi={} lo={}",
        (product >> 64) as u64,
        product as u64
    );
    Ok(())
}
