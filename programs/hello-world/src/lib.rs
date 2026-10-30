#![no_std]

use {
    pinocchio::{nostd_panic_handler, program_entrypoint, AccountView, Address, ProgramResult},
    solana_program_log::log,
};

program_entrypoint!(process_instruction);
nostd_panic_handler!();

pub fn process_instruction(
    _program_id: &Address,
    _accounts: &mut [AccountView],
    _instruction_data: &[u8],
) -> ProgramResult {
    log!("Hello, world!");
    Ok(())
}
