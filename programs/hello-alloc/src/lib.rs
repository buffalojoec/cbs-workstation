//! Uses an allocator (from Pinocchio), and therefore the `alloc` lib.

#![no_std]

extern crate alloc;

#[cfg(target_arch = "bpf")]
use solana_compiler_builtins as _;
use {
    alloc::string::String,
    pinocchio::{
        default_allocator, nostd_panic_handler, program_entrypoint, AccountView, Address,
        ProgramResult,
    },
    solana_program_log::log,
};

program_entrypoint!(process_instruction);
default_allocator!();
nostd_panic_handler!();

pub fn process_instruction(
    _program_id: &Address,
    _accounts: &mut [AccountView],
    _instruction_data: &[u8],
) -> ProgramResult {
    let mut greeting = String::from("Hello, ");
    greeting.push_str("heap!");
    log(&greeting);
    Ok(())
}
