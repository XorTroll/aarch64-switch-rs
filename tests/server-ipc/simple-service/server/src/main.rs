#![no_std]
#![no_main]

#[macro_use]
extern crate nx;

#[macro_use]
extern crate alloc;

extern crate paste;

use nx::result::*;
use nx::results;
use nx::util;
use nx::mem;
use nx::diag::assert;
use nx::diag::log;
use nx::ipc::sf;
use nx::ipc::server;

use core::panic;

pub trait IDemoSubInterface {
    ipc_interface_define_command!(sample_cmd_1: (input: u32) => (output: u32));
}

pub trait IDemoService {
    ipc_interface_define_command!(open_sub_interface: (value: u32, pid: sf::ProcessId) => (sub_interface: mem::Shared<dyn sf::IObject>));
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct SampleStruct {
    val1: u32,
    val2: u64,
    val3: bool,
    val4: char
}

pub struct DemoSubInterface {
    session: sf::Session,
    value: u32
}

impl DemoSubInterface {
    pub fn new(value: u32) -> Self {
        Self { session: sf::Session::new(), value: value }
    }
}

impl IDemoSubInterface for DemoSubInterface {
    fn sample_cmd_1(&mut self, input: u32) -> Result<u32> {
        Ok(input * self.value)
    }
}

impl sf::IObject for DemoSubInterface {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        ipc_server_make_command_table!(
            sample_cmd_1: 246
        )
    }
}

pub struct DemoService {
    session: sf::Session
}

impl IDemoService for DemoService {
    fn open_sub_interface(&mut self, value: u32, pid: sf::ProcessId) -> Result<mem::Shared<dyn sf::IObject>> {
        diag_log!(log::LmLogger { log::LogSeverity::Error, true } => "Opening interface (process ID 0x{:X}) with value {}", pid.process_id, value);
        Ok(mem::Shared::new(DemoSubInterface::new(value)))
    }
}

impl sf::IObject for DemoService {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        ipc_server_make_command_table!(
            open_sub_interface: 123
        )
    }
}

impl server::IServerObject for DemoService {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }
}

impl server::IService for DemoService {
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
    manager.loop_process()?;

    Ok(())
}

#[no_mangle]
pub fn main() -> Result<()> {
    match server_main() {
        Err(rc) => diag_result_log_assert!(log::LmLogger, assert::AssertMode::FatalThrow => rc),
        _ => {}
    }

    Ok(())
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::on_panic_handler::<log::LmLogger>(info, assert::AssertMode::FatalThrow, results::lib::assert::ResultAssertionFailed::make())
}