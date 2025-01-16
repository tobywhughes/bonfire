use core::arch::asm;

use crate::println;

use super::address::{GDT_Pointer, IDT_Pointer};

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

#[inline]
pub unsafe fn lidt(idt: &IDT_Pointer) {
    unsafe {
        asm!("lidt [{}]", in(reg) idt, options(readonly, nostack, preserves_flags));
    }
}

#[inline]
pub fn sidt() -> IDT_Pointer {
    let mut idt_ptr = IDT_Pointer { size: 0, ptr: 0 };

    unsafe {
        asm!("sidt [{}]", in(reg) &mut idt_ptr, options(nostack, preserves_flags));
    }
    idt_ptr
}

#[inline]
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

#[inline]
pub unsafe fn get_cs() -> u16 {
    let segment: u16;

    asm!("mov {0:x}, cs", out(reg) segment, options(nomem, nostack, preserves_flags));

    segment
}

#[inline]
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

#[inline]
pub unsafe fn int3() {
    asm!("int3", options(nomem, nostack));
}

#[inline]
pub unsafe fn write_u8_to_port(port: u16, value: u8) {
    asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
}

#[inline]
pub unsafe fn read_u8_from_port(port: u16) -> u8 {
    let value: u8;
    asm!("in al, dx", out("al") value, in("dx") port, options(nomem, nostack, preserves_flags));
    value
}

#[inline]
pub fn enable_interrupts() {
    unsafe {
        asm!("sti", options(preserves_flags, nostack));
    }
}

#[inline]
pub fn disable_interrupts() {
    unsafe {
        asm!("cli", options(preserves_flags, nostack));
    }
}
