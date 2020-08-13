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

pub trait IDemoService {
    fn sample_cmd_1(&mut self, in_value: u32) -> Result<u64>;
}

session_object_define!(DemoService);

impl IDemoService for DemoService {
    fn sample_cmd_1(&mut self, in_value: u32) -> Result<u64> {
        let out: u64;
        ipc_client_session_send_request_command!([self.session; 123; false] => {
            In {
                in_value: u32 = in_value
            };
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {};
            Out {
                out: u64 => out
            };
            OutHandles {};
            OutObjects {};
            OutSessions {};
        });
        Ok(out)
    }
}

impl service::Service for DemoService {
    fn get_name() -> &'static str {
        nul!("dmo-srv")
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
    let mut demosrv = service::new_service_object::<DemoService>()?;

    let mut test_command_with_value = |val: u32| {
        match demosrv.sample_cmd_1(val) {
            Ok(value) => diag_log!(log::LmLogger { log::LogSeverity::Error, true } => "Out value for {}: {}", val, value),
            Err(rc) => diag_log!(log::LmLogger { log::LogSeverity::Error, true } => "Error: {0} - {0:?}", rc),
        };
    };

    test_command_with_value(2);
    test_command_with_value(5);

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