use shim::const_assert_size;
use core::fmt::Error;
use alloc::vec::Vec;
use shim::path::{Path, PathBuf};
use crate::FILESYSTEM;
use fat32::traits::FileSystem;
use fat32::traits::Entry;
use crate::console::{kprintln};
use core::ops::{Deref, DerefMut};

// ELFFile struct, contains a vector of raw u8s
pub struct ELFFile {
    pub raw: Vec<u8>
}

impl ELFFile {
    // ELFFile new function, initialize a new vector inside the struct and return it
    pub fn new() -> ELFFile {
        ELFFile {
            raw: Vec::new()
        }
    }

    // read_file into this elf file struct
    // Takes in a path, read it, and put it in the struct
    // 
    // returns length of the file if succeeds
    // else, panic
    pub fn read_file<P: AsRef<Path>>(&mut self, path: P) -> usize {
        let mut working_dir = PathBuf::from("/");
        let mut dir = working_dir.clone();
        dir.push(path);
        let entry = FILESYSTEM.open(dir.as_path());

        if entry.is_err() {
            kprintln!("Can't open file at path: {:?}", dir.to_str());
            panic!("Can't open file");
        }

        let entry = entry.unwrap();
        let mut buffer = [0u8; 80000];                              // change this to something huge
        let mut file_length : usize = 0usize;

        if let Some(mut file) = entry.into_file() {                 
            use shim::io::Read;
            let length = match file.read(&mut buffer) {             // read the file into the buffer
                Ok(length) => {
                    length
                },
                Err(error)=> {
                    kprintln!("Can't read file {:?}", error);
                    panic!("Can't read");
                }
            };
            file_length = length;
        }

        for byte in buffer[..file_length].iter() {                  // iterate through buffer, read it in vec
            self.raw.push(byte.clone());
        }

        file_length
    }
}

impl Deref for ELFFile {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

impl DerefMut for ELFFile {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.raw
    }
}


#[derive(Debug, Default)]
pub struct ELFHeader {
    pub ei_mag: [u8; 4],
    pub ei_class: u8,
    pub ei_data: u8,
    pub ei_version: u8,
    pub ei_osabi: u8,
    pub ei_abiversion: u8,
    pub ei_pad: [u8; 7],
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry: u64,
    pub e_phoff: u64,
    pub e_shoff: u64,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}
const_assert_size!(ELFHeader, 64); // ELF File Header is 64 bytes

impl ELFHeader {

    pub fn new() -> ELFHeader {
        ELFHeader::default()
    }

    /*
    * From function converting the raw elf file into elf header
    */
    pub fn from(mut raw: &Vec<u8>) -> Result<ELFHeader, Error> {
        let mut header = [0u8; 64];
        header.copy_from_slice(&raw[..64]);
        let elfheader : ELFHeader = unsafe {
            core::mem::transmute(header)
        };
        Ok(elfheader)
    }
}