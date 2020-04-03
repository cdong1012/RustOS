use alloc::boxed::Box;
use shim::io;
use shim::path::{Path, PathBuf};
use shim::{const_assert_eq, const_assert_size};

use aarch64;
use crate::FILESYSTEM;
use fat32::traits::FileSystem;
use fat32::traits::Entry;
use crate::param::*;
use crate::process::{Stack, State};
use crate::traps::TrapFrame;
use crate::vm::*;
use kernel_api::{OsError, OsResult};
use core::mem::replace;
use crate::console::{kprintln};
use crate::ALLOCATOR;
use crate::allocator;
use alloc::string::String;
use core::ptr::Unique;
/// Type alias for the type of a process ID.
pub type Id = u64;

/// A structure that represents the complete state of a process.
#[derive(Debug)]
pub struct Process {
    /// The saved trap frame of a process.
    pub context: Box<TrapFrame>,
    /// The memory allocation used for the process's stack.
    //pub stack: Stack,
    /// The page table describing the Virtual Memory of the process
    pub vmap: Box<UserPageTable>,
    /// The scheduling state of the process.
    pub state: State,
}

impl Process {
    /// Creates a new process with a zeroed `TrapFrame` (the default), a zeroed
    /// stack of the default size, and a state of `Ready`.
    ///
    /// If enough memory could not be allocated to start the process, returns
    /// `None`. Otherwise returns `Some` of the new `Process`.
    pub fn new() -> OsResult<Process> {
        let tf = Box::new(TrapFrame::default()); // default trap frame
        // let stack = match Stack::new() {
        //     Some(stack) => stack,
        //     None => {
        //         return Err(OsError::from(20)); // no memory errror
        //     }
        // };
        let vmap = Box::new(UserPageTable::new());
        return Ok(Process {
            context : tf,
            //stack : stack,
            vmap : vmap,
            state : State::Ready
        });
    }

    /// Load a program stored in the given path by calling `do_load()` method.
    /// Set trapframe `context` corresponding to the its page table.
    /// `sp` - the address of stack top
    /// `elr` - the address of image base.
    /// `ttbr0` - the base address of kernel page table
    /// `ttbr1` - the base address of user page table
    /// `spsr` - `F`, `A`, `D` bit should be set.
    ///
    /// Returns Os Error if do_load fails.
    pub fn load<P: AsRef<Path>>(pn: P) -> OsResult<Process> {
        use crate::VMM;

        let mut p = Process::do_load(pn)?;

        p.context.sp = Process::get_stack_top().as_u64();
        p.context.elr = USER_IMG_BASE as u64;
        p.context.ttbr0 = VMM.get_baddr().as_u64();
        p.context.ttbr1 = p.vmap.get_baddr().as_u64();
        p.context.spsr = 0b1101000000;
        Ok(p)
    }

    /// Creates a process and open a file with given path.
    /// Allocates one page for stack with read/write permission, and N pages with read/write/execute
    /// permission to load file's contents.
    fn do_load<P: AsRef<Path>>(pn: P) -> OsResult<Process> {
        let mut process = Process::new().unwrap();
        let user_stack = process.vmap.alloc(VirtualAddr::from(USER_STACK_BASE), PagePerm::RW);
        
        let mut working_dir = PathBuf::from("/");
        let mut page = process.vmap.alloc(VirtualAddr::from(USER_IMG_BASE as u64), PagePerm::RWX);
        
        let mut dir = working_dir.clone();
        dir.push(pn);
        let entry = FILESYSTEM.open(dir.as_path());
        if entry.is_err() {
            kprintln!("Path not found");
            panic!("Path not found");
        }

        let entry = entry.unwrap();
        if let Some(mut file) = entry.into_file() {
            let mut start_index = 0;
            use shim::io::Read;
            let mut buffer = [0u8; PAGE_SIZE]; // buffer of PAGE_SIZE bytes
            let length = match file.read(&mut buffer) {
                Ok(length) => {
                    length
                },
                Err(error)=> {
                    kprintln!("Can't read file {:?}", error);
                    0usize
                }
            };
            page.copy_from_slice(&buffer);
        }
        Ok(process)
    }

    /// Returns the highest `VirtualAddr` that is supported by this system.
    pub fn get_max_va() -> VirtualAddr {
        // 65536 * 16382
        VirtualAddr::from(0xffff_ffff_ffff_ffff as u64)
    }

    /// Returns the `VirtualAddr` represents the base address of the user
    /// memory space.
    pub fn get_image_base() -> VirtualAddr {
        VirtualAddr::from(USER_IMG_BASE)
    }

    /// Returns the `VirtualAddr` represents the base address of the user
    /// process's stack.
    pub fn get_stack_base() -> VirtualAddr {
        VirtualAddr::from(USER_STACK_BASE) // last space in virtual addr
    }

    /// Returns the `VirtualAddr` represents the top of the user process's
    /// stack.
    pub fn get_stack_top() -> VirtualAddr {
        /// The default stack size is 1MiB = 1 << 20.
        use core::ops::Add;
        VirtualAddr::from(USER_STACK_BASE + (PAGE_SIZE - 16)) // align with this
        
    }

    /// Returns `true` if this process is ready to be scheduled.
    ///
    /// This functions returns `true` only if one of the following holds:
    ///
    ///   * The state is currently `Ready`.
    ///
    ///   * An event being waited for has arrived.
    ///
    ///     If the process is currently waiting, the corresponding event
    ///     function is polled to determine if the event being waiting for has
    ///     occured. If it has, the state is switched to `Ready` and this
    ///     function returns `true`.
    ///
    /// Returns `false` in all other cases.
    pub fn is_ready(&mut self) -> bool {
        let state = State::Ready;
        let original_state : State = core::mem::replace(&mut self.state, state);
        match original_state {
            State::Ready => true,
            State::Waiting(mut function) => {
                if function(self) {
                    self.state = State::Ready;
                    kprintln!("Process {} ready", self.context.tpidr);
                    return true;
                } else {
                    self.state = State::Waiting(function);
                    kprintln!("Process {} not ready", self.context.tpidr);
                    return false;
                }
            },
            _ => {
                self.state = original_state;
                return false;
            }
        }
    }
}
