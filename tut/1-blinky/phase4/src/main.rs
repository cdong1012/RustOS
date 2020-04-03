#![feature(asm)]
#![feature(global_asm)]

#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(not(test))]
mod init;

const GPIO_BASE: usize = 0x3F000000 + 0x200000;

const GPIO_FSEL1: *mut u32 = (GPIO_BASE + 0x04) as *mut u32;
const GPIO_SET0: *mut u32 = (GPIO_BASE + 0x1C) as *mut u32;
const GPIO_CLR0: *mut u32 = (GPIO_BASE + 0x28) as *mut u32;

#[inline(never)]
fn spin_sleep_ms(ms: usize) {
    for _ in 0..(ms * 6000) {
        unsafe { asm!("nop" :::: "volatile"); }
    }
}
unsafe fn kmain() -> ! {
    // FIXME: STEP 1: Set GPIO Pin 16 as output.
    GPIO_FSEL1.write_volatile((0b0 << 18) as u32); // clear everything(including bit 18-20)
    GPIO_FSEL1.write_volatile((0b001 << 18) as u32); // set 001 to bit 18-20
    loop {
        GPIO_SET0.write_volatile((1 << 16) as u32); // set bit 16
        spin_sleep_ms(1000);
        GPIO_CLR0.write_volatile((1 << 16) as u32); // clear bit 16
        spin_sleep_ms(1000);   
    }
}
