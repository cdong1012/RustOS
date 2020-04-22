use core::fmt;

#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub mod FileHeaderClass {
    pub const ELF32: u8 = 1;
    pub const ELF64: u8 = 2;
}

#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub mod FileHeaderComp {
    pub const LE: u8 = 1;
    pub const BE: u8 = 2;
}

#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub mod FileHeaderOSABI {
    pub const SYSV: u8 = 0x0;
    pub const HPUX: u8 = 0x1;
    pub const NETBSD: u8 = 0x2;
    pub const LINUX: u8 = 0x3;
    pub const GNUHURD: u8 = 0x4;
    pub const SOLARIS: u8 = 0x6;
    pub const AIX: u8 = 0x7;
    pub const IRIX: u8 = 0x8;
    pub const FREEBSD: u8 = 0x9;
    pub const TRU64: u8 = 0xA;
    pub const NOVMOD: u8 = 0xB;
    pub const OPENBSD: u8 = 0xC;
    pub const OPENVMS: u8 = 0xD;
    pub const NONSTOP: u8 = 0xE;
    pub const AROS: u8 = 0xF;
    pub const FENIX: u8 = 0x10;
    pub const CLOUDABI: u8 = 0x11;
    pub const OPENVOS: u8 = 0x12;
}

#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub mod FileHeaderType {
    pub const NONE: u16 = 0x0;
    pub const REL: u16 = 0x1;
    pub const EXEC: u16 = 0x2;
    pub const DYN: u16 = 0x3;
    pub const CORE: u16 = 0x4;
    pub const LOOS: u16 = 0xfe00;
    pub const HIOS: u16 = 0xfeff;
    pub const LOPROC: u16 = 0xff00;
    pub const HIPROC: u16 = 0xffff;
}

#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub mod FileHeaderMachine {
    pub const NONE: u16 = 0x0;
    pub const SPARC: u16 = 0x2;
    pub const x86: u16 = 0x3;
    pub const MIPS: u16 = 0x8;
    pub const PowerPC: u16 = 0x14;
    pub const S390: u16 = 0x16;
    pub const ARM: u16 = 0x28;
    pub const SuperH: u16 = 0x2A;
    pub const IA64: u16 = 0x32;
    pub const AMD64: u16 = 0x3E;
    pub const AArch64: u16 = 0xB7;
    pub const RISCV: u16 = 0xF3;
}


#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub mod ProgHeaderMachine {
    pub const NULL: u32 = 0x0;
    pub const LOAD: u32 = 0x1;
    pub const DYNAMIC: u32 = 0x2;
    pub const INTERP: u32 = 0x3;
    pub const NOTE: u32 = 0x4;
    pub const SHLIB: u32 = 0x5;
    pub const PHDR: u32 = 0x6;
    pub const TLS: u32 = 0x7;
    pub const LOOS: u32 = 0x60000000;
    pub const HIOS: u32 = 0x6FFFFFFF;
    pub const LOPROC: u32 = 0x70000000;
    pub const HIPROC: u32 = 0x7FFFFFFF;
    pub const GNU_STACK: u32 = 0x6474e551;
    pub const GNU_EH_FRAME: u32 = 0x6474e550;
    pub const GNU_RELRO: u32 = 0x6474e552;
}

#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub mod SectionType {
    pub const NULL: u32 = 0x0;
    pub const PROGBITS: u32 = 0x1;
    pub const SYMTAB: u32 = 0x2;
    pub const STRTAB: u32 = 0x3;
    pub const RELA: u32 = 0x4;
    pub const HASH: u32 = 0x5;
    pub const DYNAMIC: u32 = 0x6;
    pub const NOTE: u32 = 0x7;
    pub const NOBITS: u32 = 0x8;
    pub const REL: u32 = 0x9;
    pub const SHLIB: u32 = 0xA;
    pub const DYNSYM: u32 = 0xB;
    pub const INIT_ARRAY: u32 = 0xE;
    pub const FINI_ARRAY: u32 = 0xF;
    pub const PREINIT_ARRAY: u32 = 0x10;
    pub const GROUP: u32 = 0x11;
    pub const SYMTAB_SHNDX: u32 = 0x12;
    pub const NUM: u32 = 0x13;
    pub const LOOS: u32 = 0x60000000;
    pub const VERDEF: u32 = 0x6ffffffd;
    pub const VERNEED: u32 = 0x6ffffffe;
    pub const VERSYM: u32 = 0x6fffffff;
}	

#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub mod SectionFlag {
    pub const WRITE: u64 = 0x1;
    pub const ALLOC: u64 = 0x2;
    pub const EXECINSTR: u64 = 0x4;
    pub const MERGE: u64 = 0x10;
    pub const STRINGS: u64 = 0x20;
    pub const INFO_LINK: u64 = 0x40;
    pub const LINK_ORDER: u64 = 0x80;
    pub const OS_NONCONFORMING: u64 = 0x100;
    pub const GROUP: u64 = 0x200;
    pub const TLS: u64 = 0x400;
    pub const MASKOS: u64 = 0x0ff00000;
    pub const MASKPROC: u64 = 0xf0000000;
    pub const ORDERED: u64 = 0x4000000;
    pub const EXCLUDE: u64 = 0x8000000;
}


#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub mod SymbolType {
    pub const NOTYPE: u8 = 0x0;
    pub const OBJECT: u8 = 0x1;
    pub const FUNC: u8 = 0x2;
    pub const SECTION: u8 = 0x3;
    pub const FILE: u8 = 0x4;
    pub const COMMON: u8 = 0x5;
    pub const TLS: u8 = 0x6;
    pub const LOOS: u8 = 0x10;
    pub const HIOS: u8 = 0x12;
    pub const LOPROC: u8 = 0x13;
    pub const HIPROC: u8 = 0x15;
}

#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub mod SymbolBind {
    pub const LOCAL: u8 = 0x0;
    pub const GLOBAL: u8 = 0x1;
    pub const WEAK: u8 = 0x2;
    pub const LOOS: u8 = 0x10;
    pub const HIOS: u8 = 0x12;
    pub const LOPROC: u8 = 0x13;
    pub const HIPROC: u8 = 0x15;
}

#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub mod SymbolVis {
    pub const DEFAULT: u8 = 0x0;
    pub const INTERNAL: u8 = 0x1;
    pub const HIDDEN: u8 = 0x2;
    pub const PROTECTED: u8 = 0x3;
    pub const EXPORTED: u8 = 0x4;
    pub const SINGLETON: u8 = 0x5;
    pub const ELIMINATE: u8 = 0x6;
}