pub mod sd;

use alloc::rc::Rc;
use core::fmt::{self, Debug};
use shim::io;
use shim::path::Path;
use fat32::vfat::{VFat, VFatHandle};

use self::sd::Sd;
use crate::mutex::Mutex;

#[derive(Clone)]
pub struct PiVFatHandle(Rc<Mutex<VFat<Self>>>);

// These impls are *unsound*. We should use `Arc` instead of `Rc` to implement
// `Sync` and `Send` trait for `PiVFatHandle`. However, `Arc` uses atomic memory
// access, which requires MMU to be initialized on ARM architecture. Since we
// have enabled only one core of the board, these unsound impls will not cause
// any immediate harm for now. We will fix this in the future.
unsafe impl Send for PiVFatHandle {}
unsafe impl Sync for PiVFatHandle {}

impl Debug for PiVFatHandle {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "PiVFatHandle")
    }
}

impl VFatHandle for PiVFatHandle {
    fn new(val: VFat<PiVFatHandle>) -> Self {
        PiVFatHandle(Rc::new(Mutex::new(val)))
    }

    fn lock<R>(&self, f: impl FnOnce(&mut VFat<PiVFatHandle>) -> R) -> R {
        f(&mut self.0.lock())
    }
}
pub struct FileSystem(Mutex<Option<PiVFatHandle>>);

impl FileSystem {
    /// Returns an uninitialized `FileSystem`.
    ///
    /// The file system must be initialized by calling `initialize()` before the
    /// first memory allocation. Failure to do will result in panics.
    pub const fn uninitialized() -> Self {
        FileSystem(Mutex::new(None))
    }

    /// Initializes the file system.
    /// The caller should assure that the method is invoked only once during the
    /// kernel initialization.
    ///
    /// # Panics
    ///
    /// Panics if the underlying disk or file sytem failed to initialize.
    pub unsafe fn initialize(&self) {
        let sd = Sd::new().expect("Error creating SD");
        
        let vfat = VFat::from(sd).expect("Error creating vfat");
        *self.0.lock() = Some(vfat);
   } 

    fn get_vfat(&self) -> io::Result<PiVFatHandle>{
        match *self.0.lock() {
            Some(ref vfat) => {
                return Ok(vfat.clone());
            },
            None => {
                return Err(io::Error::new(io::ErrorKind::NotConnected, "Can't get Vfat from SD"));
            }
        }
    }
}

// FIXME: Implement `fat32::traits::FileSystem` for `&FileSystem`
impl fat32::traits::FileSystem for &FileSystem {
    /// The type of files in this file system.
    type File = fat32::vfat::File<PiVFatHandle>;

    /// The type of directories in this file system.
    type Dir = fat32::vfat::Dir<PiVFatHandle>;

    /// The type of directory entries in this file system.
    type Entry = fat32::vfat::Entry<PiVFatHandle>;

    /// Opens the entry at `path`. `path` must be absolute.
    ///
    /// # Errors
    ///
    /// If `path` is not absolute, an error kind of `InvalidInput` is returned.
    ///
    /// If any component but the last in `path` does not refer to an existing
    /// directory, an error kind of `InvalidInput` is returned.
    ///
    /// If there is no entry at `path`, an error kind of `NotFound` is returned.
    ///
    /// All other error values are implementation defined.
    fn open<P: AsRef<Path>>(self, path: P) -> io::Result<Self::Entry> {
        self.get_vfat()?.open(path)
    }
}
