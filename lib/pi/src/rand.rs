use crate::common::IO_BASE;
use volatile::prelude::*;
use volatile::{ReadVolatile, Volatile};

const RAND_REG_BASE: usize = IO_BASE + 0x104000;

#[repr(C)]
#[allow(non_snake_case)]
struct Registers {
    RNG_CTRL: Volatile<u32>,
    RNG_STATUS: Volatile<u32>,
    RNG_DATA: Volatile<u32>,
    RNG_INT_MASK: Volatile<u32>,
}

// #define RNG_CTRL        ((volatile unsigned int*)(MMIO_BASE+0x00104000))
// #define RNG_STATUS      ((volatile unsigned int*)(MMIO_BASE+0x00104004))
// #define RNG_DATA        ((volatile unsigned int*)(MMIO_BASE+0x00104008))
// #define RNG_INT_MASK    ((volatile unsigned int*)(MMIO_BASE+0x00104010))

/// The Raspberry Pi ARM system timer.
pub struct RNG {
    registers: &'static mut Registers,
}

impl RNG {
    pub fn new() -> RNG {
        RNG {
            registers: unsafe { &mut *(RAND_REG_BASE as *mut Registers) },
        }
    }

    pub fn rand_init(&mut self) {
        self.registers.RNG_STATUS.write(0x40000);
        self.registers.RNG_INT_MASK.or_mask(0b1);
        self.registers.RNG_CTRL.or_mask(0b1);
        while self.registers.RNG_STATUS.read() >> 24 != 0 {
            unsafe {asm!("nop");}
        }
    }

    pub fn rand(&self, min: u32, max: u32) -> u32 {
        self.registers.RNG_DATA.read() % (max - min) + min
    }
}