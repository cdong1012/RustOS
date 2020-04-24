use shim::const_assert_size;
use core::fmt::Error;
use alloc::vec::Vec;
use crate::console::{kprintln, kprint};
use crate::elfparser::section::{SectionTable, SectionEntry64};
use crate::elfparser::values::*;
//https://docs.oracle.com/cd/E23824_01/html/819-0690/chapter6-42444.html
#[derive(Debug, Default, Clone)]
pub struct Dyn64 {
    pub d_tag: u64,
    pub d_un: u64,
}    
const_assert_size!(Dyn64, 16);

impl Dyn64 {
    pub fn new() -> Dyn64 {
        Dyn64::default()
    }

    pub fn from(raw_dyn_table: &Vec<u8>, index: usize) -> Dyn64 {
        let mut dyn64 = Vec::new();

        let mut start = index * 16;
        let end = start + 16;

        while start < end {
            dyn64.push((&raw_dyn_table)[start].clone());
            start += 1;
        }

        let mut new_dyn64 = Dyn64::new();
        new_dyn64.d_tag = u64::from_le_bytes([dyn64[0], dyn64[1], dyn64[2], dyn64[3], dyn64[4], dyn64[5], dyn64[6], dyn64[7]]);
        new_dyn64.d_un = u64::from_le_bytes([dyn64[8], dyn64[9], dyn64[10], dyn64[11], dyn64[12], dyn64[13], dyn64[14], dyn64[15]]);
        new_dyn64
    }
}

#[derive(Debug, Default, Clone)]
pub struct DynamicTable {
    pub dyns: Vec<Dyn64>,
    pub section_table: SectionTable,
    pub shared_lib_string: Vec<u8>
}

impl DynamicTable {
    pub fn new() -> DynamicTable {
        DynamicTable::default()
    }

    pub fn from(section_table: &SectionTable) -> Result<DynamicTable, usize> {

        let mut dynamic_table: &SectionEntry64 = &SectionEntry64::new();
        let mut _i = 0;
        let mut detected = false;
        for section in (&section_table.sections).iter() {
            if section.sh_type == SectionType::DYNAMIC { 
                dynamic_table = section;
                detected = true;
                break;
            }
            _i += 1;
        }       
        if !detected {
            return Err(0usize);
        }
        
        let entry_size = dynamic_table.sh_entsize as usize;
        let entry_num = (dynamic_table.sh_size as usize)/(entry_size);
        
        let raw = &section_table.elf;
        let mut raw_section_table = Vec::new();

        let mut index = dynamic_table.sh_offset as usize;
        let end = index + (dynamic_table.sh_size as usize);
        while index < end {
            raw_section_table.push((&raw)[index].clone());
            index += 1;
        }
        let mut new_dynamic_table = DynamicTable::new();

        let mut start = 0usize;
        while start < entry_num {
            new_dynamic_table.dyns.push(Dyn64::from(&raw_section_table, start));
            start += 1;
        }

        let mut shared_lib_string = &SectionEntry64::new();

        for section in section_table.sections.iter() {
            if section.sh_type == SectionType::STRTAB {
                let name = section_table.get_name(section.sh_name);
                if core::str::from_utf8(&name).unwrap() == ".dynstr" {
                    shared_lib_string = section;
                }
            }
        }
        let mut offset = shared_lib_string.sh_offset as usize;
        let size = shared_lib_string.sh_size as usize;
        let mut buffer = Vec::new();
        let end = offset + size;
        while offset < end {
            buffer.push((&section_table.elf)[offset].clone());
            offset += 1;
        }
        new_dynamic_table.shared_lib_string = buffer.clone();
        Ok(new_dynamic_table)
    }

    pub fn print_dyn_table(&self) {
        let mut i = 0;
        while i < self.dyns.len() {
            if self.dyns[i].d_tag == DynTag::DT_NULL {
                break;
            }
            i += 1
        }
        kprintln!("Dynamic section .dynamic contains {} entries:", i + 1);
        kprintln!("Tag          Type        Name/Value");
        i = 0;
        while i < self.dyns.len() {
            self.print_dyn(i);
            if self.dyns[i].d_tag == DynTag::DT_NULL {
                break;
            }
            i += 1;
        }
    }

    pub fn print_dyn(&self, index: usize) {
        let dyn64 = (&self.dyns)[index].clone();
        let tag = dyn64.d_tag;
        
        kprint!("0x{:x}     ", tag);

        match tag {
            DynTag::DT_NULL             => {kprint!("(NULL)");},
            DynTag::DT_NEEDED           => {kprint!("(NEEDED)");},
            DynTag::DT_PLTRELSZ         => {kprint!("(PLTRELSZ)");},
            DynTag::DT_PLTGOT           => {kprint!("(PLTGOT)");},
            DynTag::DT_HASH             => {kprint!("(HASH)");},
            DynTag::DT_STRTAB           => {kprint!("(STRTAB)");},
            DynTag::DT_SYMTAB           => {kprint!("(SYMTAB)");},
            DynTag::DT_RELA             => {kprint!("(RELA)");},
            DynTag::DT_RELASZ           => {kprint!("(RELASZ)");},
            DynTag::DT_RELAENT          => {kprint!("(RELAENT)");},
            DynTag::DT_STRSZ            => {kprint!("(STRSZ)");},
            DynTag::DT_SYMENT           => {kprint!("(SYMENT)");},
            DynTag::DT_INIT             => {kprint!("(INIT)");},
            DynTag::DT_FINI             => {kprint!("(FINI)");},
            DynTag::DT_SONAME           => {kprint!("(SONAME)");},
            DynTag::DT_RPATH            => {kprint!("(RPATH)");},
            DynTag::DT_SYMBOLIC         => {kprint!("(SYMBOLIC)");},
            DynTag::DT_REL              => {kprint!("(REL)");},
            DynTag::DT_RELSZ            => {kprint!("(RELSZ)");},
            DynTag::DT_RELENT           => {kprint!("(RELENT)");},
            DynTag::DT_PLTREL           => {kprint!("(PLTREL)");},
            DynTag::DT_DEBUG            => {kprint!("(DEBUG)");},
            DynTag::DT_TEXTREL          => {kprint!("(TEXTREL)");},
            DynTag::DT_JMPREL           => {kprint!("(JMPRE)L");},
            DynTag::DT_BIND_NOW         => {kprint!("(BIND_NOW)");},
            DynTag::DT_INIT_ARRAY       => {kprint!("(INIT_ARRAY)");},
            DynTag::DT_FINI_ARRAY       => {kprint!("(FINI_ARRAY)");},
            DynTag::DT_INIT_ARRAYSZ     => {kprint!("(INIT_ARRAYSZ)");},
            DynTag::DT_FINI_ARRAYSZ     => {kprint!("(FINI_ARRAYSZ)");},
            DynTag::DT_RUNPATH          => {kprint!("(RUNPATH)");},
            DynTag::DT_FLAGS            => {kprint!("(FLAGS)");},
            DynTag::DT_ENCODINGS        => {kprint!("(ENCODINGS)");},
            DynTag::DT_PREINIT_ARRAYSZ  => {kprint!("(PREINIT_ARRAYSZ)");},
            DynTag::DT_FLAGS_1          => {kprint!("(FLAGS_1)");},
            _                           => {kprint!("(UNKNOWN");},
        }
        kprint!("       ");
        match tag {
            DynTag::DT_NEEDED => {
                kprint!("Shared library: [{}]", core::str::from_utf8(&self.get_dependency(dyn64.d_un)).unwrap());
            },
            DynTag::DT_INIT_ARRAYSZ | DynTag::DT_FINI_ARRAYSZ | DynTag::DT_STRSZ | DynTag::DT_SYMENT | DynTag::DT_PLTRELSZ | DynTag::DT_RELASZ | DynTag::DT_RELAENT
                              => {
                kprint!("{} (bytes)", dyn64.d_un);
            }
            DynTag::DT_PLTREL => {
                kprint!("Rela is {}", dyn64.d_un);
            },
            DynTag::DT_FLAGS => {
                match dyn64.d_un {
                    DynFlag::DF_ORIGIN      => {kprint!("ORIGIN");},
                    DynFlag::DF_SYMBOLIC    => {kprint!("SYMBOLIC");},
                    DynFlag::DF_TEXTREL     => {kprint!("TEXTREL");},
                    DynFlag::DF_BIND_NOW    => {kprint!("BIND_NOW");},
                    DynFlag::DF_STATIC_TLS  => {kprint!("STATIC_TLS");},
                    _                       => {kprint!("UNKNOWN");},
                }
            }, 
            DynTag::DT_FLAGS_1 => {
                match dyn64.d_un {
                    DynFlag1::DF_1_NOW => {kprint!("NOW");},
                    DynFlag1::DF_1_GLOBAL => {kprint!("GLOBAL");},
                    DynFlag1::DF_1_GROUP => {kprint!("GROUP");},
                    DynFlag1::DF_1_NODELETE => {kprint!("NODELETE");},
                    DynFlag1::DF_1_LOADFLTR => {kprint!("LOADFLTR");},
                    DynFlag1::DF_1_INITFIRST => {kprint!("INITFIRST");},
                    DynFlag1::DF_1_NOOPEN => {kprint!("NOOPEN");},
                    DynFlag1::DF_1_ORIGIN => {kprint!("ORIGIN");},
                    DynFlag1::DF_1_DIRECT => {kprint!("DIRECT");},
                    DynFlag1::DF_1_INTERPOSE => {kprint!("INTERPOSE");},
                    DynFlag1::DF_1_NODEFLIB => {kprint!("NODEFLIB");},
                    DynFlag1::DF_1_NODUMP => {kprint!("NODUMP");},
                    DynFlag1::DF_1_CONFALT => {kprint!("CONFALT");},
                    DynFlag1::DF_1_ENDFILTEE => {kprint!("ENDFILTEE");},
                    DynFlag1::DF_1_DISPRELDNE => {kprint!("DISPRELDNE");},
                    DynFlag1::DF_1_DISPRELPND => {kprint!("DISPRELPND");},
                    DynFlag1::DF_1_NODIRECT => {kprint!("NODIRECT");},
                    DynFlag1::DF_1_IGNMULDEF => {kprint!("IGNMULDEF");},
                    DynFlag1::DF_1_NOKSYMS => {kprint!("NOKSYMS");},
                    DynFlag1::DF_1_NOHDR => {kprint!("NOHDR");},
                    DynFlag1::DF_1_EDITED => {kprint!("EDITED");},
                    DynFlag1::DF_1_NORELOC => {kprint!("NORELOC");},
                    DynFlag1::DF_1_SYMINTPOSE => {kprint!("SYMINTPOSE");},
                    DynFlag1::DF_1_GLOBAUDIT => {kprint!("GLOBAUDIT");},
                    DynFlag1::DF_1_SINGLETON => {kprint!("SINGLETON");},
                    _ => {kprint!("UNKNOWN");},
                }
            }
            _ => {
                kprint!("0x{:x}", dyn64.d_un);
            }
        }
        kprintln!("");
    }


    pub fn get_dependency(&self, index: u64) -> Vec<u8> {
        let buffer = &self.shared_lib_string;
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