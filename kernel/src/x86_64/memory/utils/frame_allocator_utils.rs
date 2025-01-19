use crate::{
    debug,
    utils::assert::assert_or_panic,
    x86_64::{
        instructions::flush_tlb,
        memory::{
            paging::{create_table_entry_from_hhdm_address, PageTable},
            ALLOC_ENTRY, BITMAP_ENTRY, BITMAP_MAX_OFFSET, HIGHER_HALF_MASK,
        },
    },
};



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

pub unsafe fn assert_allocator_page_tables_setup() {
    let assert_address = PageTable::hhdm_page_address_from_indexes(0, 0, 0, 0);
    debug!(
        "Creating Page Table Entry for physical address: [{:016X}]",
        ALLOC_ENTRY
    );
    let test_entry = create_table_entry_from_hhdm_address(ALLOC_ENTRY);
    *(assert_address as *mut u64) = test_entry;
}

pub unsafe fn assert_allocator_page_tables_teardown() {
    debug!("PFA Page Tables Test Teardown");
    // Clear Test Value
    *((0x0000_0000_0000_0000 as *mut u64).add(1)) = 0;

    // Clear Page Table Entry
    let assert_address = PageTable::hhdm_page_address_from_indexes(0, 0, 0, 0);
    *(assert_address as *mut u64) = 0;

    // Reflushes TLB
    flush_tlb();
}
