
use shim::io;

use fat32::traits::BlockDevice;
use crate::console::{kprintln, kprint};
extern "C" {
    /// A global representing the last SD controller error that occured.
    static sd_err: i64;

    /// Initializes the SD card controller.
    ///
    /// Returns 0 if initialization is successful. If initialization fails,
    /// returns -1 if a timeout occured, or -2 if an error sending commands to
    /// the SD controller occured.
    fn sd_init() -> i32;

    /// Reads sector `n` (512 bytes) from the SD card and writes it to `buffer`.
    /// It is undefined behavior if `buffer` does not point to at least 512
    /// bytes of memory. Also, the caller of this function should make sure that
    /// `buffer` is at least 4-byte aligned.
    ///
    /// On success, returns the number of bytes read: a positive number.
    ///
    /// On error, returns 0. The true error code is stored in the `sd_err`
    /// global. `sd_err` will be set to -1 if a timeout occured or -2 if an
    /// error sending commands to the SD controller occured. Other error codes
    /// are also possible but defined only as being less than zero.
    fn sd_readsector(n: i32, buffer: *mut u8) -> i32;
}

// FIXME: Define a `#[no_mangle]` `wait_micros` function for use by `libsd`.
// The `wait_micros` C signature is: `void wait_micros(unsigned int);`

#[no_mangle]
fn wait_micros(time: u32) {
    use crate::spin_sleep_ms;
    kprintln!("init..");
    spin_sleep_ms((time * 10) as usize);
}

/// A handle to an SD card controller.
#[derive(Debug)]
pub struct Sd;

impl Sd {
    /// Initializes the SD card controller and returns a handle to it.
    /// The caller should assure that the method is invoked only once during the
    /// kernel initialization. We can enforce the requirement in safe Rust code
    /// with atomic memory access, but we can't use it yet since we haven't
    /// written the memory management unit (MMU).
    pub unsafe fn new() -> Result<Sd, io::Error> {
        match sd_init() {
            0i32 => {
                return Ok(Sd{});
            },
            -1i32 => {
                return Err(io::Error::new(io::ErrorKind::TimedOut, "SD init time out"));
            },
            -2i32 => {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "SD init command error"));
            },
            _ => {
                return Err(io::Error::new(io::ErrorKind::Other, "SD init other error"))
            }
        }
    }
}

impl BlockDevice for Sd {
    /// Reads sector `n` from the SD card into `buf`. On success, the number of
    /// bytes read is returned.
    ///
    /// # Errors
    ///
    /// An I/O error of kind `InvalidInput` is returned if `buf.len() < 512` or
    /// `n > 2^31 - 1` (the maximum value for an `i32`).
    ///
    /// An error of kind `TimedOut` is returned if a timeout occurs while
    /// reading from the SD card.
    ///
    /// An error of kind `Other` is returned for all other errors.
    fn read_sector(&mut self, n: u64, buf: &mut [u8]) -> io::Result<usize> {
        if buf.len() < 512 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "buf len < 512"));
        }
        if n > (1 << 31) - 1 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid n"));
        }
        match unsafe { sd_readsector(n as i32, buf.as_mut_ptr())} {
            0 => {
                match unsafe {sd_err} {
                    -1i64 => {
                        return Err(io::Error::new(io::ErrorKind::TimedOut, "SD read_sectory time out"));
                    },
                    -2i64 => {
                        return Err(io::Error::new(io::ErrorKind::InvalidInput, "SD read_sectory command error"));
                    },
                    _ => {
                        return Err(io::Error::new(io::ErrorKind::Other, "SD read_sectory other error"))
                    }
                }
            },
            byte_written => {
                return Ok(byte_written as usize);
            }
        }
    }

    fn write_sector(&mut self, _n: u64, _buf: &[u8]) -> io::Result<usize> {
        unimplemented!("SD card and file system are read only")
    }
}
