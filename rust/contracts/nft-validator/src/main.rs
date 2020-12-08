#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]

use ckb_std::{default_alloc, entry};

entry!(entry);
default_alloc!();

mod validator;

/// Program entry
fn entry() -> i8 {
    // Call main function and return error code
    match validator::validate() {
        Ok(_) => 0,
        Err(err) => err as i8,
    }
}
