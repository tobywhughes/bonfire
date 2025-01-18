use limine::memory_map::{Entry, EntryType};
use limine::request::MemoryMapRequest;

use crate::debug;

#[used]
#[link_section = ".requests"]
static MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

fn map_entry_type(entry_type: EntryType) -> &'static str {
    match (entry_type) {
        EntryType::USABLE => "Usable",
        EntryType::RESERVED => "Reserved",
        EntryType::ACPI_RECLAIMABLE => "ACPI Reclaimable",
        EntryType::ACPI_NVS => "ACPI NVS",
        EntryType::BAD_MEMORY => "Bad Memory",
        EntryType::BOOTLOADER_RECLAIMABLE => "Bootloader Reclaimable",
        EntryType::KERNEL_AND_MODULES => "Kernal And Modules",
        EntryType::FRAMEBUFFER => "Framebuffer",
        _ => "Unknown",
    }
}

// This is just a temp function hwile I figure out what things actually look like
pub unsafe fn mmap_scratch() {
    let mmmap = MEMORY_MAP_REQUEST.get_response().unwrap();
    let mmap_entries = mmmap.entries();
    for entry in mmap_entries {
        debug!(
            "{:016X} {:016X} {}",
            entry.base,
            entry.length,
            map_entry_type(entry.entry_type)
        )
    }

    sum_entry_types(mmap_entries);
}

fn sum_entry_types(entries: &[&Entry]) {
    let mut usable_total = 0;
    let mut bootloader_reclaimable_total = 0;

    for entry in entries {
        match entry.entry_type {
            EntryType::USABLE => usable_total += entry.length,
            EntryType::BOOTLOADER_RECLAIMABLE => bootloader_reclaimable_total += entry.length,
            _ => (),
        };
    }

    debug!(
        "[USABLE]: {:016X}  [BL RECLAIM]: {:016X}",
        usable_total, bootloader_reclaimable_total
    );
}
