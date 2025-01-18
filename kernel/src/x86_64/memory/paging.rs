pub unsafe fn clear_table(address: u64) {
    for i in 0..512 {
        *((address as *mut u64).add(i)) = 0;
    }
}

pub fn create_table_entry_from_hhdm_address(address: u64) -> u64 {
    (address & 0x0000_7FFF_FFFF_F000) | 0x3
}
