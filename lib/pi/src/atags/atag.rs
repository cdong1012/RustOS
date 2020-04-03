use crate::atags::raw;

pub use crate::atags::raw::{Core, Mem};
use core::slice::from_raw_parts;
/// An ATAG.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Atag {
    Core(raw::Core),
    Mem(raw::Mem),
    Cmd(&'static str),
    Unknown(u32),
    None,
}

impl Atag {
    /// Returns `Some` if this is a `Core` ATAG. Otherwise returns `None`.
    pub fn core(self) -> Option<Core> {
        match self {
            Atag::Core(core) => Some(core),
            _                => None
        }
    }

    /// Returns `Some` if this is a `Mem` ATAG. Otherwise returns `None`.
    pub fn mem(self) -> Option<Mem> {
        match self {
            Atag::Mem(mem)  => Some(mem),
            _               => None
        }
    }

    /// Returns `Some` with the command line string if this is a `Cmd` ATAG.
    /// Otherwise returns `None`.
    pub fn cmd(self) -> Option<&'static str> {
        match self {
            Atag::Cmd(string) => Some(string),
            _                  => None
        }
    }
}

// FIXME: Implement `From<&raw::Atag> for `Atag`.
impl From<&'static raw::Atag> for Atag {
    fn from(atag: &'static raw::Atag) -> Atag {
        // FIXME: Complete the implementation below.

        unsafe {
            match (atag.tag, &atag.kind) {
                (raw::Atag::CORE, &raw::Kind { core }) => {
                    Atag::Core(core)
                },
                (raw::Atag::MEM, &raw::Kind { mem }) => {
                    Atag::Mem(mem)
                },
                (raw::Atag::CMDLINE, &raw::Kind { ref cmd }) => {
                    let mut size : usize = 0;
                    let mut str_pointer = &(cmd.cmd);
                    while *str_pointer != b'\0' { // check for null pointer
                        size += 1;
                        str_pointer = &*((str_pointer as *const u8).offset(1)); 
                        // cast str_pointer to a u8 pointer, add 1
                        // then cast it back to a raw u8 pointer, and make it a referencee
                    }
                    Atag::Cmd(core::str::from_utf8(core::slice::from_raw_parts(&(cmd.cmd), size)).unwrap())
                },
                (raw::Atag::NONE, _) => {
                    Atag::None
                },
                (id, _) => {
                    Atag::Unknown(id)
                },
            }
        }
    }
}
