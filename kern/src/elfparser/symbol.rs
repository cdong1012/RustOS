//https://docs.oracle.com/cd/E23824_01/html/819-0690/chapter6-79797.html

use shim::const_assert_size;
use core::fmt::Error;
use alloc::vec::Vec;
use shim::path::{Path, PathBuf};
use crate::FILESYSTEM;
use fat32::traits::FileSystem;
use fat32::traits::Entry;
use crate::console::{kprintln, kprint};
use core::ops::{Deref, DerefMut};
use alloc::fmt;
use core::mem::size_of;
use crate::elfparser::header::{RawELFFile, ELFHeader};
use crate::elfparser::section::{SectionTable, SectionEntry64};

#[derive(Debug, Default)]
pub struct Symbol64 {
    pub st_name : u32,
    pub st_info : u8,
    pub st_other: u8,
    pub st_shndx: u16,
    pub st_value: u64,
    pub st_size : u64,
}

const_assert_size!(Symbol64, 24);

impl Symbol64 {
    pub fn new() -> Symbol64 {
        Symbol64::default()
    }

    pub fn from(raw_section_table: &Vec<u8>, index: usize) -> Symbol64 {
        let mut symbol = Vec::new();

        let mut start = index * 24;
        let end = start + 24;
        while start < end {
            symbol.push((&raw_section_table)[start].clone());
            start += 1;
        }

        let mut new_symbol = Symbol64::new();
        new_symbol.st_name = u32::from_le_bytes([symbol[0], symbol[1], symbol[2], symbol[3]]);

        new_symbol.st_info = symbol[4];

        new_symbol.st_other = symbol[5];

        new_symbol.st_shndx = u16::from_le_bytes([symbol[6], symbol[7]]);

        new_symbol.st_value = u64::from_le_bytes([symbol[8], symbol[9], symbol[10], symbol[11], symbol[12], symbol[13], symbol[14], symbol[15]]);
        
        new_symbol.st_size = u64::from_le_bytes([symbol[16], symbol[17], symbol[18], symbol[19], symbol[20], symbol[21], symbol[22], symbol[23]]);
        new_symbol
    }
}

#[derive(Debug, Default)]
pub struct SymbolTable {
    pub symbols: Vec<Symbol64>,
    pub sectionTable: SectionTable,
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable::default()
    }

    /**
     * #[derive(Debug, Default)]
        pub struct SectionTable {
        pub sections: Vec<SectionEntry64>,
        pub stringTable: SectionEntry64,
        pub elf: RawELFFile
     */
    pub fn from(sectionTable: &SectionTable) -> Result<SymbolTable, Error> {
        let mut symbolTable: &SectionEntry64 = &SectionEntry64::new();
        for section in (&sectionTable.sections).iter() {
            if section.sh_type == 0x2 { // symbol table type == 0x2 
                symbolTable = section;
                break;
            }
        }
        let entry_num = (symbolTable.sh_size as usize)/(symbolTable.sh_entsize as usize);
        let entry_size = symbolTable.sh_entsize as usize;

        let raw = &sectionTable.elf;
        let mut raw_section_table = Vec::new();

        let mut index = symbolTable.sh_addr as usize;
        let end = index + (symbolTable.sh_size as usize);

        while index < end {
            raw_section_table.push((&raw)[index].clone());
            index += 1;
        }
        let mut new_symbol_table = SymbolTable::new();

        let mut start = 0usize;
        while start < entry_num {
            new_symbol_table.symbols.push(Symbol64::from(&raw_section_table, start));
            start += 1;
        }

        new_symbol_table.sectionTable = sectionTable.clone();
        Ok(new_symbol_table)
    }

    pub fn printSymbol(&self, index: usize) {
        kprintln!("Print symbol");
        for symbol in self.symbols.iter() {
            kprintln!("index: {}", symbol.st_name);
            //let name = self.sectionTable.getName(symbol.st_name);
        }
    }

    // pub fn getName(&self, index: u32) -> Vec<u8> {
    //     let stringTableIndex = self.sectionTable.
    //     let stringTable = &self.sectionTable.;
    
    //     let mut offset = stringTable.sh_offset as usize;
    //     let size = stringTable.sh_size as usize;
    //     let mut buffer = Vec::new();
    //     let file_img = &self.elf;
    //     let end = offset + size;
    //     while offset < end {
    //         buffer.push((&self.elf)[offset].clone());
    //         offset += 1;
    //     }
    //     kprintln!("{:?}", core::str::from_utf8(&buffer));
    //     let mut i = index as usize;
    //     let mut name = Vec::new();
    //     loop {
    //         if (&buffer)[i].clone() != 0u8 {
    //             name.push((&buffer)[i].clone());
    //         } else {
    //             break
    //         }
    //         i += 1;
    //     }
    //     name
    // }
}