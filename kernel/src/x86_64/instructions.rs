use core::arch::asm;

use super::address::GDT_Pointer;

#[inline]
pub fn sgdt() -> GDT_Pointer {
    let mut gdt_ptr = GDT_Pointer { size: 0, ptr: 0 };

    unsafe {
        asm!("sgdt [{}]", in(reg) &mut gdt_ptr, options(nostack, preserves_flags));
    }

    gdt_ptr.debug();

    gdt_ptr
}

#[inline]
pub unsafe fn lgdt(gdt: &GDT_Pointer) {
    unsafe {
        asm!("lgdt [{}]", in(reg) gdt, options(readonly, nostack, preserves_flags));
    }
}
