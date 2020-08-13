#![no_std]
#![no_main]

#[macro_use]
extern crate nx;

#[macro_use]
extern crate alloc;
use alloc::vec::Vec;

extern crate paste;

use nx::result::*;
use nx::results;
use nx::util;
use nx::diag::assert;
use nx::diag::log;
use nx::diag::log::Logger;
use nx::ipc::CommandParameterDeserialize;
use nx::ipc::server;

use core::panic;

pub trait IDemoService {
    ipc_server_interface_define_command!(sample_cmd_1: (in_value: u32) => (out_value: u64));
}

pub struct DemoService;

impl IDemoService for DemoService {
    fn sample_cmd_1(&mut self, in_value: u32) -> Result<u64> {
        diag_log!(log::LmLogger { log::LogSeverity::Error, true } => "Got value {}", in_value);
        if (in_value % 2) == 0 {
            Err(ResultCode::new(0xBABE))
        }
        else {
            Ok((in_value * in_value) as u64)
        }
    }
}

impl server::Server for DemoService {
    fn new() -> Self {
        Self {}
    }

    fn get_command_table(&self) -> Vec<server::CommandMetadata> {
        ipc_server_make_command_metadata!(
            sample_cmd_1: 123
        )
    }
}

impl server::Service for DemoService {
    fn get_name() -> &'static str {
        nul!("dmo-srv")
    }

    fn get_max_sesssions() -> i32 {
        0x40
    }
}

// We're using 128KB of heap
static mut STACK_HEAP: [u8; 0x20000] = [0; 0x20000];

#[no_mangle]
pub fn initialize_heap(_hbl_heap: util::PointerAndSize) -> util::PointerAndSize {
    unsafe {
        util::PointerAndSize::new(STACK_HEAP.as_mut_ptr(), STACK_HEAP.len())
    }
}

pub fn server_main() -> Result<()> {
    let mut manager = server::ServerManager::new();
    manager.register_service_server::<DemoService>()?;
    manager.loop_process();

    Ok(())
}

#[no_mangle]
pub fn main() -> Result<()> {
    match server_main() {
        Err(rc) => diag_log_result_assert!(log::LmLogger, assert::AssertMode::FatalThrow => rc),
        _ => {}
    }

    Ok(())
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::on_panic_handler::<log::LmLogger>(info, assert::AssertMode::FatalThrow, results::lib::assert::ResultAssertionFailed::make())
}