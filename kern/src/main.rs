#![feature(alloc_error_handler)]
#![feature(const_fn)]
#![feature(decl_macro)]
#![feature(asm)]
#![feature(global_asm)]
#![feature(optin_builtin_traits)]
#![feature(ptr_internals)]
#![feature(raw_vec_internals)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(not(test))]
mod init;

extern crate alloc;
pub mod allocator;
pub mod console;
pub mod fs;
pub mod mutex;
pub mod shell;
pub mod elfparser;
extern crate pi;
use core::ops::{DerefMut, Deref, Drop};
const GPIO_BASE: usize = 0x3F000000 + 0x200000;
use alloc::string::String;
use fat32::vfat::{File, VFat, VFatHandle};
use aarch64::{current_el, brk, svc};
use pi::rand::{RNG};
const GPIO_FSEL1: *mut u32 = (GPIO_BASE + 0x04) as *mut u32;
const GPIO_SET0: *mut u32 = (GPIO_BASE + 0x1C) as *mut u32;
const GPIO_CLR0: *mut u32 = (GPIO_BASE + 0x28) as *mut u32;

#[inline(never)]
fn spin_sleep_ms(ms: usize) {
    pi::timer::spin_sleep(core::time::Duration::from_millis(ms as u64));
}
pub mod param;
pub mod process;
pub mod traps;
pub mod vm;

use console::{kprintln};
use crate::shell::shell;

fn gpio() -> ! {
    // blue 16
    // green 13
    // red 6
    // white 5
    let mut blue = pi::gpio::Gpio::new(16).into_output();
    let mut green = pi::gpio::Gpio::new(13).into_output();
    let mut red = pi::gpio::Gpio::new(6).into_output();
    let mut white = pi::gpio::Gpio::new(5).into_output();
    loop {
        blue.set();
        spin_sleep_ms(500);
        blue.clear();
        white.set();
        spin_sleep_ms(500);
        white.clear();
        red.set();
        spin_sleep_ms(500);
        red.clear();
        green.set();
        spin_sleep_ms(500);
        green.clear();
    }
}

fn print_atag() {
    let atags = pi::atags::Atags::get();
    for atag in atags {
        kprintln!("{:#?}", atag);
    }
}
fn echo() -> !{
    let mut mini_uart = pi::uart::MiniUart::new();
    loop {
        let read = mini_uart.read_byte();
        mini_uart.write_byte(read);
        spin_sleep_ms(500);
    }
} 
use allocator::Allocator;
use fs::FileSystem;
use process::GlobalScheduler;
use traps::irq::Irq;
use vm::VMManager;
use rand_core::RngCore;
use elfparser::{RawELFFile, ELFHeader, ProgHeader64, ELF, PeterRand, SectionTable, Symbol64, SymbolTable};
#[cfg_attr(not(test), global_allocator)]
pub static ALLOCATOR: Allocator = Allocator::uninitialized();
pub static FILESYSTEM: FileSystem = FileSystem::uninitialized();
use alloc::vec::Vec;
pub static SCHEDULER: GlobalScheduler = GlobalScheduler::uninitialized();
pub static VMM: VMManager = VMManager::uninitialized();
pub static IRQ: Irq = Irq::uninitialized();
use pi::timer::current_time;
use shim::path::Path;
fn kmain() -> ! {
    unsafe {
        ALLOCATOR.initialize();
        FILESYSTEM.initialize();
        IRQ.initialize();
        VMM.initialize();
        kprintln!("Yeet");
        demo_print_elf()
        // SCHEDULER.initialize();
        // SCHEDULER.start();
    }
    loop {}
}

fn demo_print_elf() {
    let mut elf = ELF::new();
    elf.initialize(Path::new("real"));
    let sectionTable = SectionTable::from(&elf.raw).unwrap();
    // sectionTable.printSectionTable();
    // sectionTable.printSection(40);
    // sectionTable.printSection(41);
    //elf.print_elf();
    let symbolTable = SymbolTable::from(&sectionTable).unwrap();
    //symbolTable.getName(0);
    //kprintln!("{:?}", core::str::from_utf8(&(symbolTable.getName(786))));
    symbolTable.printSymbolTable();
}
