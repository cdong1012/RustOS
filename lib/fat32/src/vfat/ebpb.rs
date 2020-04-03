use core::fmt;
use shim::const_assert_size;

use crate::traits::BlockDevice;
use crate::vfat::Error;

#[repr(C, packed)]
pub struct BiosParameterBlock {
    pub jump_short_nop          : [u8; 3],
    pub oem_identifier          : [u8; 8],
    pub byte_per_sector         : u16,
    pub sector_per_cluster      : u8,
    pub reserved_sector_num     : u16,
    pub fat_num                 : u8,
    pub max_dir_entries_num     : u16, 
    pub logical_sector          : u16,
    pub descriptor_type         : u8,
    pub sector_per_fat          : u16,
    pub sector_per_track        : u16,
    pub heads_num               : u16,
    pub hidden_sector_num       : u32, 
    pub total_logical_sector    : u32,
    pub sector_per_fat_32       : u32,
    pub flags                   : u16,
    pub fat_version_num         : [u8; 2],
    pub root_dir_cluster_num    : u32,
    pub fsinfo_sector_num       : u16,
    pub backup_boot_sector_num  : u16,
    pub reserved                : [u8; 12],
    pub drive_num               : u8,
    pub win_nt_flags            : u8,
    pub signature               : u8,
    pub volumn_id               : u32,
    pub volumn_label            : [u8; 11],
    pub sys_id_str              : [u8; 8],
    pub boot_code               : [u8; 420],
    pub bootable_signature      : u16,
}

const_assert_size!(BiosParameterBlock, 512);

impl BiosParameterBlock {
    pub fn sector_per_fat(&self) -> u32 {
        if self.sector_per_fat != 0 {
            self.sector_per_fat as u32
        } else {
            self.sector_per_fat_32
        }
    }

    pub fn total_logical_sectors(&self) -> u32 {
        if self.logical_sector != 0 {
            self.logical_sector as u32
        } else {
            self.total_logical_sector
        }
    
    }
    pub fn change_byte_order(mut ebpb : BiosParameterBlock) -> BiosParameterBlock {
        ebpb.byte_per_sector = u16::from_le(ebpb.byte_per_sector);
        ebpb.reserved_sector_num = u16::from_le(ebpb.reserved_sector_num);
        ebpb.max_dir_entries_num = u16::from_le(ebpb.max_dir_entries_num);
        ebpb.logical_sector = u16::from_le(ebpb.logical_sector);
        ebpb.sector_per_fat = u16::from_le(ebpb.sector_per_fat);
        ebpb.sector_per_track = u16::from_le(ebpb.sector_per_track);
        ebpb.heads_num = u16::from_le(ebpb.heads_num);
        ebpb.hidden_sector_num = u32::from_le(ebpb.hidden_sector_num);
        ebpb.total_logical_sector = u32::from_le(ebpb.total_logical_sector);
        ebpb.sector_per_fat_32 = u32::from_le(ebpb.sector_per_fat_32);
        ebpb.flags = u16::from_le(ebpb.flags);
        ebpb.root_dir_cluster_num = u32::from_le(ebpb.root_dir_cluster_num);
        ebpb.fsinfo_sector_num = u16::from_le(ebpb.fsinfo_sector_num);
        ebpb.backup_boot_sector_num = u16::from_le(ebpb.backup_boot_sector_num);
        ebpb.volumn_id = u32::from_le(ebpb.volumn_id);
        ebpb.bootable_signature = u16::from_le(ebpb.bootable_signature);
        ebpb
    }

    /// Reads the FAT32 extended BIOS parameter block from sector `sector` of
    /// device `device`.
    ///
    /// # Errors
    ///
    /// If the EBPB signature is invalid, returns an error of `BadSignature`.
    pub fn from<T: BlockDevice>(mut device: T, sector: u64) -> Result<BiosParameterBlock, Error> {
        let mut buffer = [0u8; core::mem::size_of::<BiosParameterBlock>()];
        match device.read_sector(sector, &mut buffer) {
            Ok(_)      => {},
            Err(error) => {
                return Err(Error::Io(error));
            }
        };

        let ebpb : BiosParameterBlock = Self::change_byte_order(
            unsafe {
                core::mem::transmute(buffer)
            }
        );

        if ebpb.bootable_signature != 0xAA55 {
            return Err(Error::BadSignature);
        }
        Ok(ebpb)
    }
}

impl fmt::Debug for BiosParameterBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Bios Parameter Block")
            .field("OEM identifier", &self.oem_identifier)
            .field("bytes per sector", &self.byte_per_sector)
            .field("sectors per cluster", &self.sector_per_cluster)
            .field("number of reserved sectors", &self.reserved_sector_num)
            .field("number of FAT", &self.fat_num)
            .field("Max number of directory entries", &self.max_dir_entries_num)
            .field("Total logical sectors", &self.logical_sector)
            .field("Media descriptor type", &self.descriptor_type)
            .field("Number of sector per FAT", &self.sector_per_fat)
            .field("Number of sector per track", &self.sector_per_track)
            .field("Number of heads on storage media", &self.heads_num)
            .field("Number of hidden sectors", &self.hidden_sector_num)
            .field("Total logical sector", &self.total_logical_sector)
            .field("Number of sectors per FAT32", &self.sector_per_fat_32)
            .field("Flags", &self.flags)
            .field("FAT version number", &self.fat_version_num)
            .field("Cluser number of root directory", &self.root_dir_cluster_num)
            .field("Sector number of the FSInfo Structure", &self.fsinfo_sector_num)
            .field("Sector number of backup boot folder", &self.backup_boot_sector_num)
            .field("Drive number", &self.drive_num)
            .field("Flags in Windows NT", &self.win_nt_flags)
            .field("Signature", &self.signature)
            .field("VolumeID number", &self.volumn_id)
            .field("Bootable partition signature", &self.bootable_signature)
            .finish()
    }
}
