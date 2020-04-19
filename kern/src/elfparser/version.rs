//https://refspecs.linuxfoundation.org/LSB_3.0.0/LSB-PDA/LSB-PDA.junk/symversion.html
use shim::const_assert_size;
use core::fmt::Error;
use alloc::vec::Vec;
use crate::console::{kprintln};
use crate::elfparser::section::{SectionTable, SectionEntry64};

#[derive(Debug, Default, Clone)]
pub struct Verneed64 {
    pub version: u16,  // Version of structure. This value is currently set to 1, and will be reset if the versioning implementation is incompatibly altered.
    pub cnt:     u16,  // Number of associated verneed array entries.
    pub file:    u32,  // Offset to the file name string in the section header, in bytes.
    pub aux:     u32,  // Offset to a corresponding entry in the vernaux array, in bytes.
    pub next:    u32,  // Offset to the next verneed entry, in bytes.
}

const_assert_size!(Verneed64, 16); // Verneed64 has the size of 24

impl Verneed64 {
    pub fn new() -> Verneed64 {
        Verneed64::default()
    }

    pub fn from(raw_ver_req_table: &Vec<u8>, index: usize) -> Verneed64 {
        let mut ver_need = Vec::new();

        let mut start = index * 16;
        let end = start + 16;

        while start < end {
            ver_need.push((&raw_ver_req_table)[start].clone());
            start += 1;
        }

        let mut new_ver_need = Verneed64::new();
        new_ver_need.version = u16::from_le_bytes([ver_need[0], ver_need[1]]);
        new_ver_need.cnt = u16::from_le_bytes([ver_need[2], ver_need[3]]);
        new_ver_need.file = u32::from_le_bytes([ver_need[4], ver_need[5], ver_need[6], ver_need[7]]);
        new_ver_need.aux = u32::from_le_bytes([ver_need[8], ver_need[9], ver_need[10], ver_need[11]]);
        new_ver_need.next = u32::from_le_bytes([ver_need[12], ver_need[13], ver_need[14], ver_need[15]]);
        new_ver_need
    }
}

#[derive(Debug, Default, Clone)]
pub struct GnuVersionReq {
    pub verneeds: Vec<Verneed64>,
    pub section_table: SectionTable,
    pub dynamic_string_table: Vec<u8>
}

impl GnuVersionReq {
    pub fn new() -> GnuVersionReq {
        GnuVersionReq::default()
    }

    pub fn from(section_table: &SectionTable) -> Result<GnuVersionReq, Error> {
        let mut version_req: &SectionEntry64 = &SectionEntry64::new();
        let mut _i = 0;
        for section in (&section_table.sections).iter() {
            if section.sh_type == 0x6ffffffe { // symbol table type == 0x2
                version_req = section;
                break;
            }
            _i += 1;
        }        
        let entry_num = (version_req.sh_size as usize)/(16usize);
        let _entry_size = version_req.sh_entsize as usize;
        kprintln!("entry num = {}", entry_num);
        let raw = &section_table.elf;
        let mut raw_section_table = Vec::new();

        let mut index = version_req.sh_offset as usize;
        kprintln!("Index = {:x}", index);
        let end = index + (version_req.sh_size as usize);
        kprintln!("end = {:x}", end);
        while index < end {
            raw_section_table.push((&raw)[index].clone());
            index += 1;
        }
        let mut new_version_req = GnuVersionReq::new();

        let mut start = 0usize;
        while start < entry_num {
            kprintln!("start {} vs entry num {}", start, entry_num);
            new_version_req.verneeds.push(Verneed64::from(&raw_section_table, start));
            start += 1;
        }
        kprintln!("Got here?");
        new_version_req.section_table = section_table.clone();
        let mut dynamic_string_table = &SectionEntry64::new();

        for section in section_table.sections.iter() {
            if section.sh_type == 0x3 {
                let name = section_table.get_name(section.sh_name);
                if core::str::from_utf8(&name).unwrap() == ".dynstr" {
                    dynamic_string_table = section;
                }
            }
        }   
        kprintln!("Done string");

        let mut offset = dynamic_string_table.sh_offset as usize;
        kprintln!("0x{:x}", offset);
        let size = dynamic_string_table.sh_size as usize;
        let mut buffer = Vec::new();
        let end = offset + size;
        while offset < end {
            buffer.push((&section_table.elf)[offset].clone());
            offset += 1;
        }

        new_version_req.dynamic_string_table = buffer;
        Ok(new_version_req)
    }

    pub fn get_name(&self, index: u32) -> Vec<u8> {
        let buffer = &self.dynamic_string_table;
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