use alloc::vec::Vec;
use crate::elfparser::header::{ProgHeader64, ELFHeader, RawELFFile};
use shim::path::{Path};
use crate::console::{kprintln, kprint};
use crate::elfparser::values::*;
// struct for ELF64
pub struct ELF {
    pub raw: RawELFFile,
    pub header: ELFHeader,
    pub header_table: Vec<ProgHeader64>
}

impl ELF {
    pub fn new() -> ELF {
        ELF {
            raw: RawELFFile::new(),
            header: ELFHeader::new(),
            header_table: Vec::new(),
        }
    }
    
    // Caller ensures to call new before initialize
    pub fn initialize<P: AsRef<Path>>(&mut self, path: P) -> bool {
        match self.raw.read_file(path) {
            0 => {
                return false;
            },
            _ => {}
        }
        self.header = match ELFHeader::from(&self.raw) {
            Ok(header) => header,
            Err(_) => {
                return false;
            }
        };
        
        let entry_num = self.header.e_phnum;
        let mut index = 0;
        while index < entry_num {
            let program_header = match ProgHeader64::from(&self.raw, index as usize) {
                Ok(prog_header) => prog_header,
                Err(_) => {
                    return false;
                }
            };
            self.header_table.push(program_header);
            index += 1;
        }
        true
    } 

    pub fn initialize_to_execute<P: AsRef<Path>>(&mut self, path: P) -> bool {
        match self.raw.read_file_to_execute(path) {
            0 => {
                return false;
            },
            _ => {}
        }
        self.header = match ELFHeader::from(&self.raw) {
            Ok(header) => header,
            Err(_) => {
                return false;
            }
        };
        
        let entry_num = self.header.e_phnum;
        let mut index = 0;
        while index < entry_num {
            let program_header = match ProgHeader64::from(&self.raw, index as usize) {
                Ok(prog_header) => prog_header,
                Err(_) => {
                    return false;
                }
            };
            self.header_table.push(program_header);
            index += 1;
        }
        true
    }

    // Print this elf files
    // Print the file header and then the program header table
    pub fn print_elf(&self) {
        self.header.print_header();
        kprintln!("");
        self.print_htable();
    }

    // Print header table
    pub fn print_htable(&self) {
        let mut num = 0;
        kprintln!("There are {} program headers, starting at offset {}", self.header_table.len(), self.header_table[0].p_offset);
        for entry in self.header_table.iter() {
            kprint!("{}.", num);
            entry.print_header();
            num += 1;
        } 
    }


    // Returns a vector of byte containing the executable binary code
    // vector length = binary length = header_table_entry.p_filesz
    pub fn binary(&self) -> Vec<u8> {
        let mut header_table_entry: &ProgHeader64 = &ProgHeader64::new();
        for entry in self.header_table.iter() {
            if entry.p_type == ProgHeaderType::LOAD { // 	PT_LOAD
                header_table_entry = entry;
                break;
            }
        };
        let binary_size = header_table_entry.p_filesz as usize;
        let start_offset = header_table_entry.p_offset as usize;
        let mut buffer = Vec::with_capacity(binary_size);
        let raw = &(self.raw.as_slice())[start_offset..(start_offset + binary_size)];

        for byte in raw.iter() {
            buffer.push(byte.clone());
        }

        return buffer;
    }
}