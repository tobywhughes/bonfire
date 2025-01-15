#![no_std]
#![no_main]

use core::arch::asm;

use framebuffer::FrameBuffer;
use limine::request::{RequestsEndMarker, RequestsStartMarker};
use limine::BaseRevision;
use terminal::{Terminal, TERMINAL};
use x86_64::gdt::{GDTSegmentDescriptor, GlobalDescriptorTable, GLOBAL_DESCRIPTOR_TABLE};
use x86_64::instructions::sgdt;

mod framebuffer;
mod psf;
mod terminal;
mod utils;
mod x86_64;

/// Sets the base revision to the latest revision supported by the crate.
/// See specification for further info.
/// Be sure to mark all limine requests with #[used], otherwise they may be removed by the compiler.
#[used]
// The .requests section allows limine to find the requests faster and more safely.
#[link_section = ".requests"]
static BASE_REVISION: BaseRevision = BaseRevision::new();

/// Define the stand and end markers for Limine requests.
#[used]
#[link_section = ".requests_start_marker"]
static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();
#[used]
#[link_section = ".requests_end_marker"]
static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();

#[no_mangle]
unsafe extern "C" fn kmain() -> ! {
    // All limine requests must also be referenced in a called function, otherwise they may be
    // removed by the linker.
    assert!(BASE_REVISION.is_supported());

    println!("\x1b[31mBonfire OS\x1b[0m");

    GLOBAL_DESCRIPTOR_TABLE.load();
    GlobalDescriptorTable::assert_load();
    // GLOBAL_DESCRIPTOR_TABLE.debug();

    hcf();
}

fn test_panic() {
    panic!("This is a panic test.")
}

#[panic_handler]
fn rust_panic(_info: &core::panic::PanicInfo) -> ! {
    println!("{}", _info);
    hcf();
}

fn hcf() -> ! {
    loop {
        unsafe {
            #[cfg(target_arch = "x86_64")]
            asm!("hlt");
            #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
            asm!("wfi");
            #[cfg(target_arch = "loongarch64")]
            asm!("idle 0");
        }
    }
}
