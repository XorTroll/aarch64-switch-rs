#![no_std]
#![no_main]

#[macro_use]
extern crate nx;

#[macro_use]
extern crate alloc;

use nx::svc;
use nx::result::*;
use nx::util;
use nx::diag::assert;
use nx::diag::log;
use nx::diag::log::Logger;
use nx::thread;

use core::panic;

#[no_mangle]
pub fn initialize_heap(hbl_heap: util::PointerAndSize) -> util::PointerAndSize {
    if hbl_heap.is_valid() {
        hbl_heap
    }
    else {
        let heap_size: usize = 0x10000000;
        let heap_address = svc::set_heap_size(heap_size).unwrap();
        util::PointerAndSize::new(heap_address, heap_size)
    }
}

pub fn spam_fn(_: *mut u8) {
    for i in 0..20 {
        diag_log!(log::LmLogger { log::LogSeverity::Trace, true } => "Test log {}", i);
    }
}

pub fn threading_test() -> Result<()> {
    let mut thread = thread::Thread::new(spam_fn, core::ptr::null_mut(), core::ptr::null_mut(), 0x2000, "ThreadNo2")?;
    thread.create_and_start(thread::INVALID_PRIORITY, -2)?;

    spam_fn(core::ptr::null_mut());

    thread.join()?;

    Ok(())
}

#[no_mangle]
pub fn main() -> Result<()> {
    if let Err(rc) = threading_test() {
        assert::assert(assert::AssertMode::SvcBreak, rc);
    }

    Ok(())
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::on_panic_handler::<log::LmLogger>(info, assert::AssertMode::SvcBreak, ResultCode::from::<assert::ResultAssertionFailed>())
}