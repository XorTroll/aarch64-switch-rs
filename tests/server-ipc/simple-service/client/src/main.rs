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

use core::panic;

// Same interface as /server project
pub trait IDemoService {
    ipc_interface_define_command!(sample_cmd_1: (in_value: u32) => (out_value: u64));
}

pub struct DemoService {
    session: service::Session
}

impl service::ISessionObject for DemoService {
    fn new(session: service::Session) -> Self {
        Self { session: session }
    }
    
    fn get_session(&mut self) -> &mut service::Session {
        &mut self.session
    }
}

impl IDemoService for DemoService {
    fn sample_cmd_1(&mut self, in_value: u32) -> Result<u64> {
        ipc_client_send_request_command!([self.session.session; 123] (in_value) => (out_value: u64))
    }
}

impl service::IService for DemoService {
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
    let demosrv = service::new_service_object::<DemoService>()?;

    let test_command_with_value = |val: u32| {
        match demosrv.get().sample_cmd_1(val) {
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
        Err(rc) => diag_result_log_assert!(log::LmLogger, assert::AssertMode::FatalThrow => rc),
        _ => {}
    }

    Ok(())
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::on_panic_handler::<log::LmLogger>(info, assert::AssertMode::FatalThrow, results::lib::assert::ResultAssertionFailed::make())
}