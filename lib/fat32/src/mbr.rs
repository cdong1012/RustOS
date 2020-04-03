use core::fmt;
use shim::const_assert_size;
use shim::io;

use crate::traits::BlockDevice;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CHS {
    // 3 bytes
    // header 1 byte
    // cylinder 1 byte
    // sector 1 byte
    header : u8,
    sector : u8,
    cylinder : u8,
}

impl fmt::Debug for CHS {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CHS")
            .field("header", &self.header)
            .field("sector", &self.sector)
            .field("cylinder", &self.cylinder)
            .finish()
    }
}

const_assert_size!(CHS, 3);

#[repr(C, packed)]
pub struct PartitionEntry {
    // Partition entry 16 bytes
    // boot status 1 byte
    pub boot_status : u8,
    // first CHS 3 bytes
    pub first_chs : CHS, 
    // partition type 1 byte
    pub partition_type : u8,
    // last CHS 3 bytes 
    pub last_chs : CHS,
    // first LBA 4 bytes 
    pub first_lba : u32,
    // # of sector in partition 4 bytes
    pub sector_num : u32
}

impl fmt::Debug for PartitionEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Partition Entry")
            .field("Boot status", &self.boot_status)
            .field("Starting CHS", &self.first_chs)
            .field("Partition type", &self.partition_type)
            .field("Last CHS", &self.last_chs)
            .field("First LBA", &self.first_lba)
            .field("Total number of sector in partition", &self.sector_num)
            .finish()
    }
}
const_assert_size!(PartitionEntry, 16);

/// The master boot record (MBR).
#[repr(C, packed)]
pub struct MasterBootRecord {
    // 512 bytes

    // bootstrap executable code 436 bytes
    pub bootstrap : [u8; 436],

    // disk ID 10 bytes 
    pub disk_id : [u8; 10],

    // partition table, 4 partition entries
    pub partition_table : [PartitionEntry; 4],

    // valid signature, 2 bytes
    pub valid_signature : [u8; 2]
}

impl fmt::Debug for MasterBootRecord {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Master Boot Record")
            .field("Bootstrap code", &"<bootstrap binary>")
            .field("disk_ID", &self.disk_id)
            .field("Partition table", &self.partition_table)
            .field("valid signature", &self.valid_signature)
            .finish()
    }
}

const_assert_size!(MasterBootRecord, 512);

#[derive(Debug)]
pub enum Error {
    /// There was an I/O error while reading the MBR.
    Io(io::Error),
    /// Partiion `.0` (0-indexed) contains an invalid or unknown boot indicator.
    UnknownBootIndicator(u8),
    /// The MBR magic signature was invalid.
    BadSignature,
}

impl MasterBootRecord {
    /// Reads and returns the master boot record (MBR) from `device`.
    ///
    /// # Errors
    ///
    /// Returns `BadSignature` if the MBR contains an invalid magic signature.
    /// Returns `UnknownBootIndicator(n)` if partition `n` contains an invalid
    /// boot indicator. Returns `Io(err)` if the I/O error `err` occured while
    /// reading the MBR.
    pub fn from<T: BlockDevice>(mut device: T) -> Result<MasterBootRecord, Error> {
        let mut buffer = [0u8; core::mem::size_of::<MasterBootRecord>()]; // allocate buffer to store MBR, list of 512 bytes

        device.read_sector(0, &mut buffer).map_err(|error| {Error::Io(error)})?; // map read sector error into Io error from Error

        let mbr : MasterBootRecord = unsafe {
            core::mem::transmute(buffer)
        }; // transfer the buffer into the struct MBR

        if mbr.valid_signature != [0x55, 0xAA] {
            return Err(Error::BadSignature);
        }    // check bad signature

        // check the 4 partition entries for unknown boot indicator, only accept 0 as inactive and 80 as bootable
        for index in 0..mbr.partition_table.len() {
            let partition_entry : &PartitionEntry = &mbr.partition_table[index];
            if partition_entry.boot_status != 0u8 && partition_entry.boot_status != 0x80 as u8 {
                return Err(Error::UnknownBootIndicator(index as u8));
            }
        }

        Ok(mbr)

    } 
}
