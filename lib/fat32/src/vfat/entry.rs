use crate::traits;
use crate::vfat::{Dir, File, Metadata, VFatHandle};
use core::fmt;

// You can change this definition if you want
#[derive(Debug)]
pub enum Entry<HANDLE: VFatHandle> {
    FileEntry(File<HANDLE>),
    DirEntry(Dir<HANDLE>),
}

impl<HANDLE: VFatHandle> traits::Entry for Entry<HANDLE> {
    type File = File<HANDLE>;
    type Dir = Dir<HANDLE>;
    type Metadata = Metadata;

    fn name(&self) -> &str {
        match self {
            &Entry::FileEntry(ref file) => {
                return file.file_name.as_str();
            },
            &Entry::DirEntry(ref directory) => {
                return directory.dir_name.as_str();
            }
        }
    }

    /// The metadata associated with the entry.
    fn metadata(&self) -> &Self::Metadata {
        match self {
            &Entry::FileEntry(ref file) => {
                return &(file.metadata);
            },
            &Entry::DirEntry(ref directory) => {
                return &(directory.metadata);
            }
        }
    }

    /// If `self` is a file, returns `Some` of a reference to the file.
    /// Otherwise returns `None`.
    fn as_file(&self) -> Option<&Self::File> {
        match self {
            &Entry::FileEntry(ref file) => {
                return Some(file);
            },
            _ => {
                return None;
            }
        }
    }

    /// If `self` is a directory, returns `Some` of a reference to the
    /// directory. Otherwise returns `None`.
    fn as_dir(&self) -> Option<&Self::Dir> {
        match self {
            &Entry::DirEntry(ref directory) => {
                return Some(directory);
            },
            _ => {
                return None;
            }
        }
    }

    /// If `self` is a file, returns `Some` of the file. Otherwise returns
    /// `None`.
    fn into_file(self) -> Option<Self::File> {
        if let Entry::FileEntry(file) = self {
            Some(file)
        } else {
            None
        }
    }

        /// If `self` is a directory, returns `Some` of the directory. Otherwise
    /// returns `None`.
    fn into_dir(self) -> Option<Self::Dir> {
        if let Entry::DirEntry(dir) = self {
            Some(dir)
        } else {
            None
        }
    }

    fn size(&self) -> u32 {
        match self {
            &Entry::DirEntry(ref directory) => {
                return directory.size();
            },
            &Entry::FileEntry(ref file) => {
                return file.file_size;
            }
        }
    }
} 
