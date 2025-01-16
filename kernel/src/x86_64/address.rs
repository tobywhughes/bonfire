use crate::debug;

#[derive(Debug)]
#[repr(packed(2))]
pub struct GDT_Pointer {
    pub size: u16,
    pub ptr: u64,
}

impl GDT_Pointer {
    pub fn debug(&self) {
        let ptr_value: u64 = unsafe {
            core::ptr::read_unaligned((self as *const _ as *const u16).add(1) as *const u64)
        };

        debug!("GDT PTR - 0x{:04X} 0x{:016X}", self.size, ptr_value);
    }
}

#[derive(Debug, PartialEq)]
#[repr(packed(2))]
pub struct IDT_Pointer {
    pub size: u16,
    pub ptr: u64,
}

impl IDT_Pointer {
    pub fn debug(&self) {
        let ptr_value: u64 = unsafe {
            core::ptr::read_unaligned((self as *const _ as *const u16).add(1) as *const u64)
        };

        debug!("IDT PTR - 0x{:04X} 0x{:016X}", self.size, ptr_value);
    }
}
