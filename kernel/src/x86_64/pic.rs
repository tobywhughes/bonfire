use crate::{
    debug, println, utils::assert::assert_or_panic, x86_64::instructions::write_u8_to_port,
};

use super::instructions::read_u8_from_port;

const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_COMMAND: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

const PIC_EOI_COMMAND: u8 = 0x20;
const ICW_1_COMMAND: u8 = 0x11;
const ICW_2_PIC1_VECTOR: u8 = 0x20;
const ICW_2_PIC2_VECTOR: u8 = 0x28;

const ICW_3_PIC1_IRQ: u8 = 0x04;
const ICW_3_PIC2_IRQ: u8 = 0x02;

const ICW_4_COMMAND: u8 = 0x01;

pub unsafe fn init_pic() {
    write_u8_to_port(PIC1_COMMAND, ICW_1_COMMAND);
    write_u8_to_port(PIC2_COMMAND, ICW_1_COMMAND);

    write_u8_to_port(PIC1_DATA, ICW_2_PIC1_VECTOR);
    write_u8_to_port(PIC1_DATA, ICW_2_PIC2_VECTOR);

    write_u8_to_port(PIC1_DATA, ICW_3_PIC1_IRQ);
    write_u8_to_port(PIC1_DATA, ICW_3_PIC2_IRQ);

    write_u8_to_port(PIC1_DATA, ICW_4_COMMAND);
    write_u8_to_port(PIC1_DATA, ICW_4_COMMAND);

    // Removes masks??
    write_u8_to_port(PIC1_DATA, 0x00);
    write_u8_to_port(PIC2_DATA, 0x00);
}

pub static mut TIMER_ASSERT_CALLED: bool = false;

// Simple test that timer interrupt has been called at least once
// Will probably need something more complex as timer interrupt actually gets used but good enough for today
pub unsafe fn assert_pic() {
    TIMER_ASSERT_CALLED = false;

    while !TIMER_ASSERT_CALLED {}

    assert_or_panic(TIMER_ASSERT_CALLED, "PIC Initialized");
}

pub fn pic_1_eoi() {
    unsafe {
        write_u8_to_port(PIC1_COMMAND, PIC_EOI_COMMAND);
    }
}

pub fn pic_2_eoi() {
    unsafe {
        write_u8_to_port(PIC2_COMMAND, PIC_EOI_COMMAND);
        write_u8_to_port(PIC1_COMMAND, PIC_EOI_COMMAND);
    }
}
