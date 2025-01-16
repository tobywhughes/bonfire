use core::fmt;

pub mod irq;

#[repr(C)]
pub struct ExceptionStackFrame {
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
