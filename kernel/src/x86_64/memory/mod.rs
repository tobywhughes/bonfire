pub mod frame_allocator;
pub mod paging;
mod utils;

const HIGHER_HALF_MASK: u64 = 0xFFFF_8000_0000_0000;
const PAGE_ENTRY_PHYSICAL_MASK: u64 = 0x0000_FFFF_FFFF_F000;

// The goal is to get rid of these and have a way to automatically determine them
// But they work for now
const BITMAP_ENTRY: u64 = 0x0000_0000_0010_0000;
const BITMAP_MAX_OFFSET: u64 = 0x0000_0000_0070_0000;
const ALLOC_ENTRY: u64 = 0x0000_0000_0090_0000;
const ALLOC_PAGE_TABLES: u64 = 0x7F7F_F000 | HIGHER_HALF_MASK;
