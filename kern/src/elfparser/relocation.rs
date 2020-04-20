use shim::const_assert_size;
use core::fmt::Error;
use alloc::vec::Vec;
use crate::console::{kprintln, kprint};
use crate::elfparser::section::{SectionTable, SectionEntry64};


pub struct Rela64 {
    pub r_offset: u64, // (the variable's [usually position-independent] virtual memory address which holds the "patched" value during the relocation process)
    pub r_info: u64,    // (Index into .dynsym section and Relocation Type)
    pub r_addend: i64,   
}

const_assert_size!(Rela64, 24);