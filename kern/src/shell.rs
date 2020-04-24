use shim::path::{Path, PathBuf};
use stack_vec::StackVec;
use alloc::string::String;
use fat32::traits::FileSystem;
use fat32::traits::{Dir, Entry, Timestamp, Metadata};
use crate::console::{kprint, kprintln, CONSOLE};
use crate::FILESYSTEM;
use core::fmt::Write;
use core::str::FromStr;
use crate::elfparser::{ELF, SectionTable, SymbolTable, DynamicSymbolTable, GnuVersionReq, GnuVersion, RelaTable, RelaPLT, DynamicTable};
/// Error type for `Command` parse failures.
#[derive(Debug)]
enum Error {
    Empty,
    TooManyArgs,
}

/// A structure representing a single shell command.
struct Command<'a> {
    args: StackVec<'a, &'a str>,
}

impl<'a> Command<'a> {
    /// Parse a command from a string `s` using `buf` as storage for the
    /// arguments.
    ///
    /// # Errors
    ///
    /// If `s` contains no arguments, returns `Error::Empty`. If there are more
    /// arguments than `buf` can hold, returns `Error::TooManyArgs`.
    fn parse(s: &'a str, buf: &'a mut [&'a str]) -> Result<Command<'a>, Error> {
        let mut args = StackVec::new(buf);
        for arg in s.split(' ').filter(|a| !a.is_empty()) {
            args.push(arg).map_err(|_| Error::TooManyArgs)?;
        }

        if args.is_empty() {
            return Err(Error::Empty);
        }

        Ok(Command { args })
    }

    /// Returns this command's path. This is equivalent to the first argument.
    fn path(&self) -> &str {
        self.args[0]
    }

}

/// Starts a shell using `prefix` as the prefix for each line. This function
/// returns if the `exit` command is called.

pub fn shell(prefix: &str) {
    const BELL: u8      = 0x7;          // Bell ring
    const BACKSPACE: u8 = 0x8;          // Backspace same with delete
    const DELETE: u8    = 0x7F;         // Delete
    const NEWLINE: u8   = 0x0A;         // Line Feed
    const CR: u8        = 0x0D;         // Carriage Return, move cursor back to the head of line
    let mut working_dir = PathBuf::from("/");
    'shell: loop { // loop lines to lines
        kprint!("{} ",prefix);
        
        let mut command_buf = [0u8; 512]; // store the command str in here :D
        let mut command = StackVec::new(&mut command_buf);
        'line: loop { // loop characters to characters
            let mut console = CONSOLE.lock();
            let input = console.read_byte();

            if !input.is_ascii() { // invalid ascii, unregconizable
                console.write_byte(BELL); // ring bell and go back
                continue;
            }
            if input == BACKSPACE || input == DELETE {
                // backspace or delete is hit
                // erase a single character
                match command.pop() { // pop the last character
                    Some(_) => { // if we successfully pop
                        console.write_byte(BACKSPACE); // move the console back
                        console.write_byte(b' ');
                        console.write_byte(BACKSPACE);
                    },
                    None    => {
                        console.write_byte(BELL); // nothing to move back, just ring bell
                    }
                };
            } else if input == NEWLINE || input == CR {
                // new line is hit
                let mut args : [&str; 64] = [""; 64];
                console.write_byte(NEWLINE);
                console.write_byte(CR);
                let args_str = core::str::from_utf8(command.as_slice()).unwrap();
                match Command::parse(args_str, &mut args) {
                    Ok(com) => {
                        if Command::path(&com) == "echo" { // handle echo
                            let mut iterator = com.args.into_iter();
                            iterator.next();
                            loop {
                                match iterator.next() {
                                    Some(element) => {
                                        (*console).write_str(element);
                                        console.write_byte(b' ');
                                    },
                                    None          => {
                                        console.write_byte(NEWLINE);
                                        console.write_byte(CR);
                                        break 'line;
                                    }
                                }
                            }
                        } else if Command::path(&com) == "exit" {
                            return;
                        } else if Command::path(&com) == "sleep" {
                            if com.args.len() > 2 {
                                kprintln!("Too many argumnets");
                                kprintln!("Usage: sleep <ms>");
                            }
                            let ms : u32 = u32::from_str(com.args[1]).unwrap();
                            
                            kernel_api::syscall::sleep(core::time::Duration::from_millis(ms as u64));
                        
                        } else if Command::path(&com) == "readelf" {
                            if com.args.len() != 3 {
                                kprintln!("Invalid number of argument");
                                kprintln!("Usage: readelf <option(s)> elf-file(s)");
                                kprintln!(" Display information about the contents of ELF format files");
                                kprintln!(" Options are:");
                                kprintln!("  -h --file-header       Display the ELF file header");
                                kprintln!("  -l --program-headers   Display the program headers");
                                kprintln!("  -S --section-headers   Display the sections' header");
                                kprintln!("  -s --symbols           Display the symbol table");
                                kprintln!("  --dyn-syms             Display the dynamic symbol table");
                                kprintln!("  -r --relocs            Display the relocations (if present)");
                                kprintln!("  -d --dynamic           Display the dynamic section (if present)");
                                kprintln!("  -V --version-info      Display the version sections (if present)");
                                break 'line;
                            }
                            let args = com.args;
                            let dir = working_dir.clone();
                            let entry = FILESYSTEM.open(dir.as_path());
                            if entry.is_err() {
                                kprintln!("Path not found");
                                break 'line;
                            }

                            let mut elf = ELF::new();
                            if !elf.initialize(Path::new(args[2])) {
                                kprintln!("File not found");
                                break 'line;
                            }
                            let section_table = match SectionTable::from(&elf.raw){
                                Ok(section_table) => {section_table},
                                Err(_) => {
                                    kprintln!("Can't find section table");
                                    break 'line;
                                }
                            };
                            if args[1] == "-a" || args[1] == "--all" {
                                elf.print_elf();
                            } else if args[1] == "-h" || args[1] == "--file-header" {
                                elf.header.print_header();
                            } else if args[1] == "-l" || args[1] == "--program-headers" {
                                elf.print_htable();
                            } else if args[1] == "-S" || args[1] == "--section-headers" {
                                section_table.print_section_table();
                            } else if args[1] == "-s" || args[1] == "--symbols" {
                                let symbol_table = match SymbolTable::from(&section_table) {
                                    Ok(section_table) => section_table,
                                    Err(_) => {
                                        kprintln!("Can't find symbol table");
                                        break 'line;
                                    }
                                };
                                symbol_table.print_symbol_table();
                            } else if args[1] == "-ds" || args[1] == "--dyn-syms" {
                                let dynamic_symbol_table = match DynamicSymbolTable::from(&section_table) {
                                    Ok(dyn_sym_table) => dyn_sym_table,
                                    Err(_) => {
                                        kprintln!("Cant find dynamic symbol table");
                                        break 'line;
                                    }
                                };
                                dynamic_symbol_table.print_dynamic_symbol_table();
                            } else if args[1] == "-r" || args[1] == "--relocs" {
                                let rela_table = match RelaTable::from(&section_table) {
                                    Ok(rela_table) => rela_table,
                                    Err(_) => {
                                        kprintln!("Can't find relocation table");
                                        break 'line;
                                    }
                                };
                                let plt_table = match RelaPLT::from(&section_table) {
                                    Ok(plt) => plt,
                                    Err(_) => {
                                        kprintln!("Can't find PLT table");
                                        break 'line;
                                    }
                                };
                                rela_table.print_rela_table();
                                kprintln!("");
                                plt_table.print_rela_plt();
                            } else if args[1] == "-d" || args[1] == "--dynamic" {
                                let dynamic_table = match DynamicTable::from(&section_table) {
                                    Ok(dynamic_table) => dynamic_table,
                                    Err(_) => {
                                        kprintln!("Can't find dynamic table");
                                        break 'line;
                                    }
                                };
                                dynamic_table.print_dyn_table();
                            } else if args[1] == "-V" || args[2] == "--version-info" {
                                let gnu_req = match GnuVersionReq::from(&section_table) {
                                    Ok(gnu_req) => gnu_req,
                                    Err(_) => {
                                        kprintln!("Can't find version requirement section");
                                        break 'line;
                                    }
                                };
                                let gnu_ver = match GnuVersion::from(&section_table) {
                                    Ok(gnu_ver) => gnu_ver,
                                    Err(_) => {
                                        kprintln!("Can't find version section");
                                        break 'line;
                                    }
                                };
                                
                                gnu_ver.print_gnu_version();
                                kprintln!("");
                                gnu_req.print_version_req();
                            } else {
                                kprintln!("The flag you submitted is not supported");
                                kprintln!("Usage: readelf <option(s)> elf-file(s)");
                                kprintln!(" Display information about the contents of ELF format files");
                                kprintln!(" Options are:");
                                kprintln!("  -h --file-header       Display the ELF file header");
                                kprintln!("  -l --program-headers   Display the program headers");
                                kprintln!("  -S --section-headers   Display the sections' header");
                                kprintln!("  -s --symbols           Display the symbol table");
                                kprintln!("  --dyn-syms             Display the dynamic symbol table");
                                kprintln!("  -r --relocs            Display the relocations (if present)");
                                kprintln!("  -d --dynamic           Display the dynamic section (if present)");
                                kprintln!("  -V --version-info      Display the version sections (if present)");
                            }
                            break 'line;
                        } else if Command::path(&com) == "pwd" {
                            if com.args.len() > 1 {
                                kprintln!("Too many arguments");
                                kprintln!("Usage: pwd");
                                break 'line;
                            }
                            kprint!("{}", working_dir.as_path().display());
                        } else if Command::path(&com) == "cd" {
                            if com.args.len() > 2 {
                                kprintln!("Too many arguments");
                                kprintln!("Usage: cd <directory>");
                                break 'line;
                            }
                            let args = com.args;
                            if args[1] == "." {
                                // no op
                            } else if args[1] == ".." {
                                working_dir.pop(); // truncate to parent
                            } else {
                                let new_path = Path::new(args[1]);
                                let mut new_dir = working_dir.clone();
                                new_dir.push(new_path);

                                let entry = FILESYSTEM.open(new_dir.as_path());
                                if entry.is_err() {
                                    kprintln!("Path not found");
                                    break 'line;
                                }
                                if entry.unwrap().as_dir().is_some() {
                                    working_dir.push(new_path);
                                } else {
                                    kprintln!("Not a valid directory");
                                    break 'line;
                                }
                            }
                            break 'line;
                        } else if Command::path(&com) == "ls" {
                            if com.args.len() > 3 {
                                kprintln!("Too many arguments");
                                kprintln!("Usage: cd <directory>");
                                break 'line;
                            }
                            let args = com.args;
                            let not_hidden = args.len() > 1 && args[1] == "-a";
                            let mut dir = working_dir.clone();
                            if args.len() == 3 {
                                // command: ls -a <directory>
                                if args[2] == "." {
                                    // no op
                                } else if args[2] == ".." {
                                    dir.pop();
                                } else {
                                    dir.push(args[2]);
                                }
                            } else if args.len() == 2 && args[1] != "-a" {
                                if args[1] == "." {
                                    // no op
                                } else if args[1] == ".." {
                                    dir.pop();
                                } else {
                                    dir.push(args[1]);
                                }
                            }
                            let entry = FILESYSTEM.open(dir.as_path());
                            if entry.is_err() {
                                kprintln!("Path not found");
                                break 'line;
                            }
                            let entry = entry.unwrap();
                            if let Some(directory) = entry.into_dir() {
                                // is a directory
                                let dir_entry_iterator = directory.entries().expect("current dir entries");
                                for dir_entry in dir_entry_iterator {
                                    if not_hidden || !dir_entry.metadata().hidden() {
                                        print_entry(&dir_entry);
                                    }
                                }
                            } else {
                                kprintln!("Not a directory");    
                            }
                            break 'line;
                        } else if Command::path(&com) == "cat" {
                            if com.args.len() == 1 {
                                kprintln!("Not enough arguments");
                                kprintln!("Usage: cat <path>");
                                break 'line;
                            }
                            let args = com.args;
                            for path in args {
                                if path != &"cat" {
                                    let mut dir = working_dir.clone();
                                    dir.push(path);
                                    let entry = FILESYSTEM.open(dir.as_path());
                                    if entry.is_err() {
                                        kprintln!("Path {} not found", path);
                                        break 'line;
                                    }
                                    let entry = entry.unwrap();
                                    if let Some(mut file) = entry.into_file() {
                                        loop {
                                            use shim::io::Read;
                                            let mut buffer = [0u8; 512];
                                            match file.read(&mut buffer) {
                                                Ok(0) => {
                                                    break;
                                                },
                                                Ok(length) => {
                                                    match String::from_utf8(buffer[..length].to_vec()) {
                                                        Ok(string) => {
                                                            kprint!("{}", string);
                                                        },
                                                        Err(_) => {
                                                            kprintln!("Error reading");
                                                            break 'line;
                                                        }
                                                    };
                                                },
                                                Err(error)=> {
                                                    kprint!("Can't read file {:?}", error);
                                                    break;
                                                }
                                            }
                                        }
                                        kprintln!("");
                                    } else {
                                        kprintln!("{} is not a file", path);
                                    }
                                }
                            }
                        } else {
                            kprint!("unknown command: ");
                            kprint!("{}", Command::path(&com));
                        }
                        // go to new line
                        console.write_byte(NEWLINE);
                        console.write_byte(CR);
                        break 'line;
                    }, 
                    Err(Error::TooManyArgs) => {
                        console.write_byte(BELL);
                        console.write_byte(NEWLINE);
                        console.write_byte(CR);
                        kprintln!("error: too many arguments");
                        console.write_byte(NEWLINE);
                        console.write_byte(CR);
                        break 'line;
                    }, 
                    Err(Error::Empty) => {
                        console.write_byte(BELL);
                        console.write_byte(NEWLINE);
                        console.write_byte(CR);
                        kprintln!("error: you didn't type anything..");
                        console.write_byte(NEWLINE);
                        console.write_byte(CR);
                        break 'line;
                    }
                }
            } else {
                // this is just any character
                // push on stack
                match command.push(input) {
                    Ok(_) => {
                        console.write_byte(input); // write it out
                            // loop back 'line
                    },
                    Err(_) => {
                        console.write_byte(BELL);
                    }
                }
            }
        }
    }
}
fn print_entry<E:Entry> (entry : &E) {
    write_bool(entry.is_dir(), 'd');
    write_bool(entry.is_file(), 'f');
    write_bool(entry.metadata().read_only(), 'r');
    write_bool(entry.metadata().hidden(), 'h');
    kprint!("   ");
    write_timestamp(entry.metadata().created());
    write_timestamp(entry.metadata().accessed());
    write_timestamp(entry.metadata().modified());
    print_size(entry.size());
    kprintln!("{}", entry.name());
}
fn write_bool(b : bool, c: char) {
    if b {
        kprint!("{}", c);
    } else {
        kprint!("-");
    }
}
fn print_size(size : u32) {
    use crate::alloc::string::ToString;
    let string : String = size.to_string();
    let length = string.len();
    let max_length = 10;
    let space_length = max_length - length;
    kprint!("{}", size);
    for _time in 0..space_length {
        kprint!(" ");
    }
}
fn write_timestamp<TS: Timestamp>(timestamp : TS) {
    kprint!("{:02}/{:02}/{} {:02}:{:02}:{:02}   ",
        timestamp.month(), timestamp.day(), timestamp.year(), timestamp.hour(), timestamp.minute(), timestamp.second());
}
