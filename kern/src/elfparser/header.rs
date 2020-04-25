use shim::const_assert_size;
use core::fmt::Error;
use alloc::vec::Vec;
use shim::path::{Path, PathBuf};
use crate::FILESYSTEM;
use fat32::traits::FileSystem;
use fat32::traits::Entry;
use crate::console::{kprintln, kprint};
use core::ops::{Deref, DerefMut};
use core::mem::size_of;
use crate::elfparser::values::*;
// RawELFFile struct, contains a vector of raw u8s
#[derive(Debug, Default, Clone)]
pub struct RawELFFile {
    pub raw: Vec<u8>
}

impl RawELFFile {
    // RawELFFile new function, initialize a new vector inside the struct and return it
    pub fn new() -> RawELFFile {
        RawELFFile {
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
            kprintln!("Can't open file at path: {:?}", dir.to_str().unwrap());
            return 0;
        }
        let entry = entry.unwrap();
        let mut buffer = [0u8; 2500000usize];                              // change this to something huge
        let mut file_length : usize = 0usize;

        if let Some(mut file) = entry.into_file() {                 
            use shim::io::Read;
            let length = match file.read(&mut buffer) {                 // read the file into the buffer
                Ok(length) => {
                    length
                },
                Err(error)=> {
                    kprintln!("Can't read file {:?}", error);
                    return 0;
                }
            };
            file_length = length;
        }
        for byte in buffer[..file_length].iter() {                  // iterate through buffer, read it in vec
            self.raw.push(byte.clone());
        }
        file_length
    }

    // This function is only for demo-ing the fact that I can get the elf file to execute in my first demo.
    // Use read_file for any other case. If I use read_file instead of this for the process, the buffer of 
    // 2500000 cause an error periodically when I run it... Not stable at all. But there is virtually no difference between the two
    pub fn read_file_to_execute<P: AsRef<Path>>(&mut self, path: P) -> usize {
        let mut working_dir = PathBuf::from("/");
        let mut dir = working_dir.clone();
        dir.push(path);
        let entry = FILESYSTEM.open(dir.as_path());

        if entry.is_err() {
            kprintln!("Can't open file at path: {:?}", dir.to_str().unwrap());
            return 0;
        }
        let entry = entry.unwrap();
        let mut buffer = [0u8; 80000usize];                              // change this to something huge
        let mut file_length : usize = 0usize;

        if let Some(mut file) = entry.into_file() {                 
            use shim::io::Read;
            let length = match file.read(&mut buffer) {                 // read the file into the buffer
                Ok(length) => {
                    length
                },
                Err(error)=> {
                    kprintln!("Can't read file {:?}", error);
                    return 0;
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

impl Deref for RawELFFile {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

impl DerefMut for RawELFFile {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.raw
    }
}

// ELF header(file header) struct, the first 64 bits of the elf file.
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
    * From the raw elf file into elf header
    */
    pub fn from(elf: &RawELFFile) -> Result<ELFHeader, usize> {
        let mut header = [0u8; 64];
        let raw = elf.as_slice();
        if raw.len() < 64 {
            return Err(0usize);
        }
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
            FileHeaderClass::ELF32 => { kprint!("ELF32");},
            FileHeaderClass::ELF64 => { kprint!("ELF64");},
            _                  => { kprint!("Can't detect class");}
        };
        kprintln!("");

        kprint!("   Data:                                   2's complement, ");
        match self.ei_data {
            FileHeaderComp::LE => { kprint!("little endian");},
            FileHeaderComp::BE => { kprint!("big endian");},
            _              => { kprint!("Can't detect endianess");}
        }
        kprintln!("");

        kprint!("   Version:                                ");
        match self.ei_version {
            1 => { kprint!("1 (current)");},
            _ => { kprint!("Can't detect version");}
        }
        kprintln!("");

        kprint!("   OS/ABI:                                 UNIX - ");
        match self.ei_osabi {
            FileHeaderOSABI::SYSV       =>  {kprint!("System V"); },
            FileHeaderOSABI::HPUX       =>  {kprint!("HP-UX"); },
            FileHeaderOSABI::NETBSD     =>  {kprint!("NetBSD"); },
            FileHeaderOSABI::LINUX      =>  {kprint!("Linux"); },
            FileHeaderOSABI::GNUHURD    =>  {kprint!("GNU Hurd"); },
            FileHeaderOSABI::SOLARIS    =>  {kprint!("Solaris"); },
            FileHeaderOSABI::AIX        =>  {kprint!("AIX"); },
            FileHeaderOSABI::IRIX       =>	{kprint!("IRIX"); },
            FileHeaderOSABI::FREEBSD    =>	{kprint!("FreeBSD"); },
            FileHeaderOSABI::TRU64      =>  {kprint!("Tru64"); },
            FileHeaderOSABI::NOVMOD     =>  {kprint!("Novell Modesto");},
            FileHeaderOSABI::OPENBSD    =>  {kprint!("OpenBSD");},
            FileHeaderOSABI::OPENVMS    =>	{kprint!("OpenVMS");},
            FileHeaderOSABI::NONSTOP    =>	{kprint!("NonStop Kernel");},
            FileHeaderOSABI::AROS       =>	{kprint!("AROS");},
            FileHeaderOSABI::FENIX      =>  {kprint!("Fenix OS");},
            FileHeaderOSABI::CLOUDABI   =>	{kprint!("CloudABI");},
            FileHeaderOSABI::OPENVOS    =>	{kprint!("Stratus Technologies OpenVOS");},
            _                       =>  {kprint!("Can't detect OS/ABI");},
        }
        kprintln!("");

        kprintln!("   ABI Version:                            {}", self.ei_abiversion);

        kprint!("   Type:                                   ");
        match self.e_type {
            FileHeaderType::NONE    =>	{kprint!("NONE");},
            FileHeaderType::REL     =>	{kprint!("REL");},
            FileHeaderType::EXEC    =>	{kprint!("EXEC");},
            FileHeaderType::DYN     =>	{kprint!("DYN");},
            FileHeaderType::CORE    =>	{kprint!("CORE");},
            FileHeaderType::LOOS    =>  {kprint!("LOOS");},
            FileHeaderType::HIOS    =>  {kprint!("HIOS");},
            FileHeaderType::LOPROC  =>  {kprint!("LOPROC");},
            FileHeaderType::HIPROC  =>  {kprint!("HIPROC");},
            _                   =>  {kprint!("Can't detect type");}
        }

        kprintln!("");

        kprint!("   Machine:                                ");
        match self.e_machine {
            FileHeaderMachine::NONE     =>	{kprint!("No specific instruction set");},
            FileHeaderMachine::SPARC    =>	{kprint!("SPARC");},
            FileHeaderMachine::x86      =>	{kprint!("x86");},
            FileHeaderMachine::MIPS     =>	{kprint!("MIPS");},
            FileHeaderMachine::PowerPC  =>	{kprint!("PowerPC");},
            FileHeaderMachine::S390     =>	{kprint!("S390");},
            FileHeaderMachine::ARM      =>	{kprint!("ARM");},
            FileHeaderMachine::SuperH   =>	{kprint!("SuperH");},
            FileHeaderMachine::IA64     =>  {kprint!("IA-64");},
            FileHeaderMachine::AMD64    =>	{kprint!("amd64");},
            FileHeaderMachine::AArch64  =>	{kprint!("AArch64");},
            FileHeaderMachine::RISCV    =>	{kprint!("RISC-V");},
            _                       =>  {kprint!("Can't detect machine");}
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


// This is program header for ELF32
// probably not gonna be used for our OS, but oh well, wouldn't hurt to be able to read different kind of ELF files :D
#[derive(Debug, Default)]
pub struct ProgHeader32 {
    pub p_type: u32,
    pub p_offset: u32,
    pub p_vaddr: u32,
    pub p_paddr: u32,
    pub p_filesz: u32,
    pub p_memsz: u32,
    pub p_flags: u32,
    pub p_align: u32,
}
const_assert_size!(ProgHeader32, 32); // Program header of ELF32 is 32 bits

impl ProgHeader32 {
    pub fn new() -> ProgHeader32 {
        ProgHeader32::default()
    }
    
    //Parser Program header from ELF file
    // Index is header table index
    // #Panic 
    // panic if index >= RawELFFile.ephnum
    pub fn from(elf: &RawELFFile, index: usize) -> Result<ProgHeader32, usize> {
        let elfheader = match ELFHeader::from(elf) {
            Ok(header) => {header},
            Err(_) => {
                return Err(0usize);
            }
        };

        let start = elfheader.e_phoff as usize + index * size_of::<ProgHeader32>();
        let raw = elf.as_slice();
        let mut buffer = [0u8; size_of::<ProgHeader32>()];
        buffer.copy_from_slice(&raw[start..(start + size_of::<ProgHeader32>())]);

        // buffer now has the program header in it
        // Parsing

        let mut program_header = ProgHeader32::new();
        let is_little = match elfheader.ei_data {
            1 => true,
            2 => false,
            _ => {
                panic!("EI_DATA not valid");
            }
        };

        program_header.p_type = match is_little {
            true => u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]),
            false => u32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]),
        };

        program_header.p_offset = match is_little {
            true => u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]),
            false => u32::from_be_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]),
        };

        program_header.p_vaddr = match is_little {
            true => u32::from_le_bytes([buffer[8], buffer[9], buffer[10], buffer[11]]),
            false => u32::from_be_bytes([buffer[8], buffer[9], buffer[10], buffer[11]]),
        };

        program_header.p_paddr = match is_little {
            true => u32::from_le_bytes([buffer[12], buffer[13], buffer[14], buffer[15]]),
            false => u32::from_be_bytes([buffer[12], buffer[13], buffer[14], buffer[15]]),
        };


        program_header.p_filesz = match is_little {
            true => u32::from_le_bytes([buffer[16], buffer[17], buffer[18], buffer[19]]),
            false => u32::from_be_bytes([buffer[16], buffer[17], buffer[18], buffer[19]]),
        };

        program_header.p_memsz = match is_little {
            true => u32::from_le_bytes([buffer[20], buffer[21], buffer[22], buffer[23]]),
            false => u32::from_be_bytes([buffer[20], buffer[21], buffer[22], buffer[23]]),
        };

        program_header.p_flags = match is_little {
            true => u32::from_le_bytes([buffer[24], buffer[25], buffer[26], buffer[27]]),
            false => u32::from_be_bytes([buffer[24], buffer[25], buffer[26], buffer[27]]),
        };

        program_header.p_align = match is_little {
            true => u32::from_le_bytes([buffer[28], buffer[29], buffer[30], buffer[31]]),
            false => u32::from_be_bytes([buffer[28], buffer[29], buffer[30], buffer[31]]),
        };

        Ok(program_header)
    }

    pub fn print_header(&self) {
        kprintln!("Program Header:");
        kprint!("   Type:                    ");
        match self.p_type {
            ProgHeaderMachine::NULL         => {kprint!("NULL");},	
            ProgHeaderMachine::LOAD         => {kprint!("LOAD");},	
            ProgHeaderMachine::DYNAMIC      => {kprint!("DYNAMIC");},	
            ProgHeaderMachine::INTERP       => {kprint!("INTERP");},	
            ProgHeaderMachine::NOTE         => {kprint!("NOTE");},	
            ProgHeaderMachine::SHLIB        => {kprint!("SHLIB");},
            ProgHeaderMachine::PHDR         => {kprint!("PHDR");},	
            ProgHeaderMachine::TLS          => {kprint!("TLS");},	
            ProgHeaderMachine::LOOS         => {kprint!("LOOS");},	
            ProgHeaderMachine::HIOS         => {kprint!("HIOS");},
            ProgHeaderMachine::LOPROC       => {kprint!("LOPROC");},
            ProgHeaderMachine::HIPROC       => {kprint!("HIPROC");},
            ProgHeaderMachine::GNU_STACK    => {kprint!("GNU_STACK");},
            ProgHeaderMachine::GNU_EH_FRAME => {kprint!("GNU_EH_FRAME");},
            ProgHeaderMachine::GNU_RELRO    => {kprint!("GNU_RELRO");},
            _                               => {kprint!("Can't detect type 0x{:x}", self.p_type);}
        }
        kprintln!("");

        kprint!("   Flags:                   ");
        match self.p_flags {
            0 => {},
            1 => {kprint!("  X");},
            2 => {kprint!(" W");},
            3 => {kprint!(" WX");},
            4 => {kprint!("R");},
            5 => {kprint!("R X");},
            6 => {kprint!("RW");},
            7 => {kprint!("RWX");},
            _ => {kprint!("Can't detect flag");}
        }
        kprintln!("");

        kprintln!("   Offset:                  0x{:x}", self.p_offset);
        kprintln!("   Virtual address:         0x{:x}", self.p_vaddr);
        kprintln!("   Physical address:        0x{:x}", self.p_paddr);
        kprintln!("   File size:               0x{:x}", self.p_filesz);
        kprintln!("   Memory size:             0x{:x}", self.p_memsz);
        kprintln!("   Align:                   0x{:x}", self.p_align);
    }
}

// This is program header for ELF64
// We'll be mainly using this.
// Note: view p_flags here https://docs.oracle.com/cd/E19683-01/816-1386/6m7qcoblk/index.html#chapter6-tbl-39
// Note: view p-types here https://static.docs.arm.com/ihi0056/b/IHI0056B_aaelf64.pdf
#[derive(Debug, Default)]
pub struct ProgHeader64 {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}
const_assert_size!(ProgHeader64, 56); // Program header of ELF64 is 56 bits

impl ProgHeader64 {
    pub fn new() -> ProgHeader64 {
        ProgHeader64::default()
    }

    // Parser Program header from ELF file. 
    // Index is the header table index.
    // #Panic
    // panic if index >= RawELFFile.e_phnum
    pub fn from(elf: &RawELFFile, index: usize) -> Result<ProgHeader64, usize> {
        let elfheader = match ELFHeader::from(elf) {
            Ok(header) => {header},
            Err(_) => {
                return Err(0usize);
            }
        };

        let start = elfheader.e_phoff as usize + index * size_of::<ProgHeader64>();
        let raw = elf.as_slice();
        let mut buffer = [0u8; size_of::<ProgHeader64>()];
        buffer.copy_from_slice(&raw[start..(start + size_of::<ProgHeader64>())]);

        // buffer now has the program header in it
        // Parsing

        let mut program_header = ProgHeader64::new();
        let is_little = match elfheader.ei_data {
            1 => true,
            2 => false,
            _ => {
                panic!("EI_DATA not valid");
            }
        };

        program_header.p_type = match is_little {
            true => u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]),
            false => u32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]),
        };

        program_header.p_flags = match is_little {
            true => u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]),
            false => u32::from_be_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]),
        };

        program_header.p_offset = match is_little {
            true => u64::from_le_bytes([buffer[8], buffer[9], buffer[10], buffer[11], buffer[12], buffer[13], buffer[14], buffer[15]]),
            false => u64::from_be_bytes([buffer[8], buffer[9], buffer[10], buffer[11], buffer[12], buffer[13], buffer[14], buffer[15]])
        };

        program_header.p_vaddr = match is_little {
            true => u64::from_le_bytes([buffer[16], buffer[17], buffer[18], buffer[19], buffer[20], buffer[21], buffer[22], buffer[23]]),
            false => u64::from_be_bytes([buffer[16], buffer[17], buffer[18], buffer[19], buffer[20], buffer[21], buffer[22], buffer[23]])
        };

        program_header.p_paddr = match is_little {
            true => u64::from_le_bytes([buffer[24], buffer[25], buffer[26], buffer[27], buffer[28], buffer[29], buffer[30], buffer[31]]),
            false => u64::from_be_bytes([buffer[24], buffer[25], buffer[26], buffer[27], buffer[28], buffer[29], buffer[30], buffer[31]])
        };

        program_header.p_filesz = match is_little {
            true => u64::from_le_bytes([buffer[32], buffer[33], buffer[34], buffer[35], buffer[36], buffer[37], buffer[38], buffer[39]]),
            false => u64::from_be_bytes([buffer[32], buffer[33], buffer[34], buffer[35], buffer[36], buffer[37], buffer[38], buffer[39]])
        };

        program_header.p_memsz = match is_little {
            true => u64::from_le_bytes([buffer[40], buffer[41], buffer[42], buffer[43], buffer[44], buffer[45], buffer[46], buffer[47]]),
            false => u64::from_be_bytes([buffer[40], buffer[41], buffer[42], buffer[43], buffer[44], buffer[45], buffer[46], buffer[47]])
        };

        program_header.p_align = match is_little {
            true => u64::from_le_bytes([buffer[47], buffer[48], buffer[49], buffer[50], buffer[52], buffer[53], buffer[54], buffer[55]]),
            false => u64::from_be_bytes([buffer[47], buffer[48], buffer[49], buffer[50], buffer[52], buffer[53], buffer[54], buffer[55]])
        };
        Ok(program_header)
    }

    // Print the program header
    // Similar to readelf -l
    pub fn print_header(&self) {
        kprintln!("Program Header:");
        kprint!("   Type:                    ");
        match self.p_type {
            ProgHeaderMachine::NULL         => {kprint!("NULL");},	
            ProgHeaderMachine::LOAD         => {kprint!("LOAD");},	
            ProgHeaderMachine::DYNAMIC      => {kprint!("DYNAMIC");},	
            ProgHeaderMachine::INTERP       => {kprint!("INTERP");},	
            ProgHeaderMachine::NOTE         => {kprint!("NOTE");},	
            ProgHeaderMachine::SHLIB        => {kprint!("SHLIB");},
            ProgHeaderMachine::PHDR         => {kprint!("PHDR");},	
            ProgHeaderMachine::TLS          => {kprint!("TLS");},	
            ProgHeaderMachine::LOOS         => {kprint!("LOOS");},	
            ProgHeaderMachine::HIOS         => {kprint!("HIOS");},
            ProgHeaderMachine::LOPROC       => {kprint!("LOPROC");},
            ProgHeaderMachine::HIPROC       => {kprint!("HIPROC");},
            ProgHeaderMachine::GNU_STACK    => {kprint!("GNU_STACK");},
            ProgHeaderMachine::GNU_EH_FRAME => {kprint!("GNU_EH_FRAME");},
            ProgHeaderMachine::GNU_RELRO    => {kprint!("GNU_RELRO");},
            _                               => {kprint!("Can't detect type");}
        }
        kprintln!("");

        kprint!("   Flags:                   ");
        match self.p_flags {
            0 => {},
            1 => {kprint!("  X");},
            2 => {kprint!(" W");},
            3 => {kprint!(" WX");},
            4 => {kprint!("R");},
            5 => {kprint!("R X");},
            6 => {kprint!("RW");},
            7 => {kprint!("RWX");},
            _ => {kprint!("Can't detect flag");}
        }
        kprintln!("");

        kprintln!("   Offset:                  0x{:x}", self.p_offset);
        kprintln!("   Virtual address:         0x{:x}", self.p_vaddr);
        kprintln!("   Physical address:        0x{:x}", self.p_paddr);
        kprintln!("   File size:               0x{:x}", self.p_filesz);
        kprintln!("   Memory size:             0x{:x}", self.p_memsz);
        kprintln!("   Align:                   0x{:x}", self.p_align);
    }
}

