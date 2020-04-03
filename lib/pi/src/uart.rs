use core::fmt;
use core::time::Duration;

use shim::const_assert_size;
use shim::io;

use volatile::prelude::*;
use volatile::{ReadVolatile, Reserved, Volatile};

use crate::common::IO_BASE;
use crate::gpio::{Function, Gpio};
use crate::timer;

/// The base address for the `MU` registers.
const MU_REG_BASE: usize = IO_BASE + 0x215040;

/// The `AUXENB` register from page 9 of the BCM2837 documentation.
const AUX_ENABLES: *mut Volatile<u8> = (IO_BASE + 0x215004) as *mut Volatile<u8>;

/// Enum representing bit fields of the `AUX_MU_LSR_REG` register.
#[repr(u8)]
enum LsrStatus {
    DataReady = 1,
    TxAvailable = 1 << 5,
}

#[repr(C)]
#[allow(non_snake_case)]
struct Registers {
    // FIXME: Declare the "MU" registers from page 8.
    AUX_MU_IO_REG: Volatile<u8>,            
    __r0: [Reserved<u8>; 3],
    AUX_MU_IER_REG: Volatile<u8>,          
    __r1: [Reserved<u8>; 3],
    AUX_MU_IIR_REG: Volatile<u8>,           
    __r2: [Reserved<u8>; 3],
    AUX_MU_LCR_REG: Volatile<u8>,           
    __r3: [Reserved<u8>; 3],
    AUX_MU_MCR_REG: Volatile<u8>,          
    __r4: [Reserved<u8>; 3],
    AUX_MU_LSR_REG: ReadVolatile<u8>,      
    __r5: [Reserved<u8>; 3],
    AUX_MU_MSR_REG: ReadVolatile<u8>,       
    __r6: [Reserved<u8>; 3],
    AUX_MU_SCRATCH_REG: Volatile<u8>,      
    __r7: [Reserved<u8>; 3],
    AUX_MU_CNTL_REG: Volatile<u8>,          
    __r8: [Reserved<u8>; 3],
    AUX_MU_STAT_REG: ReadVolatile<u32>,     
    AUX_MU_BAUD_REG: Volatile<u16>,
}

const_assert_size!(Registers, 0x7E21506C - 0x7E215040);

/// The Raspberry Pi's "mini UART".
pub struct MiniUart {
    registers: &'static mut Registers,
    timeout: Option<Duration>,
}

impl MiniUart {
    /// Initializes the mini UART by enabling it as an auxiliary peripheral,
    /// setting the data size to 8 bits, setting the BAUD rate to ~115200 (baud
    /// divider of 270), setting GPIO pins 14 and 15 to alternative function 5
    /// (TXD1/RDXD1), and finally enabling the UART transmitter and receiver.
    ///
    /// By default, reads will never time out. To set a read timeout, use
    /// `set_read_timeout()`.
    pub fn new() -> MiniUart {
        let registers = unsafe {
            // Enable the mini UART as an auxiliary device.
            (*AUX_ENABLES).or_mask(1);
            &mut *(MU_REG_BASE as *mut Registers)
        };

        // set data size to 8 bits
        registers.AUX_MU_LCR_REG.or_mask(0b11 as u8);

        // set BAUD rate to ~115200
        registers.AUX_MU_BAUD_REG.write(270);

        // set GPIO pin 14 to alternative func 5
        Gpio::new(14u8).into_alt(Function::Alt5);

        // set GPIO pin 15 to alternative func 5
        Gpio::new(15u8).into_alt(Function::Alt5);

        // enable transmitter and receiver
        registers.AUX_MU_CNTL_REG.or_mask(0b11);

        MiniUart {
            registers: registers,
            timeout : None,
        }
        // FIXME: Implement remaining mini UART initialization.

    }

    /// Set the read timeout to `t` duration.
    pub fn set_read_timeout(&mut self, t: Duration) {
        self.timeout = Some(t);
    }

    /// Write the byte `byte`. This method blocks until there is space available
    /// in the output FIFO.
    pub fn write_byte(&mut self, byte: u8) {
        loop {
            if self.registers.AUX_MU_LSR_REG.has_mask(LsrStatus::TxAvailable as u8) {
                self.registers.AUX_MU_IO_REG.write(byte);
                break;
            }
        }
    }

    /// Returns `true` if there is at least one byte ready to be read. If this
    /// method returns `true`, a subsequent call to `read_byte` is guaranteed to
    /// return immediately. This method does not block.
    pub fn has_byte(&self) -> bool {
        // if has at least 1 byte, bit is 1 -> has_mask return true
        self.registers.AUX_MU_LSR_REG.has_mask(LsrStatus::DataReady as u8)
    }

    /// Blocks until there is a byte ready to read. If a read timeout is set,
    /// this method blocks for at most that amount of time. Otherwise, this
    /// method blocks indefinitely until there is a byte to read.
    ///
    /// Returns `Ok(())` if a byte is ready to read. Returns `Err(())` if the
    /// timeout expired while waiting for a byte to be ready. If this method
    /// returns `Ok(())`, a subsequent call to `read_byte` is guaranteed to
    /// return immediately.
    pub fn wait_for_byte(&self) -> Result<(), ()> {
        match self.timeout {
            Some(time) => {
                let time_stop = timer::current_time().checked_add(time).unwrap();
                while timer::current_time() <= time_stop {
                    if self.has_byte() {
                        return Ok(())
                    }
                }
                return Err(())
            }, 
            None  => {
                loop {
                    if self.has_byte() {
                        return Ok(())
                    }
                }
            },
        };
    }

    /// Reads a byte. Blocks indefinitely until a byte is ready to be read.
    pub fn read_byte(&mut self) -> u8 {
        loop {
            match self.wait_for_byte() {
                Ok(()) => {return self.registers.AUX_MU_IO_REG.read();},
                Err(()) => {continue;}
            }
        }
    }
}

impl fmt::Write for MiniUart {
    fn write_str(&mut self, s : &str) -> core::result::Result<(), core::fmt::Error> {
        for byte in s.bytes() {
            match byte {
                b'\n'    => { // write '\r' before
                    self.write_byte(b'\r');
                    self.write_byte(b'\n');
                },
                32..=126 => { // ascii values that we can print
                    self.write_byte(byte);
                },
                _       => { // invalid ascii, print an error character
                    self.write_byte(233);
                }
            };
        }
        Ok(())
    }
}
// FIXME: Implement `fmt::Write` for `MiniUart`. A b'\r' byte should be written
// before writing any b'\n' byte.

mod uart_io {
    use super::io;
    use super::MiniUart;
    use volatile::prelude::*;
    // FIXME: Implement `io::Read` and `io::Write` for `MiniUart`.
    //
    // The `io::Read::read()` implementation must respect the read timeout by
    // waiting at most that time for the _first byte_. It should not wait for
    // any additional bytes but _should_ read as many bytes as possible. If the
    // read times out, an error of kind `TimedOut` should be returned.
    //
    // The `io::Write::write()` method must write all of the requested bytes
    // before returning.
    use shim::ioerr;
    impl io::Read for MiniUart {
        
        fn read(&mut self, buffer: &mut [u8]) -> core::result::Result<usize, io::Error> {
            let mut index = 0;
            match self.wait_for_byte() { // wait until we get the first byte
                Ok(_)  => {
                    buffer[index] = self.read_byte(); // read first byte
                    index += 1;
                    while index < buffer.len() { // if still has some more, read some more
                        if !self.has_byte() {
                            return Ok(index);
                        }
                        buffer[index] = self.read_byte();
                        index += 1;
                    }
                    return Ok(index);
                },
                Err(_) => {
                    return Err(io::Error::new(io::ErrorKind::TimedOut, "timeout"));
                }
            }
        }
    }

    impl io::Write for MiniUart {
        fn write(&mut self, buffer :&[u8]) -> core::result::Result<usize, io::Error> {
            for byte in buffer {
                self.write_byte(*byte);
            }
            Ok(buffer.len())
        }
        fn flush(&mut self) -> core::result::Result<(), io::Error> {
            Ok(())
        }
    }


}
