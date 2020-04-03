use core::alloc::Layout;
use core::fmt;


use crate::allocator::linked_list::LinkedList;
use crate::allocator::util::*;
use crate::allocator::LocalAlloc;

/// A simple allocator that allocates based on size classes.
///   bin 0 (2^3 bytes)    : handles allocations in (0, 2^3]
///   bin 1 (2^4 bytes)    : handles allocations in (2^3, 2^4]
///   ...
///   bin 29 (2^22 bytes): handles allocations in (2^31, 2^32]
///   
///   map_to_bin(size) -> k
///   

const BIN_SIZE : usize = 32;

pub struct Allocator {
    // there's bins in allocator.
    // bins stores a list of linkedlists with the size of 30
    bins : [LinkedList; BIN_SIZE]
}

impl Allocator {
    /// Creates a new bin allocator that will allocate memory from the region
    /// starting at address `start` and ending at address `end`.
    pub fn new(start: usize, end: usize) -> Allocator {
        let mut bins = [LinkedList::new(); BIN_SIZE];
        let mut real_start = start;
        let mut count : usize = start.trailing_zeros() as usize;
        while real_start <= end { // loop everytime start is still less than end
            let size : usize = 1 << count;
            if real_start + size <= end {
                unsafe {
                    // use size.trailing_zeros() as index
                    // for example, if size = 2^3, it goes into bins[3]
                    // This means bins[n] for n = 0,1,2 is empty. 
                    bins[count].push(real_start as *mut usize);
                }
            }
            real_start += size;
            count += 1;
        } 
        Allocator {
            bins : bins
        }
    }
}

impl LocalAlloc for Allocator {
    /// Allocates memory. Returns a pointer meeting the size and alignment
    /// properties of `layout.size()` and `layout.align()`.
    ///
    /// If this method returns an `Ok(addr)`, `addr` will be non-null address
    /// pointing to a block of storage suitable for holding an instance of
    /// `layout`. In particular, the block will be at least `layout.size()`
    /// bytes large and will be aligned to `layout.align()`. The returned block
    /// of storage may or may not have its contents initialized or zeroed.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure that `layout.size() > 0` and that
    /// `layout.align()` is a power of two. Parameters not meeting these
    /// conditions may result in undefined behavior.
    ///
    /// # Errors
    ///
    /// Returning null pointer (`core::ptr::null_mut`)
    /// indicates that either memory is exhausted
    /// or `layout` does not meet this allocator's
    /// size or alignment constraints.
    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let null : *mut u8 = core::ptr::null_mut();
        if layout.size() <= 0 {
            return null;
        }
        if !(layout.align().is_power_of_two()) {
            return null;
        } 
        // check exhaustion
        let size = align_up(layout.size().next_power_of_two(), core::cmp::max(layout.align(), 1 << 4));
        let index : usize = size.trailing_zeros() as usize; // index into the bins list
        for i in index..self.bins.len() { // going from the index bins toward the end
            if self.bins[i].is_empty() { // if the bin is empty, no memory to use
                continue
            }
            // if the bin is not empty, pop the first address on that bin for us to use
            let block_addr = self.bins[i].pop().unwrap() as *mut u8;
            // if we have skipped some bins, there will be external fragmentation 
            // in using our current bins because our current bins will be much bigger 
            // than the memory needed.
            for j in index..i {
                // add to the bins before the free memory that won't be used in this current bin
                let block_addr_align = align_up(block_addr.add(1 << j) as usize, layout.align());
                self.bins[j].push(block_addr_align as *mut usize);
            }
            return align_up(block_addr as usize, layout.align()) as *mut u8;
        } 
        return null;
    }

    /// Deallocates the memory referenced by `ptr`.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure the following:
    ///
    ///   * `ptr` must denote a block of memory currently allocated via this
    ///     allocator
    ///   * `layout` must properly represent the original layout used in the
    ///     allocation call that returned `ptr`
    ///
    /// Parameters not meeting these conditions may result in undefined
    /// behavior.
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        //Deallocation pushes an item to the linked list in the corresponding bin.
        let size = align_up(layout.size().next_power_of_two(), layout.align());
        let index : usize = size.trailing_zeros() as usize; // index into the bins list

        self.bins[index].push(ptr as *mut usize);
    }
}

impl fmt::Debug for Allocator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bin allocator with {:?}", self.bins)
    }
}