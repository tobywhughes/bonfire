use crate::{info, println};

pub fn assert_print(result: bool, label: &str) {
    if result {
        info!("{} - \x1b[32m[OK]\x1b[0m", label)
    } else {
        info!("{} - \x1b[31m[ERROR]\x1b[0m", label)
    }
}

pub fn assert_or_panic(result: bool, label: &str) {
    if result {
        info!("{} - \x1b[32m[OK]\x1b[0m", label)
    } else {
        info!("{} - \x1b[31m[ERROR]\x1b[0m", label);
        panic!("Critical Assertion Failed")
    }
}
