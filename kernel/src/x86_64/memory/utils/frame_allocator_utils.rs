use crate::{
    debug,
    utils::assert::assert_or_panic,
    x86_64::{
        instructions::flush_tlb,
        memory::{
            paging::create_table_entry_from_hhdm_address, ALLOC_ENTRY, BITMAP_ENTRY,
            BITMAP_MAX_OFFSET, HIGHER_HALF_MASK,
        },
    },
};

pub fn read_bitmap_index(index: usize) -> u8 {
    unsafe { *((BITMAP_ENTRY | HIGHER_HALF_MASK) as *const u8).add(index) }
}

pub unsafe fn assert_allocator_page_tables() {
    let expected_value: u64 = 0xABAB_ABAB_ABAB_ABAB;
    debug!(
        "Loading Test Value {:016X} at virtual address: [{:016X}]",
        expected_value,
        (0x0000_0000_0000_0000 as *mut u64).add(1) as u64
    );
    *((0x0000_0000_0000_0000 as *mut u64).add(1)) = expected_value;
    debug!(
        "Reading value at HHDM Address: [{:016X}]",
        ((ALLOC_ENTRY | HIGHER_HALF_MASK) as *const u64).add(1) as u64
    );
    let test_value = *(((ALLOC_ENTRY | HIGHER_HALF_MASK) as *const u64).add(1));

    assert_or_panic(expected_value == test_value, "PFA Page Tables Init");
}

pub unsafe fn assert_allocator_page_tables_setup(pdt_address: u64) {
    let asset_address = pdt_address + 0x1000;
    debug!(
        "Creating Page Table Entry for physical address: [{:016X}]",
        ALLOC_ENTRY
    );
    let test_entry = create_table_entry_from_hhdm_address(ALLOC_ENTRY);
    *(asset_address as *mut u64) = test_entry;
}

pub unsafe fn assert_allocator_page_tables_teardown(pdt_address: u64) {
    debug!("PFA Page Tables Test Teardown");
    // Clear Test Value
    *((0x0000_0000_0000_0000 as *mut u64).add(1)) = 0;

    // Clear Page Table Entry
    let asset_address = pdt_address + 0x1000;
    *(asset_address as *mut u64) = 0;

    // Reflushes TLB
    flush_tlb();
}

pub fn allocate_frame() {
    let mut i = 0_usize;
    let mut bitmap_value = 0;
    while i < BITMAP_MAX_OFFSET as usize {
        bitmap_value = read_bitmap_index(i);
        if bitmap_value != 0xFF {
            break;
        }

        i += 1;
    }

    if i == BITMAP_MAX_OFFSET as usize {
        panic!("Bitmap out of space")
    } else {
    }

    let first_free_entry;
    for bit in 0..8 {
        if bitmap_value & (0x80 >> bit) == 0 {
            first_free_entry = bit;
            break;
        }
    }
}
