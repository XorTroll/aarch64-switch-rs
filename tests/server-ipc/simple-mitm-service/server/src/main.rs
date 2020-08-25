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
use nx::service::sm;
use nx::ipc::sf;
use nx::ipc::server;

use core::panic;

pub trait IAccountServiceForApplication {
    ipc_interface_define_command!(get_user_count: () => (out_value: u32));
}

pub struct AccountServiceForApplication {
    session: sf::Session
}

impl sf::IObject for AccountServiceForApplication {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }
    
    fn get_command_table(&self) -> sf::CommandMetadataTable {
        ipc_server_make_command_table!(
            get_user_count: 0
        )
    }
}

impl IAccountServiceForApplication for AccountServiceForApplication {
    fn get_user_count(&mut self) -> Result<u32> {
        let stub: u32 = 69;
        diag_log!(log::LmLogger { log::LogSeverity::Error, true } => "acc:u0 mitm accessed! returning {} as stubbed value...", stub);
        Ok(stub)
    }
}

impl server::IServerObject for AccountServiceForApplication {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }
}

impl server::IMitmService for AccountServiceForApplication {
    fn get_name() -> &'static str {
        nul!("acc:u0")
    }

    fn should_mitm(_info: sm::MitmProcessInfo) -> bool {
        true
    }
}

// We're using 128KB of heap
static mut STACK_HEAP: [u8; 0x60000] = [0; 0x60000];

#[no_mangle]
pub fn initialize_heap(_hbl_heap: util::PointerAndSize) -> util::PointerAndSize {
    unsafe {
        util::PointerAndSize::new(STACK_HEAP.as_mut_ptr(), STACK_HEAP.len())
    }
}

pub fn server_main() -> Result<()> {
    let mut manager = server::ServerManager::new();
    manager.register_mitm_service_server::<AccountServiceForApplication>()?;
    manager.loop_process()?;

    Ok(())
}

#[no_mangle]
pub fn main() -> Result<()> {
    match server_main() {
        Err(rc) => assert::assert(assert::AssertMode::FatalThrow, rc),
        _ => {}
    }

    Ok(())
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::on_panic_handler::<log::LmLogger>(info, assert::AssertMode::FatalThrow, results::lib::assert::ResultAssertionFailed::make())
}