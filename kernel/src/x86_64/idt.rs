use core::fmt::{self, Pointer};
use lazy_static::lazy_static;

use crate::{debug, info, println, x86_64::instructions::get_cs};

use super::{
    address::IDT_Pointer,
    instructions::{lidt, sidt},
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
enum GateType {
    Interrupt = 0b1110,
    Trap = 0b1111,
}

#[repr(u8)]
enum PrivilegeLevel {
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

    fn new(
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
            length: 0,
        }
    }

    fn add_descriptor(&mut self, descriptor: IDTDescriptor) -> () {
        self.descriptors[self.length] = descriptor;
        self.length += 1;
    }

    pub unsafe fn load(&self) {
        // debug!("Loading IDT");
        // debug!(
        //     "Descriptors 0 and 3: {:?} {:?}",
        //     self.descriptors[0], self.descriptors[3]
        // );
        // debug!("CS From REG: {:04X}", get_cs());

        lidt(&IDT_Pointer {
            size: (self.length as u16 - 1) * 16,
            ptr: self.descriptors.as_ptr() as u64,
        });
    }

    pub unsafe fn assert_load() {
        println!();
        debug!("Retrieving IDT from sidt...");
        let gdt_ptr = sidt();
        debug!("Descriptor From IDT");
        gdt_ptr.debug();

        let current_size = (gdt_ptr.size / 16) + 1;

        println!();
        // debug!("Loaded Descriptors:");
        // for index in 0..current_size {
        //     let descriptor = *((gdt_ptr.ptr as *const IDTDescriptor).add(index as usize));

        //     debug!("{:?}", descriptor)
        // }

        // let segment1_value: GDTSegmentDescriptor =
        //     GDTSegmentDescriptor::new(*(gdt_ptr.ptr as *const u64).add(1));
        // assert_or_panic(GDT_SEGMENT_CODE_16 == segment1_value.0, "GDT Load")
    }
}

#[repr(C)]
struct ExceptionStackFrame {
    instruction_pointer: u64,
    code_segment: u64,
    cpu_flags: u64,
    stack_pointer: u64,
    stack_segment: u64,
}

impl core::fmt::Debug for ExceptionStackFrame {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("Exception Stack Frame")
            .field(
                "\n    Instruction Pointer",
                &format_args!("0x{:016X}", self.instruction_pointer),
            )
            .field(
                "\n    Code Segment",
                &format_args!("0x{:016X}", self.code_segment),
            )
            .field(
                "\n    CPU Flags",
                &format_args!("0x{:016X}", self.cpu_flags),
            )
            .field(
                "\n    Stack Pointer",
                &format_args!("0x{:016X}", self.stack_pointer),
            )
            .field(
                "\n    Stack Segment",
                &format_args!("0x{:016X}\n", self.stack_segment),
            )
            .finish()
    }
}

extern "x86-interrupt" fn divide_by_zero_interrupt(stack_frame: ExceptionStackFrame) {
    debug!("BREAKPOINT INTERRUPT");
    debug!("{:?}", stack_frame)
}

extern "x86-interrupt" fn breakpoint_interrupt(stack_frame: ExceptionStackFrame) {
    debug!("BREAKPOINT INTERRUPT");
    debug!("{:?}", stack_frame)
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

        idt.add_descriptor(divide_by_zero_descriptor);
        idt.add_descriptor(IDTDescriptor::empty());
        idt.add_descriptor(IDTDescriptor::empty());
        idt.add_descriptor(breakpoint_descriptor);

        for _ in 0..MAX_IDT_SIZE - 4 {
            idt.add_descriptor(IDTDescriptor::empty());
        }

        idt
    };
}
