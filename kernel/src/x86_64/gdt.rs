use core::fmt;

use lazy_static::lazy_static;

use crate::{
    print, println,
    utils::assert::{assert_or_panic, assert_print},
};

use super::{
    address::GDT_Pointer,
    instructions::{lgdt, load_cs, load_data_segment, sgdt},
};

// GDT Descriptors recommended by OSDEV discord

const GDT_SEGMENT_NULL: u64 = 0x0000_0000_0000_0000; // null
const GDT_SEGMENT_CODE_16: u64 = 0x0000_9a00_0000_ffff; // 16-bit code
const GDT_SEGMENT_DATA_16: u64 = 0x0000_9300_0000_ffff; // 16-bit data
const GDT_SEGMENT_CODE_32: u64 = 0x00cf_9a00_0000_ffff; // 32-bit code
const GDT_SEGMENT_DATA_32: u64 = 0x00cf_9300_0000_ffff; // 32-bit data
const GDT_SEGMENT_CODE_64: u64 = 0x00af_9b00_0000_ffff; // 64-bit code
const GDT_SEGMENT_DATA_64: u64 = 0x00af_9300_0000_ffff; // 64-bit data
const GDT_SEGMENT_USER_CODE_64: u64 = 0x00af_fb00_0000_ffff; // usermode 64-bit code
const GDT_SEGMENT_USER_DATA_64: u64 = 0x00af_f300_0000_ffff; // usermode 64-bit data

const GDT_LEN: usize = 9;

lazy_static! {
    pub static ref GLOBAL_DESCRIPTOR_TABLE: GlobalDescriptorTable =
        GlobalDescriptorTable::default();
}

pub struct GlobalDescriptorTable {
    segments: [GDTSegmentDescriptor; GDT_LEN],
    code_selector: u16,
    data_selector: u16,
    len: usize,
}

impl GlobalDescriptorTable {
    pub fn default() -> GlobalDescriptorTable {
        GlobalDescriptorTable {
            segments: [
                GDTSegmentDescriptor::new(GDT_SEGMENT_NULL),    // 0x0000
                GDTSegmentDescriptor::new(GDT_SEGMENT_CODE_16), //0x0008
                GDTSegmentDescriptor::new(GDT_SEGMENT_DATA_16), //0x0010
                GDTSegmentDescriptor::new(GDT_SEGMENT_CODE_32), //0x0018
                GDTSegmentDescriptor::new(GDT_SEGMENT_DATA_32), //0x0020
                GDTSegmentDescriptor::new(GDT_SEGMENT_CODE_64), //0x0028
                GDTSegmentDescriptor::new(GDT_SEGMENT_DATA_64), //0x0030
                GDTSegmentDescriptor::new(GDT_SEGMENT_USER_CODE_64), //0x0038
                GDTSegmentDescriptor::new(GDT_SEGMENT_USER_DATA_64), //0x0040
            ],
            len: 9,
            code_selector: 0x0028,
            data_selector: 0x0030,
        }
    }

    pub fn load(&'static self) {
        unsafe {
            lgdt(&GDT_Pointer {
                size: (GDT_LEN as u16 - 1) * 8,
                ptr: self.segments.as_ptr() as u64,
            });

            load_cs(self.code_selector);
            load_data_segment(self.data_selector);
        }
    }

    pub fn debug(&self) {
        println!("\x1b[36m------------------------------------- Global Descriptor Table --------------------------------------\x1b[0m");
        for segment in self.segments {
            println!("{:?}", segment);
        }
        println!("\x1b[36m----------------------------------------------------------------------------------------------------\x1b[0m")
    }

    pub unsafe fn assert_load() {
        let gdt_ptr = sgdt();
        let segment1_value: GDTSegmentDescriptor =
            GDTSegmentDescriptor::new(*(gdt_ptr.ptr as *const u64).add(1));
        assert_or_panic(GDT_SEGMENT_CODE_16 == segment1_value.0, "GDT Load")
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct GDTSegmentDescriptor(u64);

impl GDTSegmentDescriptor {
    pub fn new(raw: u64) -> Self {
        Self(raw)
    }
}

impl core::fmt::Debug for GDTSegmentDescriptor {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let limit: u32 = (self.0 & 0xFFFF) as u32 | ((self.0 & 0x000F_0000_0000_0000) >> 32) as u32;
        let base: u32 = ((self.0 & 0xFFFFF_0000) >> 16) as u32
            | ((self.0 & 0xF000_0000_0000_0000) >> 32) as u32;
        let access_byte: u8 = ((self.0 & 0xFF00_0000_0000) >> 40) as u8;
        let flags: u8 = ((self.0 & 0x00F0_0000_0000_0000) >> 52) as u8;

        fmt.debug_struct("GDTSegmentDescriptor")
            // ...
            .field("Limit", &format_args!("0x{:08X}", limit))
            .field("Base", &format_args!("0x{:08X}", base))
            .field("Access Byte", &format_args!("0b{:08b}", access_byte))
            .field("Flags", &format_args!("0b{:04b}", flags))
            // ...
            .finish()
    }
}

// TODO: implement debug that outputs the details of the gdt segment
