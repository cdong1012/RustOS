# RustOS - ELF Parser Write up

## Author: Chuong Dong - Group 21

### **1. Proposal**
  - During lab 4, we can only load the raw bin file into memory to execute it, but the build.sh script also gives us elf files. In a real world scenario, we should be able to execute elf files instead of just bin files in order to accommodate dynamic linking and other stuff. 
  - On linux, there is a really interesting and useful elf parser called ***readelf*** that can parse different types of elf files, so I thought it would be a fun idea to explore and create my own elf parser for our RustOS.
  - Even though the files are specified to be built without dynamic linking, I still wanted to try and parse the dynamic sections of the ELF file(similar to ldd or readelf -d on Linux).
  - After finishing the parser, I should be able to
    1. Parse fib.elf and sleep.elf, extract their binary code, load it into user virtual memory and execute it normally like with the bin files
    2. Parse any elf files with dynamic linking enable. For this, I have compiled a simple Rust program that dynamically links with some libc files on my machine in order to test my parser.

### **2. Layout**
  - Everything is located in ***/kern/elfparser***
    1. ***elf.rs***
      - This file stores the most generable structure of an elf file.
      - In my ELF struct, I have a raw elf file in a byte array, the elf header, and the header table
      - ELF struct has the binary() function to extract the actual binary code used to execute(basically the content of the bin file). We can use this function to load the binary into virtual memory
    2. ***header.rs***
      - All the things we need to parse a file and execute it(without dynamic linking)
      - There are 3 types of struct here:
        - **RawELFFile**: contains a vector of u8. Basically a wrapper for the elf file itself.
        - **ELFHeader**: A struct parsed from the first 64 bytes of the file. Store metadata and points us to where we can get the program header table and section table
        - **ProgHeader64** for 64-bit architecture or **ProgHeader32** for 32-bit architecture: The program header. An entry in the program header table. Each entry corresponds to a segment in the file.
    3. ***section.rs***
      - Section header lists the set of sections of the binary.
      - There are 2 types of struct here:
        - **SectionEntry64**: a section header entry. Each represents and contains a pointer to a section in memory.
        - **SectionTable**: the section header table. Contains a number of SectionEntry64.
    4. ***symbol.rs***
      - The symbols that are used in the ELF files.
      - There are 3 types of struct here:
        - **Symbol64**: a symbol. Contains info about a symbol like name and value
        - **SymbolTable**: the symbol table, .symtab section. Contains a number of Symbol64s. 
        - **DynamicSymbolTable**: the dynamic symbol table, .dyn.sym section. Contains a number of Symbol64s that are dynamically linked. Can be used by the dynamic linker(if we ever have one...) to identify the symbols defined and referenced in a module.
    5. ***relocation.rs***
      - Specifies the ELF file relocation. Can be used by the dynamic linker if we ever have one...
      - There are 3 types of struct here:
        - **Rela64**: Elf64_Rela in Linux. An entry in relocation tables.
        - **RelaTable**: Relocation table, .rela.dyn section. Stores a number of Rela64s. 
        - **RelaPLT**: Procedure linkage table, .rela.plt section. Used to call external procedures/functions. 
    6. ***dynamic.rs***
      - Finds the NEEDED entries determining which libraries have to be loaded before the program can be run
      - There are 2 types of struct here:
        - **Dyn64**: Elf64_Dyn in Linux. An entry in the dynamic section.
        - **DynamicTable**: dynamic table, .dynamic section. Contains Dyn64s. The Dyn64 entries with type NEEDED contains the file name of the libraries that this executable depends on.
    7. ***version.rs***
      - Specifies the version of the symbols this ELF uses.
      - There are 4 types of struct here:
        - **Verneed64**: Elf64_Verneed in Linux. An entry in the version requirement table
        - **GnuVersionReq**: the version requirement table, .gnu.version_r section. Contains required symbol version definitions
        - **Version64**: Elf64_Half in linux. Basically a small entry of 2 bytes in the symbol version table
        - **GnuVersion**: The symbol version table, .gnu.version section. This should have the same number of entries as the Dynamic Symbol Table. Each entry in this represents the version of that in the Dynamic Symbol Table.
    8. ***values.rs***
      - This file stores the constant value for all of the above structs types, flags,...
      - Since each of these has a bunch of different values varied throughout machine, architecture, etc, it took me way too long to look for documentations from Linux and other sources to collect them. 
      - These modules should be used when parsing and printing the sections instead of their actual values. If this project ends up getting expand(maybe into a dynamic linker), this file will help tremendously in localizing these ELF values into one spot.

### **3. How to run**
  - In ***/user/build.sh***, to make sure I load the elf files into the sd card, I changed the last for loop to this to load the elf files into sd card instead of the bin files
  
    ```
      for d in ${PROGS[@]}; do
        #sudo cp $d/build/$d.bin $MNT/$d
        sudo cp $d/build/$d.elf $MNT/$d
      done 
    ```
    
  1. To run the first demo, please execute this kmain
    
    ```
      fn kmain() -> ! {
        unsafe {
            ALLOCATOR.initialize();
            FILESYSTEM.initialize();
            IRQ.initialize();
            VMM.initialize();

            SCHEDULER.initialize();
            SCHEDULER.start();
        }
        loop {}
      }
    ```
    
    - This demo will attempt to execute the fib.elf file! Check out my process.rs if you want to see how I load the elf file into memory!
  2. To run the second demo, please execute this kmain
    
    ```
      fn kmain() -> ! {
        unsafe {
            ALLOCATOR.initialize();
            FILESYSTEM.initialize();
            IRQ.initialize();
            VMM.initialize();

            shell(">");
        }
        loop {}
      }
    ```
    
    - In this demo, please try using my readelf command in my shell on all the elf files we have in the SD card.

### **4. readelf**
  - I have tried to modify and copy the spacing for Linux readelf, but my code is still not perfect... It stills parses the file correctly, so I think I'll settle with it!
  - Follow this command info to execute ***readelf***
  
    ```
      Usage: readelf <option(s)> elf-file(s)
      Display information about the contents of ELF format files
      Options are:
        -h --file-header       Display the ELF file header
        -l --program-headers   Display the program headers
        -S --section-headers   Display the sections' header
        -s --symbols           Display the symbol table
        --dyn-syms             Display the dynamic symbol table
        -r --relocs            Display the relocations (if present)
        -d --dynamic           Display the dynamic section (if present)
        -V --version-info      Display the version sections (if present)
    ```
    
  - I suggest testing most of these on the **real** file! It's the executable I compiled on my Linux machine!

**5. Wrapping Up**
  - I have been trying to look into if I can write a dynamic linker to get dynamic linking fully working in our machine, but getting our OS to even generate library code and dynamically link our executables with that was painfully complicated...
  - This is all I have! Most of the time I spend on this project was from reading how Linux implements their own readelf, and while seeing how people back then could come up with such a complicated format was amazing, I never want to go through this again... The Linux source code and also the documentations on ELF in general are not well-documented at all...
  - I'm glad I have been able to take this class! I would like to thank professor Kim and all of the TAs for making this class such a wonderful experience for us!
  - Also, huge shoutout specifically to Mansour for suggesting this topic! It was really eye-opening to break down and look into the ELF file's components, and it certainly has given me a new way to look at binary exploitation for ELF from now on!

