#![feature(asm)]
#![feature(global_asm)]

#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(not(test))]
mod init;
use shim::io;
use shim::ioerr;

use core::time::Duration;
use pi;
use core::fmt::Write;
/// Start address of the binary to load and of the bootloader.
const BINARY_START_ADDR: usize = 0x80000;
const BOOTLOADER_START_ADDR: usize = 0x4000000;

/// Pointer to where the loaded binary expects to be laoded.
const BINARY_START: *mut u8 = BINARY_START_ADDR as *mut u8;

/// Free space between the bootloader and the loaded binary's start address.
const MAX_BINARY_SIZE: usize = BOOTLOADER_START_ADDR - BINARY_START_ADDR;

/// Branches to the address `addr` unconditionally.
unsafe fn jump_to(addr: *mut u8) -> ! {
    asm!("br $0" : : "r"(addr as usize));
    loop {
        asm!("wfe" :::: "volatile")
    }
}

fn kmain() -> ! {
    // init uart, timeout 750

    // buffer_mem now points to the memory region where we can load our binaries into
    let mut buffer_mem = unsafe {
        core::slice::from_raw_parts_mut(BINARY_START, MAX_BINARY_SIZE)
    };
    loop {
        let mut miniuart = pi::uart::MiniUart::new();
        miniuart.set_read_timeout(Duration::from_millis(750));
        match xmodem::Xmodem::receive(miniuart, &mut buffer_mem) {
            Ok(_) => {
                unsafe { jump_to(BINARY_START); }
            },
            Err(ref error) => {
                continue;
            }
        }
    }

}
