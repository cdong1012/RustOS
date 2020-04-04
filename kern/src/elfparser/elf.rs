use alloc::vec::Vec;
use crate::elfparser::header::{ProgHeader64, ELFHeader, RawELFFile};
use shim::path::{Path, PathBuf};

// for ELF64
pub struct ELF {
    pub raw: RawELFFile,
    pub header: ELFHeader,
    pub headerTable: Vec<ProgHeader64>
}

impl ELF {
    pub fn new() -> ELF {
        ELF {
            raw: RawELFFile::new(),
            header: ELFHeader::new(),
            headerTable: Vec::new(),
        }
    }

    pub fn initialize<P: AsRef<Path>>(&mut self, path: P) {
        self.raw.read_file(path);
        self.header = ELFHeader::from(&self.raw).unwrap();
        let entry_num = self.header.e_phnum;
        let mut index = 0;
        while index < entry_num {
            let program_header = ProgHeader64::from(&self.raw, index as usize).unwrap();
            self.headerTable.push(program_header);
            index += 1;
        }
    } 

    pub fn print_elf(&self) {
        self.header.print_header();
        for entry in self.headerTable.iter() {
            entry.print_program_header();
        }
    }
}