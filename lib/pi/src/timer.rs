use crate::common::IO_BASE;
use core::time::Duration;

use volatile::prelude::*;
use volatile::{ReadVolatile, Volatile};
/// The base address for the ARM system timer registers.
const TIMER_REG_BASE: usize = IO_BASE + 0x3000;

// doc: https://jsandler18.github.io/extra/sys-time.html
#[repr(C)]
#[allow(non_snake_case)]
struct Registers {
    CS: Volatile<u32>,
    CLO: ReadVolatile<u32>,
    CHI: ReadVolatile<u32>,
    COMPARE: [Volatile<u32>; 4],
}

/// The Raspberry Pi ARM system timer.
pub struct Timer {
    registers: &'static mut Registers,
}

impl Timer {
    /// Returns a new instance of `Timer`.
    pub fn new() -> Timer {
        Timer {
            registers: unsafe { &mut *(TIMER_REG_BASE as *mut Registers) },
        }
    }

    /// Reads the system timer's counter and returns Duration.
    /// `CLO` and `CHI` together can represent the number of elapsed microseconds.
    pub fn read(&self) -> Duration {
        let low : u64 = self.registers.CLO.read() as u64;
        let high : u64 = (self.registers.CHI.read() as u64) << 32;
        core::time::Duration::from_micros(high | low)
    }

    /// Sets up a match in timer 1 to occur `t` duration from now. If
    /// interrupts for timer 1 are enabled and IRQs are unmasked, then a timer
    /// interrupt will be issued in `t` duration.
    pub fn tick_in(&mut self, t: Duration) {
        let current_time = self.registers.CLO.read();
        self.registers.COMPARE[1].write(current_time + t.as_micros() as u32);
        self.registers.CS.or_mask(0b10 as u32);
    }
}

/// Returns current time.
pub fn current_time() -> Duration {
    Timer::new().read()
}

/// Spins until `t` duration have passed.
pub fn spin_sleep(t: Duration) {
    let time_stop = current_time().as_millis() + t.as_millis();
    while current_time().as_millis() <= time_stop {}
}

/// Sets up a match in timer 1 to occur `t` duration from now. If
/// interrupts for timer 1 are enabled and IRQs are unmasked, then a timer
/// interrupt will be issued in `t` duration.
pub fn tick_in(t: Duration) {
    let mut timer = Timer::new();
    timer.tick_in(t);
}

