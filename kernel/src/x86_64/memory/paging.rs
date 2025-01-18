use core::ops::Add;

use crate::{
    debug, println,
    utils::assert::assert_or_panic,
    x86_64::{
        address,
        instructions::{flush_tlb, read_cr3},
    },
};

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

unsafe fn clear_table(address: u64) {
    for i in 0..512 {
        *((address as *mut u64).add(i)) = 0;
    }
}

const HIGHER_HALF_MASK: u64 = 0xFFFF_8000_0000_0000;
const PAGE_ENTRY_PHYSICAL_MASK: u64 = 0x0000_FFFF_FFFF_F000;

const BITMAP_ENTRY: u64 = 0x0000_0000_0010_0000;
const BITMAP_MAX_OFFSET: u64 = 0x0000_0000_0070_0000;
static mut BITMAP_OFFSET: u64 = 0;

const ALLOC_ENTRY: u64 = 0x0000_0000_0090_0000;

const ALLOC_PAGE_TABLES: u64 = 0x7F7F_F000 | HIGHER_HALF_MASK;

pub fn read_bitmap_index(index: usize) -> u8 {
    unsafe { *((BITMAP_ENTRY | HIGHER_HALF_MASK) as *const u8).add(index) }
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

fn create_table_entry_from_hhdm_address(address: u64) -> u64 {
    (address & 0x0000_7FFF_FFFF_F000) | 0x3
}

// Basic page table set up to be used by allocator
// Each PT is 4KB, 512 entires makes 2MB, 512 PDT entries means 1GB of adressable space
pub unsafe fn init_allocator_page_tables() {
    let cr3: u64 = HIGHER_HALF_MASK | read_cr3();

    //Creates PDPT Entry in PML4 - Index 0
    let pdpt_address = ALLOC_PAGE_TABLES;
    clear_table(pdpt_address);
    let pml4_entry = create_table_entry_from_hhdm_address(pdpt_address);
    *(cr3 as *mut u64) = pml4_entry;
    // entries_for_table_with_values(cr3);

    //Creates PDT Entry in PDPT - Index 0
    let pdt_address = pdpt_address + 0x1000;
    clear_table(pdt_address);
    let pdpt_entry = create_table_entry_from_hhdm_address(pdt_address);
    *(pdpt_address as *mut u64) = pdpt_entry;
    // entries_for_table_with_values(pdpt_address);

    //Creates 0..512 PDT entries and page table for each entry
    for pdt_index in 0..512_u64 {
        let pt_address = pdt_address + ((pdt_index + 1) * 0x1000);
        clear_table(pt_address);
        let pdt_entry = create_table_entry_from_hhdm_address(pt_address);
        *(pdt_address as *mut u64).add(pdt_index as usize) = pdt_entry;
    }
    // entries_for_table_with_values(pdt_address);

    assert_allocator_page_tables_setup(pdt_address);

    flush_tlb();

    //test
    assert_allocator_page_tables();

    assert_allocator_page_tables_teardown(pdt_address);
}

unsafe fn assert_allocator_page_tables() {
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

unsafe fn assert_allocator_page_tables_setup(pdt_address: u64) {
    let asset_address = pdt_address + 0x1000;
    debug!(
        "Creating Page Table Entry for physical address: [{:016X}]",
        ALLOC_ENTRY
    );
    let test_entry = create_table_entry_from_hhdm_address(ALLOC_ENTRY);
    *(asset_address as *mut u64) = test_entry;
}

unsafe fn assert_allocator_page_tables_teardown(pdt_address: u64) {
    debug!("PFA Page Tables Test Teardown");
    // Clear Test Value
    *((0x0000_0000_0000_0000 as *mut u64).add(1)) = 0;

    // Clear Page Table Entry
    let asset_address = pdt_address + 0x1000;
    *(asset_address as *mut u64) = 0;

    // Reflushes TLB
    flush_tlb();
}

pub unsafe fn map_entry() {
    let cr3: u64 = read_cr3();

    println!("CR3: {:016X}", cr3);

    let cr3_address = HIGHER_HALF_MASK | (cr3);
}

//7FE0300
// 0x0010 0000

//7F7FF000 601000
