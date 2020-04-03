use alloc::string::String;
use alloc::vec::Vec;

use shim::io::{self, SeekFrom};

use crate::traits;
use crate::vfat::{Cluster, Metadata, VFatHandle};

#[derive(Debug)]
pub struct File<HANDLE: VFatHandle> {
    pub vfat: HANDLE,
    pub file_name : alloc::string::String,
    pub first_cluster: Cluster,
    pub metadata: Metadata,
    pub file_size: u32,
    pub file_ptr: u32, // point to where to read in the file
}

// FIXME: Implement `traits::File` (and its supertraits) for `File`.

impl<HANDLE: VFatHandle> traits::File for File<HANDLE> {
    /// Writes any buffered data to disk.
    fn sync(&mut self) -> io::Result<()> {
        Ok(())
    }

    /// Returns the size of the file in bytes.
    fn size(&self) -> u64 {
        self.file_size as u64
    }
}

impl<HANDLE: VFatHandle> io::Write for File<HANDLE> {
    fn write(&mut self, buf : &[u8]) -> Result<usize, io::Error> {
        unimplemented!("io::Write - Write for file");
    }
    fn flush(&mut self) -> Result<(), io::Error> {
        unimplemented!("io::Write - Flush for file");
    }
}

impl<HANDLE: VFatHandle> io::Read for File<HANDLE> {
    fn read(&mut self, buf : &mut [u8]) -> Result<usize, io::Error> {
        if self.file_size == 0 {
            return Ok(0);
        }
        let mut file_vector = Vec::new();
        let read_byte = self.vfat.lock(|vfat| vfat.read_chain(self.first_cluster, &mut file_vector))?;

        let read_size = self.file_size - self.file_ptr;
        let can_read = core::cmp::min(read_size, buf.len() as u32);

        buf[..can_read as usize].copy_from_slice(&file_vector[self.file_ptr as usize .. (self.file_ptr + can_read) as usize]); 
        self.file_ptr += can_read;
        Ok(can_read as usize)
    }
}
impl<HANDLE: VFatHandle> io::Seek for File<HANDLE> {
    /// Seek to offset `pos` in the file.
    ///
    /// A seek to the end of the file is allowed. A seek _beyond_ the end of the
    /// file returns an `InvalidInput` error.
    ///
    /// If the seek operation completes successfully, this method returns the
    /// new position from the start of the stream. That position can be used
    /// later with SeekFrom::Start.
    ///
    /// # Errors
    ///
    /// Seeking before the start of a file or beyond the end of the file results
    /// in an `InvalidInput` error.
    fn seek(&mut self, _pos: SeekFrom) -> io::Result<u64> {
        match _pos {
            SeekFrom::Start(file_offset) => {
                if file_offset > (self.file_size as u64) {
                    return Err(io::Error::new(io::ErrorKind::InvalidInput, "Seek fail. Offset > file_size"));
                } else {
                    self.file_ptr = file_offset as u32;
                    return Ok(self.file_ptr as u64);
                }
            },
            SeekFrom::Current(offset_add) => {
                let mut pointer : u32 = 0u32;
                if offset_add.is_negative() {
                    match self.file_ptr.checked_sub(-offset_add as u32) {
                        Some(new_pointer) => {
                            pointer = new_pointer;
                        },
                        None => {
                            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Seek fail. Offset > file_size"));
                        }
                    }
                } else {
                    match self.file_ptr.checked_add(offset_add as u32) {
                        Some(new_pointer) => {
                            pointer = new_pointer;
                        },
                        None => {
                            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Seek fail. Offset > file_size"));
                        }
                    }
                }
                if pointer <= self.file_size as u32{
                    self.file_ptr = pointer;
                    return Ok(self.file_ptr as u64);
                } else {
                    return Err(io::Error::new(io::ErrorKind::InvalidInput, "Seek fail. Offset > file_size"));
                }
            },
            SeekFrom::End(offset_add) => {
                let mut pointer : u32 = 0u32;
                if offset_add.is_negative() {
                    match self.file_ptr.checked_sub(-offset_add as u32) {
                        Some(new_pointer) => {
                            pointer = new_pointer;
                        },
                        None => {
                            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Seek fail. Offset > file_size"));
                        }
                    }
                } else {
                    match self.file_ptr.checked_add(offset_add as u32) {
                        Some(new_pointer) => {
                            pointer = new_pointer;
                        },
                        None => {
                            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Seek fail. Offset > file_size"));
                        }
                    }
                }
                if pointer <= self.file_size as u32{
                    self.file_ptr = pointer;
                    return Ok(self.file_ptr as u64);
                } else {
                    return Err(io::Error::new(io::ErrorKind::InvalidInput, "Seek fail. Offset > file_size"));
                }
            }
        }
    }
}
