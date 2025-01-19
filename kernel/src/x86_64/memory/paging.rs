use crate::x86_64::instructions::read_cr3;

use super::{HIGHER_HALF_MASK, PAGE_ENTRY_PHYSICAL_MASK};

pub unsafe fn clear_table(address: u64) {
    for i in 0..512 {
        *((address as *mut u64).add(i)) = 0;
    }
}

pub fn create_table_entry_from_hhdm_address(address: u64) -> u64 {
    (address & 0x0000_7FFF_FFFF_F000) | 0x3
}

pub struct PageTable {
    entry: u64,
}

impl PageTable {
    pub unsafe fn hhdm_page_address_from_indexes(
        pml4_index: usize,
        pdpt_index: usize,
        pdt_index: usize,
        pt_index: usize,
    ) -> u64 {
        let cr3 = read_cr3() | HIGHER_HALF_MASK;
        let pdpt_address =
            ((*(cr3 as *const u64).add(pml4_index)) & PAGE_ENTRY_PHYSICAL_MASK) | HIGHER_HALF_MASK;
        let pdt_address = ((*(pdpt_address as *const u64).add(pdpt_index))
            & PAGE_ENTRY_PHYSICAL_MASK)
            | HIGHER_HALF_MASK;
        let pt_address = ((*(pdt_address as *const u64).add(pdt_index)) & PAGE_ENTRY_PHYSICAL_MASK)
            | HIGHER_HALF_MASK;
        let page_address = (pt_address as *const u64).add(pt_index);

        page_address as u64
    }
}

#[repr(transparent)]
struct PageTableOptions(u64);

impl PageTableOptions {
    fn default() -> PageTableOptions {
        PageTableOptions(0)
    }

    fn present(&mut self) -> &mut Self {
        self.0 |= 0x1;
        self
    }

    fn writeable(&mut self) -> &mut Self {
        self.0 |= 0x2;
        self
    }
}

pub struct PageFrame(u64);

impl PageFrame {
    pub fn writable_from_physical_address(address: u64) -> PageFrame {
        let mut options = PageTableOptions::default();
        options.writeable().present();
        let masked_address = address & 0x000F_FFFF_FFFF_F000;

        PageFrame(masked_address | options.0)
    }

    pub fn assign_to_pt_direct(&self, pt_address: u64) {
        unsafe {
            *(pt_address as *mut u64) = self.0;
        };
    }
}

pub trait PageEntryHHDM {
    fn get_physical_address_from_hhdm_address(address: u64) -> u64 {
        address & 0x0000_7FFF_FFFF_F000
    }

    fn build_entry<T>(address: u64, options: PageTableOptions) -> u64
    where
        T: PageEntryHHDM,
    {
        T::get_physical_address_from_hhdm_address(address) | options.0
    }

    fn _clear_table(address: u64) -> () {
        unsafe {
            for i in 0..512 {
                *((address as *mut u64).add(i)) = 0;
            }
        }
    }

    fn clear_table(&self);
}

pub struct PageTableEntryHHDM(u64, u64);

impl PageTableEntryHHDM {
    pub fn writeable_from_hhdm_address(address: u64) -> PageTableEntryHHDM {
        let mut options = PageTableOptions::default();
        options.writeable().present();

        let entry = PageTableEntryHHDM::build_entry::<PageTableEntryHHDM>(address, options);

        PageTableEntryHHDM(entry, address)
    }

    pub fn assign_to_pdt(&self, pdt_address: u64, index: usize) {
        unsafe {
            *(pdt_address as *mut u64).add(index) = self.0;
        };
    }
}

impl PageEntryHHDM for PageTableEntryHHDM {
    fn clear_table(&self) {
        PageTableEntryHHDM::_clear_table(self.1);
    }
}

pub struct PageDirectoryTableEntryHHDM(pub u64, pub u64);

impl PageDirectoryTableEntryHHDM {
    pub fn writeable_from_hhdm_address(address: u64) -> PageDirectoryTableEntryHHDM {
        let mut options = PageTableOptions::default();
        options.writeable().present();

        let entry = PageDirectoryTableEntryHHDM::build_entry::<PageDirectoryTableEntryHHDM>(
            address, options,
        );

        PageDirectoryTableEntryHHDM(entry, address)
    }

    pub fn assign_to_pdpt(&self, pdpt_address: u64, index: usize) {
        unsafe {
            *(pdpt_address as *mut u64).add(index) = self.0;
        };
    }
}

impl PageEntryHHDM for PageDirectoryTableEntryHHDM {
    fn clear_table(&self) {
        PageDirectoryTableEntryHHDM::_clear_table(self.1);
    }
}

pub struct PageDirectoryPointerTableEntryHHDM(pub u64, pub u64);

impl PageDirectoryPointerTableEntryHHDM {
    pub fn writeable_from_hhdm_address(address: u64) -> PageDirectoryPointerTableEntryHHDM {
        let mut options = PageTableOptions::default();
        options.writeable().present();

        let entry = PageDirectoryPointerTableEntryHHDM::build_entry::<
            PageDirectoryPointerTableEntryHHDM,
        >(address, options);

        PageDirectoryPointerTableEntryHHDM(entry, address)
    }

    pub fn assign_to_pml4(&self, pml4_address: u64, index: usize) {
        unsafe {
            *(pml4_address as *mut u64).add(index) = self.0;
        };
    }
}

impl PageEntryHHDM for PageDirectoryPointerTableEntryHHDM {
    fn clear_table(&self) {
        PageDirectoryPointerTableEntryHHDM::_clear_table(self.1);
    }
}
