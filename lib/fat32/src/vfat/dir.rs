use alloc::string::String;
use alloc::vec::Vec;

use shim::const_assert_size;
use shim::ffi::OsStr;
use shim::io;
use shim::newioerr;
use crate::alloc::string::ToString;
use crate::traits;
use crate::util::VecExt;
use crate::vfat::{Attributes, Date, Metadata, Time, Timestamp};
use crate::vfat::{Cluster, Entry, File, VFatHandle};

#[derive(Debug)]
pub struct Dir<HANDLE: VFatHandle> {
    pub vfat: HANDLE,
    pub dir_name : alloc::string::String,
    pub first_cluster : Cluster,
    pub metadata : Metadata,
    pub size : u32
}

impl<HANDLE: VFatHandle> Dir<HANDLE> {
    pub fn new(&self) -> Self {
        Self {
            vfat : self.vfat.clone(),
            dir_name : alloc::string::String::new(),
            first_cluster : self.vfat.lock(|vfat| vfat.rootdir_cluster),
            metadata : Metadata::default(),
            size : 0u32,
        }
    }
    pub fn name(&self) -> &String{
        &self.dir_name
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn root(&self) -> Dir<HANDLE> {
        Dir {
            vfat : self.vfat.clone(),
            dir_name : String::from("/"),
            first_cluster : self.vfat.lock(|vfat| vfat.rootdir_cluster),
            metadata : Metadata::default(),
            size : 0u32
        }
    }

}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatRegularDirEntry {
    pub dir_name : [u8; 8],
    pub file_extension : [u8; 3],
    pub attribute : Attributes,
    pub win_nt : u8, 
    pub creation_time : u8,
    pub created_time : Time,
    pub created_date : Date,
    pub accessed_date : Date,
    pub cluster_num_high : u16, 
    pub modified_time : Time,
    pub modified_date : Date,
    pub cluster_nunm_low : u16, 
    pub dir_size : u32
}

const_assert_size!(VFatRegularDirEntry, 32);

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatLfnDirEntry {
    pub sequence_num : u8,
    pub file_name1 : [u16; 5],
    pub attribute : Attributes,
    pub file_type : u8,
    pub checksum : u8,
    pub file_name2 : [u16; 6],
    pub zeroes : u16,
    pub file_name3 : [u16; 2]
}

const_assert_size!(VFatLfnDirEntry, 32);

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatUnknownDirEntry {
    pub sequence_num : u8,
    reserved1: [u8; 10],
    attribute: Attributes,
    reserved2: [u8; 20],
}

const_assert_size!(VFatUnknownDirEntry, 32);

pub union VFatDirEntry {
    unknown: VFatUnknownDirEntry,
    regular: VFatRegularDirEntry,
    long_filename: VFatLfnDirEntry,
}

impl<HANDLE: VFatHandle> Dir<HANDLE> {
    /// Finds the entry named `name` in `self` and returns it. Comparison is
    /// case-insensitive.
    ///
    /// # Errors
    ///
    /// If no entry with name `name` exists in `self`, an error of `NotFound` is
    /// returned.
    ///
    /// If `name` contains invalid UTF-8 characters, an error of `InvalidInput`
    /// is returned.
    pub fn find<P: AsRef<OsStr>>(&self, name: P) -> io::Result<Entry<HANDLE>> {
        use traits::Dir;
        use traits::Entry;
        let entry_name = name.as_ref().to_str().ok_or(io::Error::new(io::ErrorKind::InvalidInput, "name contains invalid utf-8"))?;
        
        self.entries()?.find(|entry| entry.name().eq_ignore_ascii_case(entry_name)).ok_or(io::Error::new(io::ErrorKind::NotFound, "Name not found"))
    }
    pub fn size(&self) -> u32 {
        self.size
    }
} 

pub struct EntryIterator<HANDLE: VFatHandle> {
    vfat : HANDLE,
    entries : Vec::<VFatDirEntry>,
    offset : usize,
}

impl<HANDLE: VFatHandle> Iterator for EntryIterator<HANDLE> {
    type Item = Entry<HANDLE>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut has_lfn = false;
        let mut lfn_name = [0u16; 13 * 31];
        for index in self.offset .. self.entries.len() {
            let entry = &self.entries[index];
            let unknown_entry : VFatUnknownDirEntry = unsafe {entry.unknown};
            
            if unknown_entry.sequence_num == 0x00 { // last entry
                return None; // there is nothing after
            } else if unknown_entry.sequence_num == 0xE5 {
                continue
            }
            if unknown_entry.attribute.lfn() { // Long file name
                let lfn_entry : VFatLfnDirEntry = unsafe {entry.long_filename};
                has_lfn = true;
                let sequence = (lfn_entry.sequence_num & 0x1F) as usize - 1;
                lfn_name[sequence * 13 .. sequence * 13 + 5].copy_from_slice(unsafe {&lfn_entry.file_name1});
                lfn_name[sequence * 13 + 5 .. sequence * 13 + 11].copy_from_slice(unsafe {&lfn_entry.file_name2});
                lfn_name[sequence * 13 + 11 .. sequence * 13 + 13].copy_from_slice(unsafe {&lfn_entry.file_name3});
            } else { // regular dir

                let entry : VFatRegularDirEntry = unsafe {entry.regular};
                
                let name : String = if !has_lfn { // a dir
                    // get name and extension
                    let name = entry.dir_name.clone();
                    let mut name = String::from_utf8(name.to_vec()).unwrap().trim_right().to_string();
                    
                    let extension = entry.file_extension.clone();
                    let extension = String::from_utf8(extension.to_vec()).unwrap();
                    let extension = extension.trim_right();
                    if extension.len() > 0 {
                        // add "."
                        name.push_str(".");
                        name.push_str(extension);
                    };
                    name
                } else {
                    // last of lfn
                    let len = match lfn_name.iter().position(|&x| x == 0x00 || x == 0xFF) {
                        Some(len) => len,
                        None => lfn_name.len(),
                    };

                    String::from_utf16(&lfn_name[..len]).ok()?
                };
                let first_cluster = Cluster::from(((entry.cluster_num_high as u32) << 16) | (entry.cluster_nunm_low as u32));
                let metadata = Metadata {
                    timestamp_created : Timestamp {
                        date : entry.created_date,
                        time : entry.created_time,
                    },
                    timestamp_acessed : Timestamp {
                        date : entry.accessed_date,
                        time : Time::default(),
                    },
                    timestamp_modified : Timestamp {
                        date : entry.modified_date,
                        time : entry.modified_time,
                    },
                    attri : entry.attribute,
                };
                self.offset = index + 1;
                return Some(if entry.attribute.directory() {
                    Entry::DirEntry(Dir {
                        vfat : self.vfat.clone(),
                        dir_name : name,
                        first_cluster : first_cluster,
                        metadata : metadata,
                        size : entry.dir_size
                    })
                } else {
                    Entry::FileEntry(File {
                        vfat : self.vfat.clone(),
                        file_name : name,
                        first_cluster : first_cluster,
                        metadata : metadata,
                        file_size : entry.dir_size,
                        file_ptr : 0u32,
                    })
                });
            }
        }
        self.offset = self.entries.len();
        return None;
    } 
}

impl<HANDLE: VFatHandle> traits::Dir for Dir<HANDLE> {
    type Entry = Entry<HANDLE>;
    type Iter = EntryIterator<HANDLE>;
    /// Returns an interator over the entries in this directory.
    fn entries(&self) -> io::Result<Self::Iter> {
        let mut buf = Vec::new();
        
        self.vfat.lock(|vfat| vfat.read_chain(self.first_cluster, &mut buf))?;
        
        let iterator = EntryIterator {
            vfat : self.vfat.clone(),
            entries : unsafe {buf.cast()},
            offset : 0 as usize
        };
        Ok(iterator)
    }
} 