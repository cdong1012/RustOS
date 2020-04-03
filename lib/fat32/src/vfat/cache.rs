use alloc::boxed::Box;
use alloc::vec::Vec;
use core::fmt;
use hashbrown::HashMap;
use shim::io;

use crate::traits::BlockDevice;

#[derive(Debug)]
struct CacheEntry {
    data: Vec<u8>,
    dirty: bool,
}

pub struct Partition {
    /// The physical sector where the partition begins.
    pub start: u64,
    /// Number of sectors
    pub num_sectors: u64,
    /// The size, in bytes, of a logical sector in the partition.
    pub sector_size: u64,
}

pub struct CachedPartition {
    device: Box<dyn BlockDevice>,
    cache: HashMap<u64, CacheEntry>,
    partition: Partition,
}

impl CachedPartition {
    /// Creates a new `CachedPartition` that transparently caches sectors from
    /// `device` and maps physical sectors to logical sectors inside of
    /// `partition`. All reads and writes from `CacheDevice` are performed on
    /// in-memory caches.
    ///
    /// The `partition` parameter determines the size of a logical sector and
    /// where logical sectors begin. An access to a sector `0` will be
    /// translated to physical sector `partition.start`. Virtual sectors of
    /// sector number `[0, num_sectors)` are accessible.
    ///
    /// `partition.sector_size` must be an integer multiple of
    /// `device.sector_size()`.
    ///
    /// # Panics
    ///
    /// Panics if the partition's sector size is < the device's sector size.
    pub fn new<T>(device: T, partition: Partition) -> CachedPartition
    where
        T: BlockDevice + 'static,
    {
        assert!(partition.sector_size >= device.sector_size());

        CachedPartition {
            device: Box::new(device),
            cache: HashMap::new(),
            partition: partition,
        }
    }

    /// Returns the number of physical sectors that corresponds to
    /// one logical sector.
    fn factor(&self) -> u64 {
        self.partition.sector_size / self.device.sector_size()
    }

    /// Maps a user's request for a sector `virt` to the physical sector.
    /// Returns `None` if the virtual sector number is out of range.
    fn virtual_to_physical(&self, virt: u64) -> Option<u64> {
        if virt < self.partition.start {
            return None;
        }
        let logical_offset = virt - self.partition.start;

        let physical_offset = logical_offset * self.factor();
        let physical_sector = self.partition.start + physical_offset;

        Some(physical_sector)
    }

    fn read_entry(&mut self, sector: u64) -> io::Result<CacheEntry> {
        // takes in a logical sector, translate that to the right physical sector
        // read data from the virtual sector into the cache entry
        // cache entry's data is vec<u8>, so the size of the vec
        // will be equal to the size of one logical sector in bytes

        let physical_sector = match self.virtual_to_physical(sector) {
            Some(phys_sec) => {
                phys_sec
            },
            None  => {
                panic!("Read Entry: Can't map logical to physical")
            }
        };
        let phys_sec_num = self.factor(); // number of physical sectors
        let mut data = Vec::with_capacity((phys_sec_num * self.device.sector_size()) as usize);
        
        for index in 0..phys_sec_num {
            self.device.read_all_sector(physical_sector + index, &mut data)?;
        }
        Ok(CacheEntry {
            data : data,
            dirty : false
        })
    }  

    /// Returns a mutable reference to the cached sector `sector`. If the sector
    /// is not already cached, the sector is first read from the disk.
    ///
    /// The sector is marked dirty as a result of calling this method as it is
    /// presumed that the sector will be written to. If this is not intended,
    /// use `get()` instead.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an error reading the sector from the disk.
    pub fn get_mut(&mut self, sector: u64) -> io::Result<&mut [u8]> {
        if !self.cache.contains_key(&sector) { // if sector is not cached, sector is first read from disk
            let mut cache_entry = self.read_entry(sector)?;
            self.cache.insert(sector, cache_entry); // insert this cache at the sector key
        }
        Ok(&mut self.cache.get_mut(&sector).unwrap().data)
    }

    /// Returns a reference to the cached sector `sector`. If the sector is not
    /// already cached, the sector is first read from the disk.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an error reading the sector from the disk.
    pub fn get(&mut self, sector: u64) -> io::Result<&[u8]> {
        
        if !self.cache.contains_key(&sector) { // if sector is not cached, sector is first read from disk
            
            let cache_entry = self.read_entry(sector)?;
            self.cache.insert(sector, cache_entry); // insert this cache at the sector key
        }
        
        Ok(&self.cache.get(&sector).unwrap().data.as_slice())
    }
}

// FIXME: Implement `BlockDevice` for `CacheDevice`. The `read_sector` and
// `write_sector` methods should only read/write from/to cached sectors.
impl BlockDevice for CachedPartition {
    fn sector_size(&self) -> u64 {
        //self.device.sector_size()
        self.partition.sector_size
    }

    fn read_sector(&mut self, sector: u64, buf: &mut [u8]) -> io::Result<usize> {
        
        let sector = self.get(sector)?;
        // return number of byte written
        let byte_written = core::cmp::min(sector.len(), buf.len());
        buf[..byte_written].copy_from_slice(&sector[..byte_written]);
        
        Ok(byte_written)
    }


    /// Overwrites sector `n` with the contents of `buf`.
    ///
    /// `self.sector_size()` or `buf.len()` bytes, whichever is less, are written
    /// to the sector. The number of byte written is returned.
    ///
    /// # Errors
    ///
    /// Returns an error if seeking or writing to `self` fails. Returns an
    /// error of `UnexpectedEof` if the length of `buf` is less than
    /// `self.sector_size()`.
    fn write_sector(&mut self, sector: u64, buf: &[u8]) -> io::Result<usize> {
        if buf.len() < self.sector_size() as usize {
            return Err(io::Error::new(shim::io::ErrorKind::UnexpectedEof, "cache error: buf len < sector size"));
        }
        let sector = self.get_mut(sector)?;
        let byte_written = core::cmp::min(sector.len(), buf.len());
        sector[..byte_written].copy_from_slice(&buf[..byte_written]);
        Ok(byte_written)
    }
}

impl fmt::Debug for CachedPartition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CachedPartition")
            .field("device", &"<block device>")
            .field("cache", &self.cache)
            .finish()
    }
}
