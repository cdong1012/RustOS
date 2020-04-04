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
    pub fn from(elf: &ELFFile) -> Result<ELFHeader, Error> {
        let mut header = [0u8; 64];
        let raw = elf.as_slice();
        header.copy_from_slice(&raw[..64]);

        // can use core::mem::transmute here, but there are some undefined behavior + its unsafe
        // anyone who wanna try it out, feel free to!
        // I'm just gon do it manually.. D:
        let mut elfheader = ELFHeader::new();
        elfheader.ei_mag.copy_from_slice(&raw[..4]);
        elfheader.ei_class = raw[4].clone();
        elfheader.ei_data = raw[5].clone();
        elfheader.ei_version = raw[6].clone();
        elfheader.ei_osabi = raw[7].clone();
        elfheader.ei_abiversion = raw[8].clone();
        elfheader.ei_pad.copy_from_slice(&raw[9..16]);

        let is_little = match elfheader.ei_data {
            1 => true,
            2 => false,
            _ => {
                panic!("EI_DATA not valid");
            }
        };
        elfheader.e_type = match is_little {
            true => u16::from_le_bytes([raw[16], raw[17]]),
            false => u16::from_be_bytes([raw[16], raw[17]])
        };

        elfheader.e_machine = match is_little {
            true => u16::from_le_bytes([raw[18], raw[19]]),
            false => u16::from_be_bytes([raw[18], raw[19]])
        };
        
        elfheader.e_version = match is_little {
            true => u32::from_le_bytes([raw[20], raw[21], raw[22], raw[23]]),
            false => u32::from_be_bytes([raw[20], raw[21], raw[22], raw[23]])
        };

        elfheader.e_entry = match is_little {
            true => u64::from_le_bytes([raw[24], raw[25], raw[26], raw[27], raw[28], raw[29], raw[30], raw[31]]),
            false => u64::from_be_bytes([raw[24], raw[25], raw[26], raw[27], raw[28], raw[29], raw[30], raw[31]])
        };

        elfheader.e_phoff = match is_little {
            true => u64::from_le_bytes([raw[32], raw[33], raw[34], raw[35], raw[36], raw[37], raw[38], raw[39]]),
            false => u64::from_be_bytes([raw[32], raw[33], raw[34], raw[35], raw[36], raw[37], raw[38], raw[39]])
        };

        elfheader.e_shoff = match is_little {
            true => u64::from_le_bytes([raw[40], raw[41], raw[42], raw[43], raw[44], raw[45], raw[46], raw[47]]),
            false => u64::from_be_bytes([raw[40], raw[41], raw[42], raw[43], raw[44], raw[45], raw[46], raw[47]])
        };

        elfheader.e_flags = match is_little {
            true => u32::from_le_bytes([raw[48], raw[49], raw[50], raw[51]]),
            false => u32::from_be_bytes([raw[48], raw[49], raw[50], raw[51]])
        };

        elfheader.e_ehsize = match is_little {
            true => u16::from_le_bytes([raw[52], raw[53]]),
            false => u16::from_be_bytes([raw[52], raw[53]])
        };

        elfheader.e_phentsize = match is_little {
            true => u16::from_le_bytes([raw[54], raw[55]]),
            false => u16::from_be_bytes([raw[54], raw[55]])
        };

        elfheader.e_phnum = match is_little {
            true => u16::from_le_bytes([raw[56], raw[57]]),
            false => u16::from_be_bytes([raw[56], raw[57]])
        };

        elfheader.e_shentsize = match is_little {
            true => u16::from_le_bytes([raw[58], raw[59]]),
            false => u16::from_be_bytes([raw[58], raw[59]])
        };

        elfheader.e_shnum = match is_little {
            true => u16::from_le_bytes([raw[60], raw[61]]),
            false => u16::from_be_bytes([raw[60], raw[61]])
        };

        elfheader.e_shstrndx = match is_little {
            true => u16::from_le_bytes([raw[62], raw[63]]),
            false => u16::from_be_bytes([raw[62], raw[63]])
        };   
        Ok(elfheader)
    }

    // Print header of an ELF file
    // this should print the exact same output as the command readelf -h FILE_NAME.elf on Unix!
    pub fn print_header(&self) {
        kprintln!("ELF Header");
        kprint!("   Magic:     ");
        for mag in self.ei_mag.iter() {
            kprint!("{:2x} ", mag);
        }
        kprint!("{:x} ", self.ei_class);
        kprint!("{:x} ", self.ei_data);
        kprint!("{:x} ", self.ei_version);
        kprint!("{:x} ", self.ei_osabi);
        kprint!("{:x} ", self.ei_abiversion);
        for pad in self.ei_pad.iter() {
            kprint!("{:x} ", pad);
        }
        kprintln!("");


        kprint!("   Class:                                  ");
        match self.ei_class {
            1 => {
                kprint!("ELF32");
            },
            2 => {
                kprint!("ELF64");
            }
            _ => {
                kprint!("Can't detect class");
            }
        };
        kprintln!("");

        kprint!("   Data:                                   2's complement, ");
        match self.ei_data {
            1 => {
                kprint!("little endian");
            },
            2 => {
                kprint!("big endian");
            },
            _ => {
                kprint!("Can't detect endianess");
            }
        }
        kprintln!("");

        kprint!("   Version:                                ");
        match self.ei_version {
            1 => {
                kprint!("1 (current)");
            },
            _ => {
                kprint!("Can't detect version");
            }
        }
        kprintln!("");

        kprint!("   OS/ABI:                                 UNIX - ");
        match self.ei_osabi {
            0x00 => {kprint!("System V"); },
            0x01 => {kprint!("HP-UX"); },
            0x02 => {kprint!("NetBSD"); },
            0x03 => {kprint!("Linux"); },
            0x04 => {kprint!("GNU Hurd"); },
            0x06 => {kprint!("Solaris"); },
            0x07 =>	{kprint!("AIX"); },
            0x08 =>	{kprint!("IRIX"); },
            0x09 =>	{kprint!("FreeBSD"); },
            0x0A => {kprint!("Tru64"); },
            0x0B => {kprint!("Novell Modesto");},
            0x0C => {kprint!("OpenBSD");},
            0x0D =>	{kprint!("OpenVMS");},
            0x0E =>	{kprint!("NonStop Kernel");},
            0x0F =>	{kprint!("AROS");},
            0x10 => {kprint!("Fenix OS");},
            0x11 =>	{kprint!("CloudABI");},
            0x12 =>	{kprint!("Stratus Technologies OpenVOS");},
            _    => {kprint!("Can't detect OS/ABI");},
        }
        kprintln!("");

        kprintln!("   ABI Version:                            {}", self.ei_abiversion);

        kprint!("   Type:                                   ");
        match self.e_type {
            0x00 =>	{kprint!("NONE");},
            0x01 =>	{kprint!("REL");},
            0x02 =>	{kprint!("EXEC");},
            0x03 =>	{kprint!("DYN");},
            0x04 =>	{kprint!("CORE");},
            0xfe00 => {kprint!("LOOS");},
            0xfeff => {kprint!("HIOS");},
            0xff00 => {kprint!("LOPROC");},
            0xffff => {kprint!("HIPROC");},
            _    => {kprint!("Can't detect type");}
        }
        kprintln!("");

        kprint!("   Machine:                                ");
        match self.e_machine {
            0x00 =>	{kprint!("No specific instruction set");},
            0x02 =>	{kprint!("SPARC");},
            0x03 =>	{kprint!("x86");},
            0x08 =>	{kprint!("MIPS");},
            0x14 =>	{kprint!("PowerPC");},
            0x16 =>	{kprint!("S390");},
            0x28 =>	{kprint!("ARM");},
            0x2A =>	{kprint!("SuperH");},
            0x32 => {kprint!("IA-64");},
            0x3E =>	{kprint!("amd64");},
            0xB7 =>	{kprint!("AArch64");},
            0xF3 =>	{kprint!("RISC-V");},
            _    => {kprint!("Can't detect machine");}
        }
        kprintln!("");

        kprintln!("   Version:                                0x{:x}", self.e_version);

        kprintln!("   Entry point address:                    0x{:x}", self.e_entry);

        kprintln!("   Start of program headers:               {} (bytes into file)", self.e_phoff);
        
        kprintln!("   Start of section headers:               {} (bytes into file)", self.e_shoff);

        kprintln!("   Flags:                                  0x{:x}", self.e_flags);

        kprintln!("   Size of this header:                    {} (bytes)", self.e_ehsize);

        kprintln!("   Size of program headers:                {} (bytes)", self.e_phentsize);

        kprintln!("   Number of program headers:              {}", self.e_phnum);

        kprintln!("   Size of section headers:                {} (bytes)", self.e_shentsize);

        kprintln!("   Number of section headers:              {}", self.e_shnum);

        kprintln!("   Section header string table index:      {}", self.e_shstrndx);
    }
}

