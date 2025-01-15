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

pub unsafe fn load_cs(selector: u16) {
    asm!(
        "push {selector_value}",
        "lea {lea_reg}, [2f + rip]",
        "push {lea_reg}",
        "retfq",
        "2:",
        selector_value = in(reg) selector as u64,
        lea_reg = lateout(reg) _,
        options(preserves_flags)
    );
}

pub unsafe fn load_data_segment(selector: u16) {
    asm!(
        "mov ds, {selector_value}",
        "mov es, {selector_value}", //The osdev docs seem to imply we load the ds segment into all of these, not 100% sure
        "mov fs, {selector_value}",
        "mov gs, {selector_value}",
        "mov ss, {selector_value}",
        selector_value = in(reg) selector as u64,
        options(nostack, preserves_flags)
    );
}
