#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::arch::asm;

use limine::request::{RequestsEndMarker, RequestsStartMarker};
use limine::BaseRevision;
use x86_64::gdt::{GlobalDescriptorTable, GLOBAL_DESCRIPTOR_TABLE};
use x86_64::idt::{InterruptDescriptorTable, INTERRUPT_DESCRIPTOR_TABLE};
use x86_64::instructions::enable_interrupts;
use x86_64::memory::frame_allocator::{allocate_frame, init_allocator_page_tables};
use x86_64::pic::{assert_pic, init_pic};

mod framebuffer;
mod limine_utils;
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

    INTERRUPT_DESCRIPTOR_TABLE.load();
    InterruptDescriptorTable::assert_load();

    init_pic();
    enable_interrupts();
    assert_pic();

    // mmap_scratch();
    init_allocator_page_tables();
    let frame0 = allocate_frame();
    let frame1 = allocate_frame();
    let frame2 = allocate_frame();
    frame1.free();
    let frame3 = allocate_frame();

    debug!("{:?}", frame0);
    debug!("{:?}", frame1);
    debug!("{:?}", frame2);
    debug!("{:?}", frame3);

    frame2.free();

    debug!("Testing write 0xABAB_ABAB_ABAB_ABAB to frame 0 (page 0)");
    *(frame0.base_virtual_address as *mut u64) = 0xABAB_ABAB_ABAB_ABAB;
    debug!(
        "Testing read from frame 1: {:016X}",
        *(frame0.base_virtual_address as *mut u64)
    );

    debug!("Testing write 0xABAB_ABAB_ABAB_ABAB to frame 3 (page 1)");
    *(frame3.base_virtual_address as *mut u64) = 0xABAB_ABAB_ABAB_ABAB;
    debug!(
        "Testing read from frame 4: {:016X}",
        *(frame3.base_virtual_address as *mut u64)
    );

    debug!("Testing write to frame 2 (page 2) - should page fault because page freed");
    *(frame2.base_virtual_address as *mut u64) = 0;

    hcf();
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
