use shim::const_assert_size;
use core::fmt::Error;
use alloc::vec::Vec;
use crate::console::{kprintln, kprint};
use crate::elfparser::section::{SectionTable, SectionEntry64};
use crate::elfparser::values::*;
use crate::elfparser::version::{GnuVersion};
use crate::elfparser::symbol::{DynamicSymbolTable};
use crate::alloc::string::ToString;
// https://www.ucw.cz/~hubicka/papers/abi/node19.html

//https://docs.oracle.com/cd/E19683-01/816-1386/chapter6-54839/index.html
#[derive(Debug, Default, Clone)]
pub struct Rela64 {
    pub r_offset: u64, 
    pub r_info: u64,    
    pub r_addend: i64,   
}

const_assert_size!(Rela64, 24);

impl Rela64 {
    pub fn new() -> Rela64 {
        Rela64::default()
    }

    pub fn from(raw_section_table: &Vec<u8>, index: usize) -> Rela64 {
        let mut rela = Vec::new();

        let mut start = index * 24;
        let end = start + 24;
        while start < end {
            rela.push((&raw_section_table)[start].clone());
            start += 1;
        }
        let mut new_rela = Rela64::new();

        new_rela.r_offset = u64::from_le_bytes([rela[0], rela[1], rela[2], rela[3], rela[4], rela[5], rela[6], rela[7]]);

        new_rela.r_info = u64::from_le_bytes([rela[8], rela[9], rela[10], rela[11], rela[12], rela[13], rela[14], rela[15]]);

        new_rela.r_addend = i64::from_le_bytes([rela[16], rela[17], rela[18], rela[19], rela[20], rela[21], rela[22], rela[23]]);
        
        new_rela
    }

    // Get the sym value from st_info
    pub fn get_sym(&self) -> u64 {
        self.r_info >> 32
    }
    
    // Get the type value from st_info
    pub fn get_type(&self) -> u64 {
        self.r_info & 0xffffffff
    }
}

#[derive(Debug, Default, Clone)]
pub struct RelaTable {
    pub relas: Vec<Rela64>,
    pub section_table: SectionTable,
    pub rela_string_table: Vec<Vec<u8>>
}

// .rela.dyn
impl RelaTable {
    pub fn new() -> RelaTable {
        RelaTable::default()
    }

    pub fn from(section_table: &SectionTable) -> Result<RelaTable, usize> {
        let mut symbol_table: &SectionEntry64 = &SectionEntry64::new();
        let mut _i = 0;
        let mut detected = false;
        for section in (&section_table.sections).iter() {
            if section.sh_type == SectionType::RELA {
                let name = section_table.get_name(section.sh_name);
                if core::str::from_utf8(&name).unwrap() == ".rela.dyn" {
                    symbol_table = section;
                    detected = true;
                    break;
                }
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

        let mut new_rela_table = RelaTable::new();

        let mut start = 0usize;
        while start < entry_num {
            new_rela_table.relas.push(Rela64::from(&raw_section_table, start));
            start += 1;
        }
        
        new_rela_table.section_table = section_table.clone();
        let mut dyn_sym_table = DynamicSymbolTable::from(&section_table).unwrap();
        new_rela_table.rela_string_table = dyn_sym_table.get_dynamic_string_table();
        Ok(new_rela_table)
    }

    pub fn print_rela_table(&self) {
        kprintln!("Relocation section '.rela.dyn' contains {} entries:", self.relas.len());
        let mut i = 0;
        kprintln!("Index   Offset           Info          type               sym     name + append");
        while i < self.relas.len() {
            self.print_rela(i);
            i += 1;
        }
    }

    pub fn print_rela(&self, index: usize) {
        let rela = self.relas[index].clone();

        let offset = rela.r_offset;
        let info = rela.r_info;
        let rela_type = rela.get_type();
        let sym = rela.get_sym();
        let name = &self.get_name(sym as usize).clone();
        kprint!("{}:", index);
        let mut i = 0;
        while i + index.to_string().len() < 7 {
            kprint!(" ");
            i += 1;
        }
        kprint!("{:x}   ", offset);
        kprint!("{:?}   ", info);
        i = 0;
        while i + info.to_string().len() < 14 {
            kprint!(" ");
            i += 1;
        }
        match rela_type {
            RelaType::R_X86_64_NONE => {kprint!("R_X86_64_NONE");},
            RelaType::R_X86_64_64 => {kprint!("R_X86_64_64");},
            RelaType::R_X86_64_PC32 => {kprint!("R_X86_64_PC32");},
            RelaType::R_X86_64_GOT32 => {kprint!("R_X86_64_GOT32");},
            RelaType::R_X86_64_PLT32 => {kprint!("R_X86_64_PLT32");},
            RelaType::R_X86_64_COPY => {kprint!("R_X86_64_COPY");},
            RelaType::R_X86_64_GLOB_DAT => {kprint!("R_X86_64_GLOB_DAT");},
            RelaType::R_X86_64_JUMP_SLOT => {kprint!("R_X86_64_JUMP_SLOT");},
            RelaType::R_X86_64_RELATIVE => {kprint!("R_X86_64_RELATIVE");},
            RelaType::R_X86_64_GOTPCREL => {kprint!("R_X86_64_GOTPCREL");},
            RelaType::R_X86_64_32 => {kprint!("R_X86_64_32");},
            RelaType::R_X86_64_32S => {kprint!("R_X86_64_32S");},
            RelaType::R_X86_64_16 => {kprint!("R_X86_64_16");},
            RelaType::R_X86_64_PC16 => {kprint!("R_X86_64_PC16");},
            RelaType::R_X86_64_8 => {kprint!("R_X86_64_8");},
            RelaType::R_X86_64_PC8 => {kprint!("R_X86_64_PC8");},
            RelaType::R_X86_64_DPTMOD64 => {kprint!("R_X86_64_DPTMOD64");},
            RelaType::R_X86_64_DTPOFF64 => {kprint!("R_X86_64_DTPOFF64");},
            RelaType::R_X86_64_TPOFF64 => {kprint!("R_X86_64_TPOFF64");},
            RelaType::R_X86_64_TLSGD => {kprint!("R_X86_64_TLSGD");},
            RelaType::R_X86_64_TLSLD => {kprint!("R_X86_64_TLSLD");},
            RelaType::R_X86_64_DTPOFF32 => {kprint!("R_X86_64_DTPOFF32");},
            RelaType::R_X86_64_GOTTPOFF => {kprint!("R_X86_64_GOTTPOFF");},
            RelaType::R_X86_64_TPOFF32 => {kprint!("R_X86_64_TPOFF32");},
            _ => {kprint!("UNKNOWN");},
        }

        i = 0;
        while i + rela_type.to_string().len() < 7 {
            kprint!(" ");
            i += 1;
        }
        kprint!("{} ", sym);
        i = 0;
        while i + sym.to_string().len() < 12 {
            kprint!(" ");
            i += 1;
        }
        if name.len() == 0 {
            kprintln!("{:x}", rela.r_addend);
        } else {
            kprintln!("{} + {:x}", core::str::from_utf8(&name).unwrap(), rela.r_addend);
        }
    }

    pub fn get_name(&self, index: usize) -> Vec<u8> {
        self.rela_string_table[index].clone()
    }
}

#[derive(Debug, Default, Clone)]
pub struct RelaPLT {
    pub relas: Vec<Rela64>,
    pub section_table: SectionTable,
    pub rela_string_table: Vec<Vec<u8>>
}

// .rela.plt
impl RelaPLT {
    pub fn new() -> RelaPLT {
        RelaPLT::default()
    }

    pub fn from(section_table: &SectionTable) -> Result<RelaPLT, usize> {
        let mut symbol_table: &SectionEntry64 = &SectionEntry64::new();
        let mut _i = 0;
        let mut detected = false;
        for section in (&section_table.sections).iter() {
            if section.sh_type == SectionType::RELA { 
                let name = section_table.get_name(section.sh_name);
                if core::str::from_utf8(&name).unwrap() == ".rela.plt" {
                    symbol_table = section;
                    detected = true;
                    break;
                }
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
        
        let mut new_rela_plt = RelaPLT::new();

        let mut start = 0usize;
        while start < entry_num {
            new_rela_plt.relas.push(Rela64::from(&raw_section_table, start));
            start += 1;
        }
        
        new_rela_plt.section_table = section_table.clone();
        let mut dyn_sym_table = DynamicSymbolTable::from(&section_table).unwrap();
        new_rela_plt.rela_string_table = dyn_sym_table.get_dynamic_string_table();
        Ok(new_rela_plt)
    }

    pub fn get_name(&self, index: usize) -> Vec<u8> {
        self.rela_string_table[index].clone()
    }

    pub fn print_rela_plt(&self) {
        kprintln!("Relocation section '.rela.plt' contains {} entries:", self.relas.len());
        let mut i = 0;
        kprintln!("Index   Offset           Info          type               sym     name + append");
        while i < self.relas.len() {
            self.print_rela(i);
            i += 1;
        }
    }

    pub fn print_rela(&self, index: usize) {
        let rela = self.relas[index].clone();

        let offset = rela.r_offset;
        let info = rela.r_info;
        let rela_type = rela.get_type();
        let sym = rela.get_sym();
        let name = &self.get_name(sym as usize).clone();
        kprint!("{}:", index);
        let mut i = 0;
        while i + index.to_string().len() < 7 {
            kprint!(" ");
            i += 1;
        }
        kprint!("{:x}   ", offset);
        kprint!("{:?}   ", info);
        i = 0;
        while i + info.to_string().len() < 14 {
            kprint!(" ");
            i += 1;
        }
        match rela_type {
            RelaType::R_X86_64_NONE => {kprint!("R_X86_64_NONE");},
            RelaType::R_X86_64_64 => {kprint!("R_X86_64_64");},
            RelaType::R_X86_64_PC32 => {kprint!("R_X86_64_PC32");},
            RelaType::R_X86_64_GOT32 => {kprint!("R_X86_64_GOT32");},
            RelaType::R_X86_64_PLT32 => {kprint!("R_X86_64_PLT32");},
            RelaType::R_X86_64_COPY => {kprint!("R_X86_64_COPY");},
            RelaType::R_X86_64_GLOB_DAT => {kprint!("R_X86_64_GLOB_DAT");},
            RelaType::R_X86_64_JUMP_SLOT => {kprint!("R_X86_64_JUMP_SLOT");},
            RelaType::R_X86_64_RELATIVE => {kprint!("R_X86_64_RELATIVE");},
            RelaType::R_X86_64_GOTPCREL => {kprint!("R_X86_64_GOTPCREL");},
            RelaType::R_X86_64_32 => {kprint!("R_X86_64_32");},
            RelaType::R_X86_64_32S => {kprint!("R_X86_64_32S");},
            RelaType::R_X86_64_16 => {kprint!("R_X86_64_16");},
            RelaType::R_X86_64_PC16 => {kprint!("R_X86_64_PC16");},
            RelaType::R_X86_64_8 => {kprint!("R_X86_64_8");},
            RelaType::R_X86_64_PC8 => {kprint!("R_X86_64_PC8");},
            RelaType::R_X86_64_DPTMOD64 => {kprint!("R_X86_64_DPTMOD64");},
            RelaType::R_X86_64_DTPOFF64 => {kprint!("R_X86_64_DTPOFF64");},
            RelaType::R_X86_64_TPOFF64 => {kprint!("R_X86_64_TPOFF64");},
            RelaType::R_X86_64_TLSGD => {kprint!("R_X86_64_TLSGD");},
            RelaType::R_X86_64_TLSLD => {kprint!("R_X86_64_TLSLD");},
            RelaType::R_X86_64_DTPOFF32 => {kprint!("R_X86_64_DTPOFF32");},
            RelaType::R_X86_64_GOTTPOFF => {kprint!("R_X86_64_GOTTPOFF");},
            RelaType::R_X86_64_TPOFF32 => {kprint!("R_X86_64_TPOFF32");},
            _ => {kprint!("UNKNOWN");},
        }

        i = 0;
        while i + rela_type.to_string().len() < 7 {
            kprint!(" ");
            i += 1;
        }
        kprint!("{} ", sym);
        i = 0;
        while i + sym.to_string().len() < 12 {
            kprint!(" ");
            i += 1;
        }
        if name.len() == 0 {
            kprintln!("{:x}", rela.r_addend);
        } else {
            kprintln!("{} + {:x}", core::str::from_utf8(&name).unwrap(), rela.r_addend);
        }
    }
}