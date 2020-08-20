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
use nx::mem;
use nx::diag::assert;
use nx::diag::log;
use nx::ipc::sf;
use nx::service;

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
    session: sf::Session
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

impl service::IClientObject for DemoSubInterface {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }
}

impl IDemoSubInterface for DemoSubInterface {
    fn sample_cmd_1(&mut self, input: u32) -> Result<u32> {
        ipc_client_send_request_command!([self.session.object_info; 246] (input) => (output: u32))
    }
}

pub struct DemoService {
    session: sf::Session
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

impl service::IClientObject for DemoService {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }
}

impl IDemoService for DemoService {
    fn open_sub_interface(&mut self, value: u32, pid: sf::ProcessId) -> Result<mem::Shared<dyn sf::IObject>> {
        ipc_client_send_request_command!([self.session.object_info; 123] (value, pid) => (sub_interface: mem::Shared<DemoSubInterface>))
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

    let a: u32 = 15;
    let b: u32 = 21;

    let subintf = demosrv.get().open_sub_interface(a, sf::ProcessId::new())?.to::<DemoSubInterface>();
    let output = subintf.get().sample_cmd_1(b)?;

    diag_log!(log::LmLogger { log::LogSeverity::Trace, false } => "Result: {} times {} = {}", a, b, output);

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