
/// Align `addr` downwards to the nearest multiple of `align`.
///
/// The returned usize is always <= `addr.`
///
/// # Panics
///
/// Panics if `align` is not a power of 2.
pub fn align_down(addr: usize, align: usize) -> usize {
    if !(align.is_power_of_two()) {
        panic!("Align is not a power of 2. Panic in align_down");
    }
    let mask = !(align - 1); // mask to only take the first 1, zero out the rest to round down
    addr & mask
}

/// Align `addr` upwards to the nearest multiple of `align`.
///
/// The returned `usize` is always >= `addr.`
///
/// # Panics
///
/// Panics if `align` is not a power of 2
/// or aligning up overflows the address.
pub fn align_up(addr: usize, align: usize) -> usize {
    if !(align.is_power_of_two()){
        panic!("Align is not a power of 2. Panic in align_up");
    }

    match addr.checked_add(align - 1) { // add up and then we just round down
        Some(address) => {
            let mask = !(align - 1);
            if address & mask <= core::usize::MAX {
                return address & mask;
            } else {
                panic!("Aligning up overflows the address");
            }
        },
        None          => {
            panic!("Aligning up overflows the address");
        }
    }
}
