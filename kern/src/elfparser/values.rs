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
pub mod ProgHeaderType {
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

#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub mod RelaType {
    pub const R_X86_64_NONE: u64 = 0x0;
    pub const R_X86_64_64: u64 = 0x1;
    pub const R_X86_64_PC32: u64 = 0x2;
    pub const R_X86_64_GOT32: u64 = 0x3;
    pub const R_X86_64_PLT32: u64 = 0x4;
    pub const R_X86_64_COPY: u64 = 0x5;
    pub const R_X86_64_GLOB_DAT: u64 = 0x6;
    pub const R_X86_64_JUMP_SLOT: u64 = 0x7;
    pub const R_X86_64_RELATIVE: u64 = 0x8;
    pub const R_X86_64_GOTPCREL: u64 = 0x9;
    pub const R_X86_64_32: u64 = 0xA;
    pub const R_X86_64_32S: u64 = 0xB;
    pub const R_X86_64_16: u64 = 0xC;
    pub const R_X86_64_PC16: u64 = 0xD;
    pub const R_X86_64_8: u64 = 0xE;
    pub const R_X86_64_PC8: u64 = 0xF;
    pub const R_X86_64_DPTMOD64: u64 = 0x10;
    pub const R_X86_64_DTPOFF64: u64 = 0x11;
    pub const R_X86_64_TPOFF64: u64 = 0x12;
    pub const R_X86_64_TLSGD: u64 = 0x13;
    pub const R_X86_64_TLSLD: u64 = 0x14;
    pub const R_X86_64_DTPOFF32: u64 = 0x15;
    pub const R_X86_64_GOTTPOFF: u64 = 0x16;
    pub const R_X86_64_TPOFF32: u64 = 0x17;
}

#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub mod DynTag {
    pub const DT_NULL: u64 = 0;
    pub const DT_NEEDED: u64 = 1;
    pub const DT_PLTRELSZ: u64 = 2;
    pub const DT_PLTGOT: u64 = 3;

    pub const DT_HASH: u64 = 4;
    pub const DT_STRTAB: u64 = 5;
    pub const DT_SYMTAB: u64 = 6;
    pub const DT_RELA: u64 = 7;

    pub const DT_RELASZ: u64 = 8;
    pub const DT_RELAENT: u64 = 9;
    pub const DT_STRSZ: u64 = 10;
    pub const DT_SYMENT: u64 = 11;

    pub const DT_INIT: u64 = 12;
    pub const DT_FINI: u64 = 13;
    pub const DT_SONAME: u64 = 14;
    pub const DT_RPATH: u64 = 15;

    pub const DT_SYMBOLIC: u64 = 16;
    pub const DT_REL: u64 = 17;
    pub const DT_RELSZ: u64 = 18;
    pub const DT_RELENT: u64 = 19;

    pub const DT_PLTREL: u64 = 20;
    pub const DT_DEBUG: u64 = 21;
    pub const DT_TEXTREL: u64 = 22;
    pub const DT_JMPREL: u64 = 23;

    pub const DT_BIND_NOW: u64 = 24;

    pub const DT_INIT_ARRAY: u64 = 25;
    pub const DT_FINI_ARRAY: u64 = 26;
    pub const DT_INIT_ARRAYSZ: u64 = 27;
    pub const DT_FINI_ARRAYSZ: u64 = 28;

    pub const DT_RUNPATH: u64 = 29;
    pub const DT_FLAGS: u64 = 30;
    pub const DT_ENCODINGS: u64 = 32;
    pub const DT_PREINIT_ARRAYSZ: u64 = 33;

    pub const DT_FLAGS_1: u64 = 0x6ffffffb;
}   

#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub mod DynFlag {
    pub const DF_ORIGIN: u64 = 0x1;
    pub const DF_SYMBOLIC: u64 = 0x2;
    pub const DF_TEXTREL: u64 = 0x4;
    pub const DF_BIND_NOW: u64 = 0x8;
    pub const DF_STATIC_TLS: u64 = 0x10;
}

#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub mod DynFlag1 {
    pub const DF_1_NOW: u64 = 0x1;
    pub const DF_1_GLOBAL: u64 = 0x2;
    pub const DF_1_GROUP: u64 = 0x4;
    pub const DF_1_NODELETE: u64 = 0x8;
    pub const DF_1_LOADFLTR: u64 = 0x10;
    pub const DF_1_INITFIRST: u64 = 0x20;
    pub const DF_1_NOOPEN: u64 = 0x40;
    pub const DF_1_ORIGIN: u64 = 0x80;
    pub const DF_1_DIRECT: u64 = 0x100;
    pub const DF_1_INTERPOSE: u64 = 0x400;
    pub const DF_1_NODEFLIB: u64 = 0x800;
    pub const DF_1_NODUMP: u64 = 0x1000;
    pub const DF_1_CONFALT: u64 = 0x2000;
    pub const DF_1_ENDFILTEE: u64 = 0x4000;
    pub const DF_1_DISPRELDNE: u64 = 0x8000;
    pub const DF_1_DISPRELPND: u64 = 0x10000;
    pub const DF_1_NODIRECT: u64 = 0x20000;
    pub const DF_1_IGNMULDEF: u64 = 0x40000;
    pub const DF_1_NOKSYMS: u64 = 0x80000;
    pub const DF_1_NOHDR: u64 = 0x100000;
    pub const DF_1_EDITED: u64 = 0x200000;
    pub const DF_1_NORELOC: u64 = 0x400000;
    pub const DF_1_SYMINTPOSE: u64 = 0x800000;
    pub const DF_1_GLOBAUDIT: u64 = 0x1000000;
    pub const DF_1_SINGLETON: u64 = 0x2000000;
}


