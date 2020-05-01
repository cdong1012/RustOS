use shim::const_assert_size;
use core::fmt::Error;
use alloc::vec::Vec;
use crate::console::{kprintln, kprint};
use core::mem::size_of;
use crate::elfparser::header::{RawELFFile, ELFHeader};
use crate::elfparser::values::*;
// Section entry struct. Entry is in the section table
#[derive(Debug, Default, Clone)]
pub struct SectionEntry64 {
    pub sh_name: u32,
    pub sh_type: u32,
    pub sh_flags: u64,
    pub sh_addr: u64,
    pub sh_offset: u64,
    pub sh_size: u64,
    pub sh_link: u32,
    pub sh_info: u32,
    pub sh_addralign: u64,
    pub sh_entsize: u64,
}
const_assert_size!(SectionEntry64, 64);     // section entry size is 64 bytes


impl SectionEntry64 {
    pub fn new() -> SectionEntry64 {
        SectionEntry64::default()
    }

    // From the raw elf files into a section entry
    // Index = index of section entry in the section table
    pub fn from(elf: &RawELFFile, index: usize) -> Result<SectionEntry64, usize> {
        let elfheader = match ELFHeader::from(elf) {
            Ok(header) => {header},
            Err(_) => {
                return Err(0usize);
            }
        };
        
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
                return Err(0usize);
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

        section_header.sh_flags = match is_little {
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
}

// Section table, stores a vector of section entry
// Also store the string table and the raw elf file
#[derive(Debug, Default, Clone)]
pub struct SectionTable {
    pub sections: Vec<SectionEntry64>,
    pub string_table: SectionEntry64,
    pub elf: RawELFFile
}

impl SectionTable {
    pub fn new() -> SectionTable {
        SectionTable::default()
    }

    // from raw elf file into section table
    pub fn from(elf: &RawELFFile) -> Result<SectionTable, usize> {
        let elfheader = match ELFHeader::from(elf) {
            Ok(header) => {header},
            Err(_) => {
                return Err(0usize);
            }
        };
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
            string_table: SectionEntry64::from(elf, string_table_index as usize).unwrap(),
            elf: elf.clone()
        })
    }

    // Returns a vector of u8 as the name of the section entry 
    // index = offset in the string table of this section table.
    // note: to get the name of section entry "a", do: let name = self.get_name(a.sh_name);
    pub fn get_name(&self, index: u32) -> Vec<u8> {
        let string_table = &self.string_table;
    
        let mut offset = string_table.sh_offset as usize;
        let size = string_table.sh_size as usize;
        let mut buffer = Vec::new();
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

    // Print a section entry out
    // index = index in the section table
    pub fn print_section(&self, index:usize) {
        let section = &self.sections[index];
        let name = self.get_name(section.sh_name);
        kprintln!("    Name:                            {}", core::str::from_utf8(&name).unwrap());
        kprint!("    Type:                            ");
        match section.sh_type {
            SectionType::NULL	        =>  {kprint!("NULL");},	
            SectionType::PROGBITS	    =>  {kprint!("PROGBITS");},	
            SectionType::SYMTAB	        =>  {kprint!("SYMTAB");},
            SectionType::STRTAB	        =>  {kprint!("STRTAB");},
            SectionType::RELA	        =>  {kprint!("RELA");},
            SectionType::HASH	        =>  {kprint!("HASH");},	
            SectionType::DYNAMIC	    =>  {kprint!("DYNAMIC");},	
            SectionType::NOTE	        =>  {kprint!("NOTE");},	
            SectionType::NOBITS	        =>  {kprint!("NOBITS");},
            SectionType::REL	        =>  {kprint!("REL");},	
            SectionType::SHLIB          =>  {kprint!("SHLIB");},	
            SectionType::DYNSYM         =>	{kprint!("DYNSYM");},	
            SectionType::INIT_ARRAY     =>	{kprint!("INIT_ARRAY");},
            SectionType::FINI_ARRAY     =>	{kprint!("FINI_ARRAY");},
            SectionType::PREINIT_ARRAY  =>	{kprint!("PREINIT_ARRAY");},
            SectionType::GROUP          =>	{kprint!("GROUP");},
            SectionType::SYMTAB_SHNDX   =>	{kprint!("SYMTAB_SHNDX");},
            SectionType::NUM            =>	{kprint!("NUM");},
            SectionType::LOOS	        =>  {kprint!("LOOS");},
            SectionType::VERDEF         =>  {kprint!("VERDEF");},
            SectionType::VERNEED        =>  {kprint!("VERNEED");},
            SectionType::VERSYM         =>  {kprint!("VERSYM");},
            _                           =>  {kprint!("Unknown");}	
        }
        kprintln!("");

        kprint!("    Flag:                            ");
        match section.sh_flags {
            SectionFlag::WRITE	            =>  {kprint!("WRITE");},
            SectionFlag::ALLOC	            =>  {kprint!("ALLOC");},
            SectionFlag::EXECINSTR	        =>  {kprint!("EXECINSTR");},
            SectionFlag::MERGE              =>	{kprint!("MERGE");},	
            SectionFlag::STRINGS            =>	{kprint!("STRINGS");},
            SectionFlag::INFO_LINK          =>	{kprint!("INFO_LINK");},
            SectionFlag::LINK_ORDER         =>	{kprint!("LINK_ORDER");},
            SectionFlag::OS_NONCONFORMING   =>  {kprint!("OS_NONCONFORMING");},
            SectionFlag::GROUP              =>  {kprint!("GROUP");},
            SectionFlag::TLS                =>  {kprint!("TLS");},
            SectionFlag::MASKOS	            =>  {kprint!("MASKOS");},
            SectionFlag::MASKPROC	        =>  {kprint!("MASKPROC");},
            SectionFlag::ORDERED	        =>  {kprint!("ORDERED");},
            SectionFlag::EXCLUDE	        =>  {kprint!("EXCLUDE");},
            _                               =>  {kprint!("Unknown");},	
        }
        kprintln!("");

        kprintln!("    Address of section in memory:    0x{:x}", section.sh_addr);
        kprintln!("    Offset of section in file image: 0x{:x}", section.sh_offset);
        kprintln!("    Size of section:                 {}", section.sh_size);
        kprintln!("    Index of associated section:     {}", section.sh_link);
        kprintln!("    Alignment:                       0x{:x}", section.sh_addralign);
        kprintln!("    Entry size:                      {}", section.sh_entsize);
    }

    pub fn print_section_table(&self) { // readelf -S 
        let length = (&self.sections).len();
        let mut i = 0;
        while i < length {
            kprintln!("Section {}.", i);
            self.print_section(i);
            i += 1;
        }
    }
}
