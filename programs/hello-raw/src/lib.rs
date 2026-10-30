//! Raw entrypoint, panic handler and syscall. Just like Blueshift's
//! `solana-upstream-bpf-template`.

#![no_std]

use solana_define_syscall::definitions::{abort, sol_log_};

const MESSAGE: &str = "Hello, raw world!";

/// # Safety
///
/// Called by the runtime with a pointer to the serialized input region.
#[no_mangle]
pub unsafe extern "C" fn entrypoint(_input: *mut u8) -> u64 {
    // SAFETY: `MESSAGE` is a valid UTF-8 slice for the duration of the call.
    unsafe { sol_log_(MESSAGE.as_ptr(), MESSAGE.len() as u64) };
    0
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    // SAFETY: `abort` takes no arguments and never returns.
    unsafe { abort() }
}
