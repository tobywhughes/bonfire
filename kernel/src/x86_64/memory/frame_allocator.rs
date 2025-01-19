use core::fmt;

use crate::{
    debug,
    x86_64::instructions::{flush_tlb, read_cr3},
};

use super::{
    paging::{
        clear_table, create_table_entry_from_hhdm_address, PageDirectoryPointerTableEntryHHDM,
        PageDirectoryTableEntryHHDM, PageEntryHHDM, PageFrame, PageTable, PageTableEntryHHDM,
    },
    utils::frame_allocator_utils::{
        assert_allocator_page_tables, assert_allocator_page_tables_setup,
        assert_allocator_page_tables_teardown,
    },
    ALLOC_ENTRY, ALLOC_PAGE_TABLES, BITMAP_ENTRY, BITMAP_MAX_OFFSET, HIGHER_HALF_MASK,
};

// Basic page table set up to be used by allocator
// Each PT is 4KB, 512 entires makes 2MB, 512 PDT entries means 1GB of adressable space
pub unsafe fn init_allocator_page_tables() {
    let cr3: u64 = HIGHER_HALF_MASK | read_cr3();

    //Creates PDPT Entry in PML4 - Index 0

    let pdpt_entry =
        PageDirectoryPointerTableEntryHHDM::writeable_from_hhdm_address(ALLOC_PAGE_TABLES);
    pdpt_entry.clear_table();
    pdpt_entry.assign_to_pml4(cr3, 0);

    //Creates PDT Entry in PDPT - Index 0
    let pdt_entry = PageDirectoryTableEntryHHDM::writeable_from_hhdm_address(pdpt_entry.1 + 0x1000);
    pdt_entry.clear_table();
    pdt_entry.assign_to_pdpt(pdpt_entry.1, 0);

    //Creates 0..512 PDT entries and page table for each entry
    for pt_index in 0..512_u64 {
        let pt_entry = PageTableEntryHHDM::writeable_from_hhdm_address(
            pdt_entry.1 + ((pt_index + 1) * 0x1000),
        );
        pt_entry.clear_table();
        pt_entry.assign_to_pdt(pdt_entry.1, pt_index as usize);
    }

    assert_allocator_page_tables_setup();

    flush_tlb();

    assert_allocator_page_tables();

    assert_allocator_page_tables_teardown();
}

pub fn read_bitmap_index(index: usize) -> u64 {
    unsafe { *((BITMAP_ENTRY | HIGHER_HALF_MASK) as *const u64).add(index) }
}

fn get_page_index_from_bitmap(page_num: usize) -> (usize, usize, usize, usize) {
    let pt_index = page_num % 512;
    let pdt_index = (page_num / 512) % 512;
    let pdpt_index = (page_num / 512 / 512) % 512;
    let pml4_index = (page_num / 512 / 512 / 512) % 512;

    if pdpt_index > 0 || pml4_index > 0 {
        panic!("PFA Error - Paging higher than 1GB currently unimplemented");
    }

    (pml4_index, pdpt_index, pdt_index, pt_index)
}

fn get_page_num(bitmap_index: usize, bit_index: usize) -> usize {
    (bitmap_index * 64) + bit_index
}

fn get_bitmap_index_and_bit_index(page_num: usize) -> (usize, usize) {
    let bitmap_index = page_num / 64;
    let bit_index = page_num % 64;
    (bitmap_index, bit_index)
}

pub fn allocate_frame() -> AllocatedPageFrame {
    let mut bitmap_index: usize = 0_usize;
    let mut bitmap_value = 0;
    while bitmap_index < (BITMAP_MAX_OFFSET / 8) as usize {
        bitmap_value = read_bitmap_index(bitmap_index);
        if bitmap_value != 0xFFFF_FFFF_FFFF_FFFF {
            break;
        }

        bitmap_index += 1;
    }

    if bitmap_index == (BITMAP_MAX_OFFSET / 8) as usize {
        panic!("Bitmap out of space")
    } else {
    }

    let mut first_free_entry = 0;
    for bit in 0..64_usize {
        if bitmap_value & (0x8000_0000_0000_0000 >> bit) == 0 {
            first_free_entry = bit;
            break;
        }
    }

    let page_num = get_page_num(bitmap_index, first_free_entry);
    let (pml4_index, pdpt_index, pdt_index, pt_index) = get_page_index_from_bitmap(page_num);

    let page_address = unsafe {
        PageTable::hhdm_page_address_from_indexes(pml4_index, pdpt_index, pdt_index, pt_index)
    };

    let physical_address = ALLOC_ENTRY + (0x1000 * page_num as u64);

    let page_frame = PageFrame::writable_from_physical_address(physical_address);
    page_frame.assign_to_pt_direct(page_address);

    flush_tlb();

    unsafe {
        *((BITMAP_ENTRY | HIGHER_HALF_MASK) as *mut u64).add(bitmap_index) |=
            0x8000_0000_0000_0000 >> first_free_entry;
    }

    debug!("Allocated frame for PAGE #{}", page_num);

    AllocatedPageFrame::FromPageNum(page_num, physical_address)
}

pub struct AllocatedPageFrame {
    pub base_virtual_address: u64,
    base_physical_address: u64,
    pub frame_size: u64,
    bitmap_page_num: usize,
}

impl AllocatedPageFrame {
    pub fn FromPageNum(bitmap_page_num: usize, base_physical_address: u64) -> AllocatedPageFrame {
        let base_virtual_address = AllocatedPageFrame::get_base_virtual_address(bitmap_page_num);

        AllocatedPageFrame {
            base_virtual_address,
            base_physical_address,
            frame_size: 0x1000, // Just handling 4KB Frames for now,
            bitmap_page_num,
        }
    }

    fn get_base_virtual_address(page_num: usize) -> u64 {
        let (pml4_index, pdpt_index, pdt_index, pt_index) = get_page_index_from_bitmap(page_num);

        // Calculate the linear address
        let address = ((pml4_index as u64) << 39)
            | ((pdpt_index as u64) << 30)
            | ((pdt_index as u64) << 21)
            | ((pt_index as u64) << 12);

        // Sign extend the address
        if address & (1 << 47) != 0 {
            address | 0xFFFF_0000_0000_0000 // Sign extension for 48-bit addresses
        } else {
            address
        }
    }

    pub fn free(&self) -> () {
        debug!("Freeing frame for frame for PAGE #{}", self.bitmap_page_num);
        let (pml4_index, pdpt_index, pdt_index, pt_index) =
            get_page_index_from_bitmap(self.bitmap_page_num);

        let page_address = unsafe {
            PageTable::hhdm_page_address_from_indexes(pml4_index, pdpt_index, pdt_index, pt_index)
        };

        unsafe { *(page_address as *mut u64) = 0 }

        flush_tlb();

        let (bitmap_index, bit_index) = get_bitmap_index_and_bit_index(self.bitmap_page_num);
        unsafe {
            *((BITMAP_ENTRY | HIGHER_HALF_MASK) as *mut u64).add(bitmap_index) &=
                !(0x8000_0000_0000_0000 >> bit_index);
        }
    }
}

impl core::fmt::Debug for AllocatedPageFrame {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("Allocated Page Frame")
            .field(
                "Base Virtual Address",
                &format_args!("0x{:016X}", self.base_virtual_address),
            )
            .field(
                "Base Physical Address",
                &format_args!("0x{:016X}", self.base_physical_address),
            )
            .field("Frame Size", &format_args!("0x{:04X}", self.frame_size))
            .field("Page Num", &format_args!("{}", self.bitmap_page_num))
            .finish()
    }
}
