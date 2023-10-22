#![no_std]
#![no_main]
#![feature(panic_info_message)]

use core::arch::asm;

use framebuffer::FRAME_BUFFER;
use terminal::{TerminalWriter, TERMINAL_WRITER};

use crate::{
    sanity::{sanity_checks, RUN_SANITY, SANITY_ONLY},
    terminal::Color,
};

mod framebuffer;
mod psf;
mod sanity;
mod terminal;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Ensure we got a framebuffer.
    // Get the first framebuffer's information.
    sanity_checks();
    if RUN_SANITY && SANITY_ONLY {
        hcf();
    }

    welcome_message();

    hcf();
}

fn welcome_message() {
    TERMINAL_WRITER.lock().set_terminal_color(Color::Red);
    println!("                                            )   (     ");
    println!("    (              (                     ( /(   )\\ )  ");
    println!("  ( )\\             )\\ )  (   (      (    )\\()) (()/(  ");
    println!("  )((_)  (    (   (()/(  )\\  )(    ))\\  ((_)\\   /(_)) ");
    println!(" ((_)_   )\\   )\\ ) /(_))((_)(()\\  /((_)   ((_) (_))   ");
    println!("  | _ ) ((_) _(_/((_) _| (_) ((_)(_))    / _ \\ / __|  ");
    println!("  | _ \\/ _ \\| ' \\))|  _| | || '_|/ -_)  | (_) |\\__ \\  ");
    println!("  |___/\\___/|_||_| |_|   |_||_|  \\___|   \\___/ |___/  ");
    TERMINAL_WRITER.lock().set_terminal_color(Color::White);
    println!("\nInitializing OS...");
}

#[panic_handler]
fn rust_panic(_info: &core::panic::PanicInfo) -> ! {
    TERMINAL_WRITER.lock().clear_screen();
    println!("{}", _info);
    hcf();
}

fn hcf() -> ! {
    unsafe {
        asm!("cli");
        loop {
            asm!("hlt");
        }
    }
}
