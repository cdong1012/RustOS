use core::fmt;

use alloc::string::String;

use crate::traits;

/// A date as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Date(u16);

/// Time as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Time(u16);

/// File attributes as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Attributes(pub u8);

/// A structure containing a date and time.
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct Timestamp {
    pub date: Date,
    pub time: Time,
}

/// Metadata for a directory entry.
#[derive(Default, Debug, Clone)]
pub struct Metadata {
    pub timestamp_created : Timestamp,
    pub timestamp_acessed : Timestamp,
    pub timestamp_modified : Timestamp,
    pub attri : Attributes
}

impl Attributes {
    const READ_ONLY: u8 = 0x01;
    const HIDDEN   : u8 = 0x02;
    const SYSTEM   : u8 = 0x04;
    const VOLUME_ID: u8 = 0x08;
    const DIRECTORY: u8 = 0x10;
    const ARCHIVE  : u8 = 0x20;
    const LFN      : u8 = Self::READ_ONLY | Self::HIDDEN | Self::SYSTEM | Self::VOLUME_ID;

    pub fn read_only(&self) -> bool {
        self.0 & Self::READ_ONLY == Self::READ_ONLY
    }

    pub fn hidden(&self) -> bool {
        self.0 & Self::HIDDEN == Self::HIDDEN
    }

    pub fn system(&self) -> bool {
        self.0 & Self::SYSTEM == Self::SYSTEM
    }

    pub fn volume_id(&self) -> bool {
        self.0 & Self::VOLUME_ID == Self::VOLUME_ID
    }

    pub fn directory(&self) -> bool {
        self.0 & Self::DIRECTORY == Self::DIRECTORY
    }

    pub fn archive(&self) -> bool {
        self.0 & Self::ARCHIVE == Self::ARCHIVE
    }

    pub fn lfn(&self) -> bool {
        self.0 == Self::LFN
    }
}

impl traits::Timestamp for Timestamp {
    /// The calendar year.
    ///
    /// The year is not offset. 2009 is 2009.
    fn year(&self) -> usize{
        // date bits 15 to 9
        (self.date.0 >> 9)as usize + 1980 as usize
    }

    /// The calendar month, starting at 1 for January. Always in range [1, 12].
    ///
    /// January is 1, Feburary is 2, ..., December is 12.
    fn month(&self) -> u8 {
        ((self.date.0 >> 5) & 0b111) as u8
    }

    // The calendar day, starting at 1. Always in range [1, 31].
    fn day(&self) -> u8 {
        (self.date.0 & 0b11111) as u8
    }

    /// The 24-hour hour. Always in range [0, 24).
    fn hour(&self) -> u8 {
        (self.time.0 >> 11) as u8
    }

    /// The minute. Always in range [0, 60).
    fn minute(&self) -> u8 {
        ((self.time.0 >> 5) & 0b111111) as u8
    }

    /// The second. Always in range [0, 60).
    fn second(&self) -> u8 {
        ((self.time.0 & 0b11111) as u8)*2
    }
}

impl traits::Metadata for Metadata {
    type Timestamp = Timestamp;
    /// Whether the associated entry is read only.
    fn read_only(&self) -> bool {
        self.attri.read_only()
    }

    /// Whether the entry should be "hidden" from directory traversals.
    fn hidden(&self) -> bool {
        self.attri.hidden()
    }

    /// The timestamp when the entry was created.
    fn created(&self) -> Self::Timestamp {
        self.timestamp_created
    }

    /// The timestamp for the entry's last access.
    fn accessed(&self) -> Self::Timestamp {
        self.timestamp_acessed
    }

    /// The timestamp for the entry's last modification.
    fn modified(&self) -> Self::Timestamp {
        self.timestamp_modified
    }
}

impl fmt::Display for Metadata {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MetaData:")
            .field("Access time", &self.timestamp_acessed)
            .field("Created time", &self.timestamp_created)
            .field("Modified time", &self.timestamp_modified)
            .finish()
    }
}