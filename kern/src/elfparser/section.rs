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
#[derive(Debug, Default)]
pub struct SectionEntry64 {
    pub sh_name: u32,
    pub sh_type: u32,
    pub sh_flag: u64,
    pub sh_addr: u64,
    pub sh_offset: u64,
    pub sh_size: u64,
    pub sh_link: u32,
    pub sh_info: u32,
    pub sh_addralign: u64,
    pub sh_entsize: u64,
}
const_assert_size!(SectionEntry64, 64);


impl SectionEntry64 {
    pub fn new() -> SectionEntry64 {
        SectionEntry64::default()
    }

    pub fn from(elf: &RawELFFile, index: usize) -> Result<SectionEntry64, Error> {
        let elfheader = ELFHeader::from(elf).unwrap();

        let start = elfheader.e_shoff as usize + index * size_of::<SectionEntry64>();
        let raw = elf.as_slice();
        let mut buffer = [0u8; size_of::<SectionEntry64>()];
        buffer.copy_from_slice(&raw[start..(start + size_of::<SectionEntry64>())]);
        // buffer now has the program header in it
        // Parsing
        
        let mut section_header = SectionEntry64::new();
        let is_little = match elfheader.ei_data {
            1 => true,
            2 => false,
            _ => {
                panic!("EI_DATA not valid");
            }
        };

        section_header.sh_name = match is_little {
            true => u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]),
            false => u32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]),
        };

        section_header.sh_type = match is_little {
            true => u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]),
            false => u32::from_be_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]),
        };

        section_header.sh_flag = match is_little {
            true => u64::from_le_bytes([buffer[8], buffer[9], buffer[10], buffer[11], buffer[12], buffer[13], buffer[14], buffer[15]]),
            false => u64::from_be_bytes([buffer[8], buffer[9], buffer[10], buffer[11], buffer[12], buffer[13], buffer[14], buffer[15]]),
        };

        section_header.sh_addr = match is_little {
            true => u64::from_le_bytes([buffer[16], buffer[17], buffer[18], buffer[19], buffer[20], buffer[21], buffer[22], buffer[23]]),
            false => u64::from_be_bytes([buffer[16], buffer[17], buffer[18], buffer[19], buffer[20], buffer[21], buffer[22], buffer[23]]),
        };

        section_header.sh_offset = match is_little {
            true => u64::from_le_bytes([buffer[24], buffer[25], buffer[26], buffer[27], buffer[28], buffer[29], buffer[30], buffer[31]]),
            false => u64::from_be_bytes([buffer[24], buffer[25], buffer[26], buffer[27], buffer[28], buffer[29], buffer[30], buffer[31]]),
        };

        section_header.sh_size = match is_little {
            true => u64::from_le_bytes([buffer[32], buffer[33], buffer[34], buffer[35], buffer[36], buffer[37], buffer[38], buffer[39]]),
            false => u64::from_be_bytes([buffer[32], buffer[33], buffer[34], buffer[35], buffer[36], buffer[37], buffer[38], buffer[39]]),
        };

        section_header.sh_link = match is_little {
            true => u32::from_le_bytes([buffer[40], buffer[41], buffer[42], buffer[43]]),
            false => u32::from_be_bytes([buffer[40], buffer[41], buffer[42], buffer[43]]),
        };

        section_header.sh_info = match is_little {
            true => u32::from_le_bytes([buffer[44], buffer[45], buffer[46], buffer[47]]),
            false => u32::from_be_bytes([buffer[44], buffer[45], buffer[46], buffer[47]]),
        };

        section_header.sh_addralign = match is_little {
            true => u64::from_le_bytes([buffer[48], buffer[49], buffer[50], buffer[51], buffer[52], buffer[53], buffer[54], buffer[55]]),
            false => u64::from_be_bytes([buffer[48], buffer[49], buffer[50], buffer[51], buffer[52], buffer[53], buffer[54], buffer[55]]),
        };

        section_header.sh_entsize = match is_little {
            true => u64::from_le_bytes([buffer[56], buffer[57], buffer[58], buffer[59], buffer[60], buffer[61], buffer[62], buffer[63]]),
            false => u64::from_be_bytes([buffer[56], buffer[57], buffer[58], buffer[59], buffer[60], buffer[61], buffer[62], buffer[63]]),
        };

        Ok(section_header)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut vec = Vec::with_capacity(64); // Section header is 64 bytes

        for byte in self.sh_name.to_be_bytes().iter() {
            vec.push(byte.clone());
        }

        for byte in self.sh_type.to_be_bytes().iter() {
            vec.push(byte.clone());
        }

        for byte in self.sh_flag.to_be_bytes().iter() {
            vec.push(byte.clone());
        }
        for byte in self.sh_addr.to_be_bytes().iter() {
            vec.push(byte.clone());
        }
        for byte in self.sh_offset.to_be_bytes().iter() {
            vec.push(byte.clone());
        }
        for byte in self.sh_size.to_be_bytes().iter() {
            vec.push(byte.clone());
        }
        for byte in self.sh_link.to_be_bytes().iter() {
            vec.push(byte.clone());
        }
        for byte in self.sh_info.to_be_bytes().iter() {
            vec.push(byte.clone());
        }
        for byte in self.sh_addralign.to_be_bytes().iter() {
            vec.push(byte.clone());
        }
        for byte in self.sh_entsize.to_be_bytes().iter() {
            vec.push(byte.clone());
        }
        vec
    }
}

#[derive(Debug, Default)]
pub struct SectionTable {
    pub sections: Vec<SectionEntry64>,
    pub stringTable: SectionEntry64,
    pub elf: RawELFFile
}

impl SectionTable {
    pub fn new() -> SectionTable {
        SectionTable::default()
    }

    pub fn from(elf: &RawELFFile) -> Result<SectionTable, Error> {

        let elfheader = ELFHeader::from(elf).unwrap();
        let string_table_index = elfheader.e_shstrndx;
        let section_entry_num = elfheader.e_shnum;
        let mut section_table = Vec::new();
        let mut index = 0usize;
        
        while index < section_entry_num as usize {
            section_table.push(SectionEntry64::from(elf, index).unwrap());
            index += 1;
        }
        Ok(SectionTable {
            sections: section_table,
            stringTable: SectionEntry64::from(elf, string_table_index as usize).unwrap(),
            elf: elf.clone()
        })
    }

    pub fn getName(&self, index: u32) -> Vec<u8> {
        let stringTable = &self.stringTable;

        let mut offset = stringTable.sh_offset as usize;
        let size = stringTable.sh_size as usize;
        let mut buffer = Vec::new();
        let file_img = &self.elf;
        let end = offset + size;
        while offset < end {
            buffer.push((&self.elf)[offset].clone());
            offset += 1;
        }
        let mut i = index as usize;
        let mut name = Vec::new();
        loop {
            if (&buffer)[i].clone() != 0u8 {
                name.push((&buffer)[i].clone());
            } else {
                break
            }
            i += 1;
        }
        name
    }

    pub fn printSection(&self, index:usize) {
        let section = &self.sections[index];
        let name = self.getName(section.sh_name);
        kprintln!("Name:                            {:?}", core::str::from_utf8(&name).unwrap());
        kprint!("Type:                              ");
        match section.sh_type {
            0x0	=> {kprint!("SHT_NULL");},	
            0x1	=> {kprint!("SHT_PROGBITS");},	
            0x2	=> {kprint!("SHT_SYMTAB");},
            0x3	=> {kprint!("SHT_STRTAB");},
            0x4	=> {kprint!("SHT_RELA");},
            0x5	=> {kprint!("SHT_HASH");},	
            0x6	=> {kprint!("SHT_DYNAMIC");},	
            0x7	=> {kprint!("SHT_NOTE");},	
            0x8	=> {kprint!("SHT_NOBITS");},
            0x9	=> {kprint!("SHT_REL");},	
            0x0A =>	{kprint!("SHT_SHLIB");},	
            0x0B =>	{kprint!("SHT_DYNSYM");},	
            0x0E =>	{kprint!("SHT_INIT_ARRAY");},
            0x0F =>	{kprint!("SHT_FINI_ARRAY");},
            0x10 =>	{kprint!("SHT_PREINIT_ARRAY");},
            0x11 =>	{kprint!("SHT_GROUP");},
            0x12 =>	{kprint!("SHT_SYMTAB_SHNDX");},
            0x13 =>	{kprint!("SHT_NUM");},
            0x60000000	=> {kprint!("SHT_LOOS");},
            _ => {kprint!("Unknown");}	
        }
        kprintln!("");
    }

    pub fn printSectionTable(&self) {
        let length = (&self.sections).len();
        let mut i = 0;
        while i < length {
            self.printSection(i);
            i += 1;
        }
    }
}
