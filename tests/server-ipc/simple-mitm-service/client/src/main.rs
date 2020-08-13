#![no_std]
#![no_main]

#[macro_use]
extern crate nx;

#[macro_use]
extern crate alloc;

use nx::svc;
use nx::result::*;
use nx::results;
use nx::util;
use nx::diag::assert;
use nx::diag::log;
use nx::diag::log::Logger;
use nx::service;
use nx::service::SessionObject;

use core::panic;

pub trait IAccU0Service {
    fn get_user_count(&mut self) -> Result<u32>;
}

session_object_define!(AccU0Service);

impl IAccU0Service for AccU0Service {
    fn get_user_count(&mut self) -> Result<u32> {
        let out: u32;
        ipc_client_session_send_request_command!([self.session; 0; false] => {
            In {};
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {};
            Out {
                out: u32 => out
            };
            OutHandles {};
            OutObjects {};
            OutSessions {};
        });
        Ok(out)
    }
}

impl service::Service for AccU0Service {
    fn get_name() -> &'static str {
        nul!("acc:u0")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}

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

pub fn client_main() -> Result<()> {
    let mut accu0 = service::new_service_object::<AccU0Service>()?;

    let count = accu0.get_user_count()?;
    diag_log!(log::LmLogger { log::LogSeverity::Error, true } => "Got user count: {}", count);

    Ok(())
}

#[no_mangle]
pub fn main() -> Result<()> {
    match client_main() {
        Err(rc) => diag_log_result_assert!(log::LmLogger, assert::AssertMode::FatalThrow => rc),
        _ => {}
    }

    Ok(())
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::on_panic_handler::<log::LmLogger>(info, assert::AssertMode::FatalThrow, results::lib::assert::ResultAssertionFailed::make())
}