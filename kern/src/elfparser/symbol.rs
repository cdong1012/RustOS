//https://refspecs.linuxbase.org/elf/gabi4+/ch4.symtab.html
use shim::const_assert_size;
use core::fmt::Error;
use alloc::vec::Vec;
use crate::console::{kprintln, kprint};
use crate::elfparser::section::{SectionTable, SectionEntry64};
use crate::elfparser::version::{GnuVersion};
use crate::elfparser::values::*;

// Symbol struct in symbol table
#[derive(Debug, Default, Clone)]
pub struct Symbol64 {
    pub st_name : u32,
    pub st_info : u8,
    pub st_other: u8,
    pub st_shndx: u16,
    pub st_value: u64,
    pub st_size : u64,
}

const_assert_size!(Symbol64, 24); // symbol has the size of 24

impl Symbol64 {
    pub fn new() -> Symbol64 {
        Symbol64::default()
    }

    // FRom a raw symbol table into a symbol
    // index = index of this symbol in the table
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

    // Get the bind value from st_info
    pub fn get_bind(&self) -> u8 {
        self.st_info >> 4
    }
    
    // Get the type value from st_info
    pub fn get_type(&self) -> u8 {
        self.st_info & 0xf
    }

    // Get the vis value from st_info
    pub fn get_vis(&self) -> u8 {
        self.st_other & 0x3
    }
}

// Symbol table, stores a vector of symbols, the section table, and its own symbol string table
#[derive(Debug, Default, Clone)]
pub struct SymbolTable {
    pub symbols: Vec<Symbol64>,
    pub section_table: SectionTable,
    pub symbol_string_table: Vec<u8>
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable::default()
    }

    // From section table into symbol table
    pub fn from(section_table: &SectionTable) -> Result<SymbolTable, usize> {
        let mut symbol_table: &SectionEntry64 = &SectionEntry64::new();
        let mut _i = 0;
        let mut detected = false;
        for section in (&section_table.sections).iter() {
            if section.sh_type == SectionType::SYMTAB { // symbol table type == 0x2
                symbol_table = section;
                detected = true;
                break;
            }
            _i += 1;
        }

        if !detected {
            return Err(0usize);
        }

        let entry_num = (symbol_table.sh_size as usize)/(symbol_table.sh_entsize as usize);
        let _entry_size = symbol_table.sh_entsize as usize;

        let raw = &section_table.elf;
        let mut raw_section_table = Vec::new();

        let mut index = symbol_table.sh_offset as usize;
        let end = index + (symbol_table.sh_size as usize);
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
        new_symbol_table.section_table = section_table.clone();
        let mut symbol_string_table = &SectionEntry64::new();

        for section in section_table.sections.iter() {
            if section.sh_type == SectionType::STRTAB {
                let name = section_table.get_name(section.sh_name);
                if core::str::from_utf8(&name).unwrap() == ".strtab" {
                    symbol_string_table = section;
                }
            }
        }

        let mut offset = symbol_string_table.sh_offset as usize;
        kprintln!("0x{:x}", offset);
        let size = symbol_string_table.sh_size as usize;
        let mut buffer = Vec::new();
        let end = offset + size;
        while offset < end {
            buffer.push((&section_table.elf)[offset].clone());
            offset += 1;
        }

        new_symbol_table.symbol_string_table = buffer;
        Ok(new_symbol_table)
    }

    // Print the entire symbol table, internally call print_symbol on all the symbols
    // Similar to readelf -s
    pub fn print_symbol_table(&self) {
        let mut i = 0;
        kprintln!("Symbol table '.symtab' contains {} entries:", self.symbols.len());
        while i < self.symbols.len() {
            self.print_symbol(i);
            i += 1;
        }
        
    }

    // Print individual symbols
    // Takes in the index of the symbol in the table
    pub fn print_symbol(&self, index: usize) {
        let symbol = (&self.symbols)[index].clone();

        let name = self.get_name(symbol.st_name);
        let value = symbol.st_value;
        let size = symbol.st_size;
        let symbol_type = symbol.get_type();
        let vis = symbol.get_vis();
        let bind = symbol.get_bind();
        kprint!(" {}:", index);
        kprint!("   Value:    {:x}", value);
        kprint!("   Size:     {}", size);
        kprint!("   Type:     ");
        match symbol_type {
            SymbolType::NOTYPE  => {kprint!(" NOTYPE");},
            SymbolType::OBJECT  => {kprint!(" OBJECT");},
            SymbolType::FUNC    => {kprint!("  FUNC");},
            SymbolType::SECTION => {kprint!("SECTION");},
            SymbolType::FILE    => {kprint!("   FILE");},
            SymbolType::COMMON  => {kprint!(" COMMON");}, 
            SymbolType::TLS     => {kprint!("    TLS");},
            SymbolType::LOOS    => {kprint!("   LOOS");}, 
            SymbolType::HIOS    => {kprint!("   HIOS");},
            SymbolType::LOPROC  => {kprint!(" LOPROC");},
            SymbolType::HIPROC  => {kprint!(" HIPROC");},
            _                   => {kprint!("UNKNOWN")}
        }
        kprint!("   Bind:     ");
        match bind {
            SymbolBind::LOCAL   => {kprint!("  LOCAL");},
            SymbolBind::GLOBAL  => {kprint!(" GLOBAL");},
            SymbolBind::WEAK    => {kprint!("   WEAK");},
            SymbolBind::LOOS    => {kprint!("   LOOS");},
            SymbolBind::HIOS    => {kprint!("   HIOS");},
            SymbolBind::LOPROC  => {kprint!(" LOPROC");},
            SymbolBind::HIPROC  => {kprint!(" HIPROC");},
            _                   => {kprint!("UNKNOWN");},
        }
        kprint!("   Vis:      ");
        match vis {
            SymbolVis::DEFAULT      => {kprint!("DEFAULT");},
            SymbolVis::INTERNAL     => {kprint!("INTERNAL");},
            SymbolVis::HIDDEN       => {kprint!("HIDDEN");},
            SymbolVis::PROTECTED    => {kprint!("PROTECTED");},
            SymbolVis::EXPORTED     => {kprint!("EXPORTED");},
            SymbolVis::SINGLETON    => {kprint!("SINGLETON");},
            SymbolVis::ELIMINATE    => {kprint!("ELIMINATE");},
            _                       => {kprint!("UNKNOWN");},
        }
        if name.len() > 25 {
            kprintln!("   Name:     {}", core::str::from_utf8(&name[..25]).unwrap());
        } else {
            kprintln!("   Name:     {}", core::str::from_utf8(&name).unwrap());
        }
    }

    // Symbol table get_name, index is the offset in the string table of the symbol.
    pub fn get_name(&self, index: u32) -> Vec<u8> {
        let buffer = &self.symbol_string_table;
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
}

// TODO: dynamic symbol table
#[derive(Debug, Default, Clone)]
pub struct DynamicSymbolTable {
    pub dynamic_symbols: Vec<Symbol64>,
    pub section_table: SectionTable,
    pub symbol_string_table: Vec<u8>
}

impl DynamicSymbolTable {
    pub fn new() -> DynamicSymbolTable {
        DynamicSymbolTable::default()
    }

    pub fn from(section_table: &SectionTable) -> Result<DynamicSymbolTable, usize> {
        let mut symbol_table: &SectionEntry64 = &SectionEntry64::new();
        let mut _i = 0;
        let mut detected = false;
        for section in (&section_table.sections).iter() {
            if section.sh_type == SectionType::DYNSYM { // dynamic symbol table type == 0xB
                symbol_table = section;
                detected = true;
                break;
            }
            _i += 1;
        }

        if !detected {
            return Err(0usize);
        }

        let entry_num = (symbol_table.sh_size as usize)/(symbol_table.sh_entsize as usize);
        let _entry_size = symbol_table.sh_entsize as usize;

        let raw = &section_table.elf;
        let mut raw_section_table = Vec::new();

        let mut index = symbol_table.sh_offset as usize;
        let end = index + (symbol_table.sh_size as usize);
        while index < end {
            raw_section_table.push((&raw)[index].clone());
            index += 1;
        }
        let mut new_symbol_table = DynamicSymbolTable::new();

        let mut start = 0usize;
        while start < entry_num {
            new_symbol_table.dynamic_symbols.push(Symbol64::from(&raw_section_table, start));
            start += 1;
        }
        new_symbol_table.section_table = section_table.clone();
        let mut symbol_string_table = &SectionEntry64::new();

        for section in section_table.sections.iter() {
            if section.sh_type == SectionType::STRTAB {
                let name = section_table.get_name(section.sh_name);
                if core::str::from_utf8(&name).unwrap() == ".dynstr" {
                    symbol_string_table = section;
                }
            }
        }
        let mut offset = symbol_string_table.sh_offset as usize;
        let size = symbol_string_table.sh_size as usize;
        let mut buffer = Vec::new();
        let end = offset + size;
        while offset < end {
            buffer.push((&section_table.elf)[offset].clone());
            offset += 1;
        }
        new_symbol_table.symbol_string_table = buffer;
        Ok(new_symbol_table)
    }

    pub fn print_symbol(&self, index: usize) {
        let symbol = (&self.dynamic_symbols)[index].clone();

        let mut name = self.get_name(symbol.st_name);
        let name_version = &self.get_name_version(index);
        if name_version.len() > 0 {
            for character in name_version.iter() {
                name.push(character.clone());
            }
        }

        let value = symbol.st_value;
        let size = symbol.st_size;
        let symbol_type = symbol.get_type();
        let vis = symbol.get_vis();
        let bind = symbol.get_bind();
        kprint!(" {}:", index);
        kprint!("   Value:    {:x}", value);
        kprint!("   Size:     {}", size);
        kprint!("   Type:     ");
        match symbol_type {
            SymbolType::NOTYPE  => {kprint!(" NOTYPE");},
            SymbolType::OBJECT  => {kprint!(" OBJECT");},
            SymbolType::FUNC    => {kprint!("  FUNC");},
            SymbolType::SECTION => {kprint!("SECTION");},
            SymbolType::FILE    => {kprint!("   FILE");},
            SymbolType::COMMON  => {kprint!(" COMMON");}, 
            SymbolType::TLS     => {kprint!("    TLS");},
            SymbolType::LOOS    => {kprint!("   LOOS");}, 
            SymbolType::HIOS    => {kprint!("   HIOS");},
            SymbolType::LOPROC  => {kprint!(" LOPROC");},
            SymbolType::HIPROC  => {kprint!(" HIPROC");},
            _                   => {kprint!("UNKNOWN")}
        }
        kprint!("   Bind:     ");
        match bind {
            SymbolBind::LOCAL   => {kprint!("  LOCAL");},
            SymbolBind::GLOBAL  => {kprint!(" GLOBAL");},
            SymbolBind::WEAK    => {kprint!("   WEAK");},
            SymbolBind::LOOS    => {kprint!("   LOOS");},
            SymbolBind::HIOS    => {kprint!("   HIOS");},
            SymbolBind::LOPROC  => {kprint!(" LOPROC");},
            SymbolBind::HIPROC  => {kprint!(" HIPROC");},
            _                   => {kprint!("UNKNOWN");},
        }
        kprint!("   Vis:      ");
        match vis {
            SymbolVis::DEFAULT      => {kprint!("DEFAULT");},
            SymbolVis::INTERNAL     => {kprint!("INTERNAL");},
            SymbolVis::HIDDEN       => {kprint!("HIDDEN");},
            SymbolVis::PROTECTED    => {kprint!("PROTECTED");},
            SymbolVis::EXPORTED     => {kprint!("EXPORTED");},
            SymbolVis::SINGLETON    => {kprint!("SINGLETON");},
            SymbolVis::ELIMINATE    => {kprint!("ELIMINATE");},
            _                       => {kprint!("UNKNOWN");},
        }
        kprintln!("   Name:     {}", core::str::from_utf8(&name).unwrap());
    }

    pub fn print_dynamic_symbol_table(&self) {
        let mut i = 0;
        while i < self.dynamic_symbols.len() {
            self.print_symbol(i);
            i += 1;
        }
    }

    pub fn get_name(&self, index: u32) -> Vec<u8> {
        let buffer = &self.symbol_string_table;
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

    pub fn get_name_version(&self, index: usize) -> Vec<u8> {
        let gnu_version = GnuVersion::from(&self.section_table.clone()).unwrap();
        let version_vec = gnu_version.get_gnu_version_string();
        version_vec[index].clone()
    }

    pub fn get_dynamic_string_table(&self) -> Vec<Vec<u8>> {
        let mut string_table = Vec::new();
        let mut i = 0;
        while i < self.dynamic_symbols.len() {
            let symbol = (&self.dynamic_symbols)[i].clone();
            let mut name = self.get_name(symbol.st_name);
            let name_version = &self.get_name_version(i);
            if name_version.len() > 0 {
                for character in name_version.iter() {
                    name.push(character.clone());
                }
            }
            string_table.push(name.clone());
            i += 1;
        }
        string_table
    }
}