use crate::{println, x86_64::memory::PAGE_ENTRY_PHYSICAL_MASK};

fn get_page_indexes_from_address(address: u64) -> (u64, u64, u64, u64) {
    (
        ((address >> 39) & 0x1FF),
        ((address >> 30) & 0x1FF),
        ((address >> 21) & 0x1FF),
        ((address >> 12) & 0x1FF),
    )
}

fn print_page_indexes_from_address(address: u64) {
    let (pml4_index, pdpt_index, pdt_index, pt_index) = get_page_indexes_from_address(address);
    println!(
        "PML4 {} PDPT {} PDT {} PT {}",
        pml4_index, pdpt_index, pdt_index, pt_index
    )
}

unsafe fn entries_for_table_with_values(address: u64) {
    for i in 0..512 {
        let pt = *((address as *const u64).add(i as usize));
        if pt > 0 {
            println!("{} {:016X} {:016X}", i, pt, pt & PAGE_ENTRY_PHYSICAL_MASK);
        }
    }
}
