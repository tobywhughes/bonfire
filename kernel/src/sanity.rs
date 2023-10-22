use crate::println;

pub const RUN_SANITY: bool = true;
pub const SANITY_ONLY: bool = true;

const PRINT_SANITY: bool = true;
const WRAP_SANITY: bool = true;
const SCROLL_SANITY: bool = false;
const PANIC_SANITY: bool = false;

fn print_sanity() {
    println!("Print working: {}", true);
}

fn wrap_sanity() {
    println!("Should wrap................................................................................................................................ Am I wrapped?");
}

fn scroll_sanity() {
    for i in 0..50 {
        println!("scrolling: {}", i);
    }
    println!("Scrolled if visible");
}

fn panic_sanity() {
    panic!("Should see this panic message");
}

pub fn sanity_checks() {
    if !RUN_SANITY {
        return;
    }

    if PRINT_SANITY {
        print_sanity();
    }
    if WRAP_SANITY {
        wrap_sanity();
    }
    if SCROLL_SANITY {
        scroll_sanity();
    }
    if PANIC_SANITY {
        panic_sanity();
    }
}
