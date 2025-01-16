use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use spin::Mutex;

use crate::{
    print,
    x86_64::{
        idt::{GateType, IDTDescriptor, PrivilegeLevel},
        instructions::read_u8_from_port,
        pic::pic_1_eoi,
    },
};

use super::ExceptionStackFrame;

lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
        Mutex::new(Keyboard::new(
            ScancodeSet1::new(),
            layouts::Us104Key,
            HandleControl::Ignore
        ));
}

pub extern "x86-interrupt" fn keyboard_irq(stack_frame: ExceptionStackFrame) {
    // debug!("TIMER INTERRUPT");
    // debug!("{:?}", stack_frame);
    let scancode = unsafe { read_u8_from_port(0x0060) };
    let mut keyboard = KEYBOARD.lock();

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                _ => (),
            }
        }
    }

    pic_1_eoi();
}

pub fn get_keyboard_idt_descriptor_and_index() -> (IDTDescriptor, usize) {
    (
        IDTDescriptor::new(
            keyboard_irq as u64,
            0x28,
            0,
            GateType::Interrupt,
            PrivilegeLevel::Kernel,
            true,
        ),
        0x21,
    )
}
