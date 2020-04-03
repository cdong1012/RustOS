use core::fmt::Debug;
use core::marker::PhantomData;
use core::mem::size_of;

use alloc::vec::Vec;
use alloc::string::String;
use shim::io;
use shim::ioerr;
use shim::newioerr;
use shim::path;
use shim::path::{Path, Component};

use crate::mbr::MasterBootRecord;
use crate::traits::{BlockDevice, FileSystem, Entry as traitEntry};
use crate::util::SliceExt;
use crate::vfat::{BiosParameterBlock, CachedPartition, Partition};
use crate::vfat::{Cluster, Dir, Entry, Error, FatEntry, File, Status, Metadata};
/// A generic trait that handles a critical section as a closure
pub trait VFatHandle: Clone + Debug + Send + Sync {
    fn new(val: VFat<Self>) -> Self;
    fn lock<R>(&self, f: impl FnOnce(&mut VFat<Self>) -> R) -> R;
}

#[derive(Debug)]
pub struct VFat<HANDLE: VFatHandle> {
    pub phantom: PhantomData<HANDLE>,
    pub device: CachedPartition,
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub sectors_per_fat: u32,
    pub fat_start_sector: u64,
    pub data_start_sector: u64,
    pub rootdir_cluster: Cluster,
}

impl<HANDLE: VFatHandle> VFat<HANDLE> {
    pub fn from<T>(mut device: T) -> Result<HANDLE, Error>
    where
        T: BlockDevice + 'static,
    {

        let mbr = MasterBootRecord::from(&mut device)?;
        let partition_entry = match mbr.partition_table.iter().find(|entry| entry.partition_type == 0xB || entry.partition_type == 0xC) {
            Some(part_entry) => part_entry,
            None             => {
                return Err(Error::NotFound);
            } 
        };
        
        let relative_sector = partition_entry.first_lba as u64;
        let ebpb = BiosParameterBlock::from(&mut device, relative_sector as u64)?;

        let fat_start_sector = relative_sector + ebpb.reserved_sector_num as u64;
        let data_start_sector = fat_start_sector + ebpb.sector_per_fat() as u64 * ebpb.fat_num as u64; 

        let root_cluster = Cluster::from(ebpb.root_dir_cluster_num);
        
        let cached_device = CachedPartition::new(device, Partition {
            start : relative_sector,
            num_sectors : partition_entry.sector_num as u64,
            sector_size : ebpb.byte_per_sector as u64,
        });

        Ok(HANDLE::new(VFat {
            phantom: PhantomData::<HANDLE>,
            device : cached_device,
            bytes_per_sector : ebpb.byte_per_sector,
            sectors_per_cluster : ebpb.sector_per_cluster,
            sectors_per_fat : ebpb.sector_per_fat(),
            fat_start_sector : fat_start_sector,
            data_start_sector : data_start_sector,
            rootdir_cluster : root_cluster
        }))
    }

    pub fn read_cluster(&mut self, cluster: Cluster, offset: usize, buf: &mut [u8]) -> io::Result<usize> {
        let cluster_offset = match cluster.offset() {
            Some(offset) => offset as u64,
            None         => {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "can't get cluster offset"));
            }
        };
        
        let cluster_start = self.data_start_sector + cluster_offset * (self.sectors_per_cluster as u64);
        
        let start_sector = cluster_start + offset as u64;
        let end_sector = cluster_start + self.sectors_per_cluster as u64;
        let read_length = buf.len() as u64 / self.bytes_per_sector as u64; // buf len is in bytes, divide by bytes per sector to get sectors
        let true_end = core::cmp::min(end_sector, start_sector + read_length);

        let mut bytes_written : usize = 0;
        for sector_index in start_sector..true_end {
            
            bytes_written += self.device.read_sector(sector_index, &mut buf[bytes_written..])?;
        }
        Ok(bytes_written)
    }


    // A method to return a reference to a `FatEntry` for a cluster where the
    // reference points directly into a cached sector.
    pub fn fat_entry(&mut self, cluster: Cluster) -> io::Result<&FatEntry> {
        // have fat start sector.
        // need to find the sector in the fat coresspond to the cluster
        // 1. sector = fat_start_sector + sector_index
        // Each fat entries corespond to 1 cluster
        // 2. entries_index = cluster.index
        // entr

        let cluster_index = cluster.index() as usize;
        let fat_entries_per_sector = self.bytes_per_sector as usize / core::mem::size_of::<FatEntry>();
        let fat_sector_index = cluster_index / fat_entries_per_sector;
        let fat_entry_index = cluster_index % fat_entries_per_sector;

        let sector = self.device.get(self.fat_start_sector + fat_sector_index as u64)?;
        let sector : &[FatEntry] = unsafe {
            sector.cast()
        };
        Ok(&sector[fat_entry_index])
    }
    
    //  * A method to read all of the clusters chained from a starting cluster
    //    into a vector buf.
    pub fn read_chain(&mut self, start: Cluster, buf: &mut Vec<u8>) -> io::Result<usize> {
        let mut read_byte = 0;
        let mut cluster_node = start;

        loop {
            buf.resize(buf.len() + self.bytes_per_sector as usize * self.sectors_per_cluster as usize, 0);
            read_byte += self.read_cluster(cluster_node, 0, &mut buf[read_byte..])?;

            let fat_entry = self.fat_entry(cluster_node)?;
            match fat_entry.status() {
                Status::Eoc(_) => {
                    return Ok(read_byte);
                },
                Status::Data(next_cluster) => {
                    cluster_node = next_cluster;
                },
                _ => {
                    
                    return Err(io::Error::new(io::ErrorKind::Other, "Read into invalid cluster"));
                }
            }
        }

    }
}

impl<'a, HANDLE: VFatHandle> FileSystem for &'a HANDLE {
    type File = File<HANDLE>;
    type Dir = Dir<HANDLE>;
    type Entry = Entry<HANDLE>;

    fn open<P: AsRef<Path>>(self, path: P) -> io::Result<Self::Entry> {
        let p = path.as_ref();
        if !p.is_absolute() {
            return Err( io::Error::new( io::ErrorKind::InvalidInput, "path is not absolute" ))
        } 
        
        let mut vec = alloc::vec::Vec::<Self::Entry>::new();
        for component in p.components() {
            match component {
                Component::RootDir => {
                    vec.clear();
                    let rootdir = Self::Dir {
                        vfat : self.clone(),
                        dir_name : String::from("/"),
                        first_cluster : self.lock(|vfat| vfat.rootdir_cluster),
                        metadata : Metadata::default(),
                        size : 0u32,
                    };
                    vec.push(Entry::DirEntry(rootdir));
                },
                Component::Normal(name) => {
                    let element = if let Some(entry) = vec.last() {
                        if let Some(directory) = entry.as_dir() {
                            Some(directory.find(name)?)
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    match element {
                        Some(file) => {
                            vec.push(file);
                        },
                        None => {
                            return Err( io::Error::new(io::ErrorKind::NotFound, "404 Not found :(" ));
                        }
                    }
                },
                Component::ParentDir => { vec.pop(); },
                Component::CurDir => {},
                _ => { unimplemented!(); },
            }
        }
        match vec.into_iter().last() {
            Some(file) => {
                return Ok(file);
            },
            None => {
                return Err( io::Error::new(io::ErrorKind::NotFound, "404 Not found :(" ));
            }
        }
    }
}
