use core::iter::Chain;
use core::ops::{Deref, DerefMut};
use core::slice::Iter;

use alloc::boxed::Box;
use alloc::fmt;
use core::alloc::{GlobalAlloc, Layout};

use crate::allocator;
use crate::param::*;
use crate::vm::{PhysicalAddr, VirtualAddr};
use crate::ALLOCATOR;
use crate::console::kprintln;
use crate::console::kprint;
use aarch64::vmsa::*;
use shim::const_assert_size;

#[repr(C)]
pub struct Page([u8; PAGE_SIZE]);
const_assert_size!(Page, PAGE_SIZE);

impl Page {
    pub const SIZE: usize = PAGE_SIZE;
    pub const ALIGN: usize = PAGE_SIZE;

    fn layout() -> Layout {
        unsafe { Layout::from_size_align_unchecked(Self::SIZE, Self::ALIGN) }
    }
}

#[repr(C)]
#[repr(align(65536))]
pub struct L2PageTable {
    pub entries: [RawL2Entry; 8192],
}
const_assert_size!(L2PageTable, PAGE_SIZE);

impl L2PageTable {
    /// Returns a new `L2PageTable`
    fn new() -> L2PageTable {
        let entries = [RawL2Entry::new(0); 8192];
        L2PageTable {
            entries : entries
        }
    }

    /// Returns a `PhysicalAddr` of the pagetable.
    pub fn as_ptr(&self) -> PhysicalAddr {
        PhysicalAddr::from(&(self.entries) as *const _ as u64)
    }
}

#[derive(Copy, Clone)]
pub struct L3Entry(RawL3Entry);

impl L3Entry {
    /// Returns a new `L3Entry`.
    fn new() -> L3Entry {
        L3Entry(RawL3Entry::new(0))
    }

    /// Returns `true` if the L3Entry is valid and `false` otherwise.
    fn is_valid(&self) -> bool {
        self.0.get_value(RawL3Entry::VALID) == EntryValid::Valid
    }

    /// Extracts `ADDR` field of the L3Entry and returns as a `PhysicalAddr`
    /// if valid. Otherwise, return `None`.
    fn get_page_addr(&self) -> Option<PhysicalAddr> {
        if self.is_valid() {
            return Some(PhysicalAddr::from(self.0.get_value(RawL3Entry::ADDR)));
        }
        return None;
    }
}

#[repr(C)]
#[repr(align(65536))]
pub struct L3PageTable {
    pub entries: [L3Entry; 8192],
}
const_assert_size!(L3PageTable, PAGE_SIZE);

impl L3PageTable {
    /// Returns a new `L3PageTable`.
    fn new() -> L3PageTable {
        let entries = [L3Entry::new(); 8192];
        L3PageTable {entries: entries}
    }

    /// Returns a `PhysicalAddr` of the pagetable.
    pub fn as_ptr(&self) -> PhysicalAddr {
        PhysicalAddr::from(&(self.entries[0]) as *const _ as u64)
    }
}

#[repr(C)]
#[repr(align(65536))]
pub struct PageTable {
    pub l2: L2PageTable,
    pub l3: [L3PageTable; 2],
}

impl PageTable {
    /// Returns a new `Box` containing `PageTable`.
    /// Entries in L2PageTable should be initialized properly before return.
    fn new(perm: u64) -> Box<PageTable> {
        let mut l2 : L2PageTable = L2PageTable::new();
        let mut l3 = [L3PageTable::new(), L3PageTable::new()];
        let mut pt = Box::new(PageTable {
            l2: l2,
            l3: l3,
        });
        let mut l2_entry : RawL2Entry = RawL2Entry::new(0);
        l2_entry.set_value(1, RawL2Entry::AF);
        l2_entry.set_value(EntrySh::ISh, RawL2Entry::SH);
        l2_entry.set_value(perm, RawL2Entry::AP);
        l2_entry.set_value(EntryType::Table, RawL2Entry::TYPE);
        l2_entry.set_value(EntryValid::Valid, RawL2Entry::VALID);
        l2_entry.set_value(EntryAttr::Mem, RawL2Entry::ATTR);
        pt.l2.entries[0].set(l2_entry.get());
        pt.l2.entries[1].set(l2_entry.get());

        pt.l2.entries[0].set_masked(pt.l3[0].as_ptr().as_u64(), RawL2Entry::ADDR);
        pt.l2.entries[1].set_masked(pt.l3[1].as_ptr().as_u64(), RawL2Entry::ADDR);

        pt
    }

    /// Returns the (L2index, L3index) extracted from the given virtual address.
    /// Since we are only supporting 1GB virtual memory in this system, L2index
    /// should be smaller than 2.
    ///
    /// # Panics
    ///
    /// Panics if the virtual address is not properly aligned to page size.
    /// Panics if extracted L2index exceeds the number of L3PageTable.
    fn locate(va: VirtualAddr) -> (usize, usize) {
        if va.as_usize() % PAGE_SIZE != 0 {
            panic!("Virtual Address is not properly aligned to page size");
        }
        let virtual_adress= VirtAddr::new(va.as_u64()); 
        
        // Bits [41-29] = L2index
        let l2_index = virtual_adress.get_value(VirtAddr::L2INDEX); // shift right 29 bits, bitwise or to get the last 13 bits
        // Bits [28-16] = L3index
        let l3_index = virtual_adress.get_value(VirtAddr::L3INDEX); // shift right 16 bits, bitwise or to get the last 13 bits
        if l2_index >= 2 {
            kprintln!("l2_index: {}. va: {:?}", l2_index, va);
            panic!("L2 index is >= 2. Invalid L2 index");
        }

        (l2_index as usize, l3_index as usize)
    }

    /// Returns `true` if the L3entry indicated by the given virtual address is valid.
    /// Otherwise, `false` is returned.
    pub fn is_valid(&self, va: VirtualAddr) -> bool {
        let locate_result = PageTable::locate(va);

        let l2_index = locate_result.0;
        let l3_index = locate_result.1;

        let l2_entry = self.l2.entries[l2_index];
        if l2_entry.get_value(RawL2Entry::VALID) == EntryValid::Invalid { // l2 entry not valid
            return false;
        }

        let l3_entry = self.l3[l2_index].entries[l3_index];
        if l3_entry.0.get_value(RawL3Entry::VALID) == EntryValid::Invalid { // l3 entry not valid 
            return false;
        }

        return true;
    }

    /// Returns `true` if the L3entry indicated by the given virtual address is invalid.
    /// Otherwise, `true` is returned.
    pub fn is_invalid(&self, va: VirtualAddr) -> bool {
        !self.is_valid(va)
    }

    /// Set the given RawL3Entry `entry` to the L3Entry indicated by the given virtual
    /// address.
    pub fn set_entry(&mut self, va: VirtualAddr, entry: RawL3Entry) -> &mut Self {
        let locate_result = PageTable::locate(va);

        let l2_index = locate_result.0;
        let l3_index = locate_result.1;

        self.l3[l2_index].entries[l3_index].0 = entry;
        self
    }

    /// Returns a base address of the pagetable. The returned `PhysicalAddr` value
    /// will point the start address of the L2PageTable.
    pub fn get_baddr(&self) -> PhysicalAddr {
        self.l2.as_ptr()
    }
}

// FIXME: Implement `IntoIterator` for `&PageTable`.
impl<'a> IntoIterator for &'a mut PageTable {
    type Item = &'a L3Entry;
    type IntoIter = core::iter::Chain<core::slice::Iter<'a, L3Entry>, core::slice::Iter<'a, L3Entry>>;

    fn into_iter(self) -> Self::IntoIter {
        let first_iter = self.l3[0].entries.iter();
        let second_iter = self.l3[1].entries.iter();
        first_iter.chain(second_iter)
    }
}


pub struct KernPageTable(Box<PageTable>);

impl KernPageTable {
    /// Returns a new `KernPageTable`. `KernPageTable` should have a `Pagetable`
    /// created with `KERN_RW` permission.
    ///
    /// Set L3entry of ARM physical address starting at 0x00000000 for RAM and
    /// physical address range from `IO_BASE` to `IO_BASE_END` for peripherals.
    /// Each L3 entry should have correct value for lower attributes[10:0] as well
    /// as address[47:16]. Refer to the definition of `RawL3Entry` in `vmsa.rs` for
    /// more details.
    pub fn new() -> KernPageTable {
        let mut kern_pt = PageTable::new(EntryPerm::KERN_RW);
        let mut start_addr = 0x0;
        let end_addr = allocator::memory_map().unwrap().1;

        loop {
            if start_addr + PAGE_SIZE > end_addr {
                break
            }
            let mut entry : RawL3Entry = RawL3Entry::new(0u64);
            entry.set_value(EntryValid::Valid, RawL3Entry::VALID); // set entry to valid 
            entry.set_value(PageType::Page, RawL3Entry::TYPE); // set entry type to memory block
            entry.set_value(EntryPerm::KERN_RW, RawL3Entry::AP);
            entry.set_value(1u64, RawL3Entry::AF);
            entry.set_value(EntryAttr::Mem, RawL3Entry::ATTR); // normal mem attribute
            entry.set_value(EntrySh::ISh, RawL3Entry::SH); // inner sharable for normal memory entry
            entry.set_masked(start_addr as u64, RawL3Entry::ADDR);
            kern_pt.set_entry(VirtualAddr::from(start_addr), entry);
            kprint!("");
            start_addr += PAGE_SIZE;
        }

        start_addr = IO_BASE;
        loop {
            if start_addr + PAGE_SIZE > IO_BASE_END {
                break
            }
            let mut entry : RawL3Entry = RawL3Entry::new(0u64);
            entry.set_value(EntryValid::Valid, RawL3Entry::VALID); // set entry to valid 
            entry.set_value(PageType::Page, RawL3Entry::TYPE); // set entry type to memory block
            entry.set_value(EntryPerm::KERN_RW, RawL3Entry::AP);
            entry.set_value(1u64, RawL3Entry::AF);
            entry.set_value(EntryAttr::Dev, RawL3Entry::ATTR); // device mem attribute
            entry.set_value(EntrySh::OSh, RawL3Entry::SH); // outter sharable for device memory entry
            entry.set_masked(start_addr as u64, RawL3Entry::ADDR);
            kern_pt.set_entry(VirtualAddr::from(start_addr), entry);

            start_addr += PAGE_SIZE;
        }

        KernPageTable(kern_pt)
    }
}

pub enum PagePerm {
    RW,
    RO,
    RWX,
}

pub struct UserPageTable(Box<PageTable>);

impl UserPageTable {
    /// Returns a new `UserPageTable` containing a `PageTable` created with
    /// `USER_RW` permission.
    pub fn new() -> UserPageTable {
        let user_pt = PageTable::new(EntryPerm::USER_RW);
        UserPageTable(user_pt)
    }

    /// Allocates a page and set an L3 entry translates given virtual address to the
    /// physical address of the allocated page. Returns the allocated page.
    ///
    /// # Panics
    /// Panics if the virtual address is lower than `USER_IMG_BASE`.
    /// Panics if the virtual address has already been allocated.
    /// Panics if allocator fails to allocate a page.
    ///
    /// TODO. use Result<T> and make it failurable
    /// TODO. use perm properly
    pub fn alloc(&mut self, va: VirtualAddr, _perm: PagePerm) -> &mut [u8] {
        if va.as_usize() < USER_IMG_BASE {
            kprintln!("Invalid VA. Va < USER_IMG_BASE");
            panic!("Invalid VA. Va < USER_IMG_BASE");
        }

        let page = unsafe {ALLOCATOR.alloc(Page::layout())};
        if page == core::ptr::null_mut() {
            kprintln!("Allocation fail");
            panic!("Allocation fail");
        }
        let real_va = va.as_usize() - USER_IMG_BASE;
        if self.is_valid(VirtualAddr::from(real_va)) { // already allocated
            kprintln!("VA already allocated");
            panic!("VA already allocated");
        }
        let mut entry : RawL3Entry = RawL3Entry::new(0u64);
        entry.set_value(EntryValid::Valid, RawL3Entry::VALID); // set entry to valid 
        entry.set_value(PageType::Page, RawL3Entry::TYPE); // set entry type to memory block
        entry.set_value(EntryPerm::USER_RW, RawL3Entry::AP);
        entry.set_value(1u64, RawL3Entry::AF);
        entry.set_value(EntryAttr::Mem, RawL3Entry::ATTR); // normal mem attribute
        entry.set_value(EntrySh::ISh, RawL3Entry::SH); // inner sharable for normal memory entry
        entry.set_masked(&page as *const _ as u64, RawL3Entry::ADDR);
        let phys_addr = entry.get_value(RawL3Entry::ADDR);
        self.set_entry(VirtualAddr::from(real_va), entry);

        let pa15 = VirtAddr::new(real_va as u64).get_value(VirtAddr::PA);
        let real_pa = phys_addr << 16 | pa15;

        unsafe {core::slice::from_raw_parts_mut(real_pa as *mut u8, PAGE_SIZE)}
        
    }
}

impl Deref for KernPageTable {
    type Target = PageTable;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for UserPageTable {
    type Target = PageTable;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for KernPageTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl DerefMut for UserPageTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// FIXME: Implement `Drop` for `UserPageTable`.
// FIXME: Implement `fmt::Debug` as you need.

// To implement Drop traits, iterate the internal pagetable and dealloc() each entry that exists.
impl Drop for UserPageTable {
    fn drop(&mut self) {
        for entry in (*self).into_iter() {
            if entry.0.get_value(RawL3Entry::VALID) == EntryValid::Valid { // entry is valid, dealloc
                let ptr : *mut u8 = entry.0.get_value(RawL3Entry::ADDR) as *mut u8;
                let layout : Layout = Page::layout();
                unsafe {ALLOCATOR.dealloc(ptr, layout)};
            }
        }
    }
}

// FIXME: Implement `fmt::Debug` as you need.

impl fmt::Debug for UserPageTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UserPageTable").unwrap();
        Ok(())
    }
}