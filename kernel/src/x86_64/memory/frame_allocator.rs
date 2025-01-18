use crate::x86_64::instructions::{flush_tlb, read_cr3};

use super::{
    paging::{clear_table, create_table_entry_from_hhdm_address},
    utils::frame_allocator_utils::{
        assert_allocator_page_tables, assert_allocator_page_tables_setup,
        assert_allocator_page_tables_teardown,
    },
    ALLOC_PAGE_TABLES, HIGHER_HALF_MASK,
};

// Basic page table set up to be used by allocator
// Each PT is 4KB, 512 entires makes 2MB, 512 PDT entries means 1GB of adressable space
pub unsafe fn init_allocator_page_tables() {
    let cr3: u64 = HIGHER_HALF_MASK | read_cr3();

    //Creates PDPT Entry in PML4 - Index 0
    let pdpt_address = ALLOC_PAGE_TABLES;
    clear_table(pdpt_address);
    let pml4_entry = create_table_entry_from_hhdm_address(pdpt_address);
    *(cr3 as *mut u64) = pml4_entry;

    //Creates PDT Entry in PDPT - Index 0
    let pdt_address = pdpt_address + 0x1000;
    clear_table(pdt_address);
    let pdpt_entry = create_table_entry_from_hhdm_address(pdt_address);
    *(pdpt_address as *mut u64) = pdpt_entry;

    //Creates 0..512 PDT entries and page table for each entry
    for pdt_index in 0..512_u64 {
        let pt_address = pdt_address + ((pdt_index + 1) * 0x1000);
        clear_table(pt_address);
        let pdt_entry = create_table_entry_from_hhdm_address(pt_address);
        *(pdt_address as *mut u64).add(pdt_index as usize) = pdt_entry;
    }

    assert_allocator_page_tables_setup(pdt_address);

    flush_tlb();

    assert_allocator_page_tables();

    assert_allocator_page_tables_teardown(pdt_address);
}
