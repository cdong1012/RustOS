# RustOS - ELF Parser

## Author: Chuong Dong

1. **Motivation**
  - During lab 4, we can only load the raw bin file into memory to execute it, but the build.sh script also gives us elf files. Mansour suggested that I should try and write an elf parser to parse elf files for our OS, and make it executable!
  - It's a good chance for me to explore ELF files and understanding its raw components since the knowledge will be helpful for RE ctf problems!
2. **Steps**
  - First, I read up about ELF, and I spent most of my time on Wikipedia trying to understand how everything falls into each other. I found this super clutch image on Wikipedia that explains it really well.
  ![alt text](https://upload.wikimedia.org/wikipedia/commons/e/e4/ELF_Executable_and_Linkable_Format_diagram_by_Ange_Albertini.png)
  
  - Basically, the first 64 bytes of ELF file is called the ELF header
     - *ELF header*:
        - Store data specifying the structure of the whole ELF
        - Has to offset of the file header table and section header table
  - After parse the ELF header, I use 
