use core::fmt::{self, Pointer};
use lazy_static::lazy_static;

use crate::{
    debug, info, print, println,
    utils::assert::assert_or_panic,
    x86_64::{
        instructions::{get_cs, int3},
        interrupts::irq::get_keyboard_idt_descriptor_and_index,
        pic::pic_1_eoi,
    },
};

use super::{
    address::IDT_Pointer,
    instructions::{lidt, sidt},
    interrupts::ExceptionStackFrame,
    pic::TIMER_ASSERT_CALLED,
};

#[derive(Copy, Clone)]
#[repr(C)]
pub struct IDTDescriptor {
    offset_low: u16,
    segment_selector: u16,
    ist_offset: u8,
    options: u8,
    offset_mid: u16,
    offset_high: u32,
    reserved: u32,
}

#[repr(u8)]
pub enum GateType {
    Interrupt = 0b1110,
    Trap = 0b1111,
}

#[repr(u8)]
pub enum PrivilegeLevel {
    Kernel = 0b00,
    User = 0b11,
}

impl IDTDescriptor {
    fn empty() -> IDTDescriptor {
        IDTDescriptor {
            offset_low: 0,
            segment_selector: 0,
            ist_offset: 0,
            options: 0b000_1110,
            offset_mid: 0,
            offset_high: 0,
            reserved: 0,
        }
    }

    pub fn new(
        offset: u64,
        segment_selector: u16,
        ist_offset: u8,
        gate_type: GateType,
        privilege_level: PrivilegeLevel,
        present: bool,
    ) -> IDTDescriptor {
        let (offset_low, offset_mid, offset_high) = IDTDescriptor::offset_parts_from_u64(offset);
        let options = IDTDescriptor::build_options(gate_type, privilege_level, present);

        IDTDescriptor {
            offset_low,
            segment_selector,
            ist_offset: ist_offset & 0x07,
            options,
            offset_mid,
            offset_high,
            reserved: 0,
        }
    }

    fn offset_parts_from_u64(offset: u64) -> (u16, u16, u32) {
        let offset_low: u16 = (offset & 0xFFFF) as u16;
        let offset_mid: u16 = ((offset >> 16) & 0xFFFF) as u16;
        let offset_high: u32 = ((offset >> 32) & 0xFFFF_FFFF) as u32;

        (offset_low, offset_mid, offset_high)
    }

    fn build_options(gate_type: GateType, privilege_level: PrivilegeLevel, present: bool) -> u8 {
        let mut options: u8 = 0x00;

        options |= gate_type as u8;
        options |= (privilege_level as u8) << 5;

        if present {
            options |= 0b1000_0000;
        }

        options
    }
}

impl core::fmt::Debug for IDTDescriptor {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("IDTdescriptor")
            .field("Offset High", &format_args!("0x{:08X}", self.offset_high))
            .field("Offset Mid", &format_args!("0x{:04X}", self.offset_mid))
            .field("Offset LOW", &format_args!("0x{:04X}", self.offset_low))
            .field(
                "Segment Selector",
                &format_args!("0x{:04X}", self.segment_selector),
            )
            .field("IST Offset", &format_args!("0b{:08b}", self.ist_offset))
            .field("Options", &format_args!("0b{:08b}", self.options))
            .field("Reserved", &format_args!("0x{:08X}", self.reserved))
            .finish()
    }
}

const MAX_IDT_SIZE: usize = 256;

#[repr(align(16))]
pub struct InterruptDescriptorTable {
    descriptors: [IDTDescriptor; MAX_IDT_SIZE],
    length: usize,
}

impl InterruptDescriptorTable {
    fn default() -> InterruptDescriptorTable {
        InterruptDescriptorTable {
            descriptors: [IDTDescriptor::empty(); MAX_IDT_SIZE],
            length: MAX_IDT_SIZE,
        }
    }

    fn set_descriptor(&mut self, descriptor: IDTDescriptor, index: usize) -> () {
        self.descriptors[index] = descriptor;
    }

    pub unsafe fn load(&self) {
        // debug!("Loading IDT");
        // debug!(
        //     "Descriptors 0 and 3: {:?} {:?}",
        //     self.descriptors[0], self.descriptors[3]
        // );
        // debug!("CS From REG: {:04X}", get_cs());

        lidt(&self.create_idt_pointer());
    }

    pub fn create_idt_pointer(&self) -> IDT_Pointer {
        IDT_Pointer {
            size: (self.length as u16 - 1) * 16,
            ptr: self.descriptors.as_ptr() as u64,
        }
    }

    pub unsafe fn assert_load() {
        let loaded_idt_ptr = sidt();

        let mut assert_passed = true;
        // Loaded GDT GDT PTR Matches

        assert_passed &= loaded_idt_ptr == INTERRUPT_DESCRIPTOR_TABLE.create_idt_pointer();

        // Breakpoint called
        int3();
        assert_passed &= BREAKPOINT_ASSERT_CALLED;

        assert_or_panic(assert_passed, "IDT Load");

        // println!();
    }
}

extern "x86-interrupt" fn divide_by_zero_interrupt(stack_frame: ExceptionStackFrame) {
    debug!("BREAKPOINT INTERRUPT");
    debug!("{:?}", stack_frame);
}

static mut BREAKPOINT_ASSERT_CALLED: bool = false;

extern "x86-interrupt" fn breakpoint_interrupt(stack_frame: ExceptionStackFrame) {
    debug!("BREAKPOINT INTERRUPT");
    unsafe {
        BREAKPOINT_ASSERT_CALLED = true;
    }
    debug!("{:?}", stack_frame);
}

extern "x86-interrupt" fn timer_irq(stack_frame: ExceptionStackFrame) {
    // debug!("TIMER INTERRUPT");
    // debug!("{:?}", stack_frame);
    // print!(".");
    unsafe {
        TIMER_ASSERT_CALLED = true;
    }

    pic_1_eoi();
}

extern "x86-interrupt" fn double_fault_exception(
    stack_frame: ExceptionStackFrame,
    error_code: u64,
) {
    debug!("DOBULE FAULT INTERRUPT");
    debug!("{:?}", stack_frame);
    debug! {"Error Code: {:016X}", error_code}
    panic!("DOUBLE FAULT WITH ERROR CODE [{:016X}]", error_code)
}

lazy_static! {
    pub static ref INTERRUPT_DESCRIPTOR_TABLE: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::default();

        let divide_by_zero_descriptor = IDTDescriptor::new(
            divide_by_zero_interrupt as u64,
            0x28,
            0,
            GateType::Interrupt,
            PrivilegeLevel::Kernel,
            true,
        );

        let breakpoint_descriptor = IDTDescriptor::new(
            breakpoint_interrupt as u64,
            0x28,
            0,
            GateType::Interrupt,
            PrivilegeLevel::Kernel,
            true,
        );

        let double_fault_descriptor = IDTDescriptor::new(
            double_fault_exception as u64,
            0x28,
            0,
            GateType::Trap,
            PrivilegeLevel::Kernel,
            true,
        );

        let timer_descriptor = IDTDescriptor::new(
            timer_irq as u64,
            0x28,
            0,
            GateType::Interrupt,
            PrivilegeLevel::Kernel,
            true,
        );

        let (keyboard_descriptor, keyboard_index) = get_keyboard_idt_descriptor_and_index();

        idt.set_descriptor(divide_by_zero_descriptor, 0x00);
        idt.set_descriptor(breakpoint_descriptor, 0x03);
        idt.set_descriptor(double_fault_descriptor, 0x08);
        idt.set_descriptor(timer_descriptor, 0x20);
        idt.set_descriptor(keyboard_descriptor, keyboard_index);

        idt
    };
}
