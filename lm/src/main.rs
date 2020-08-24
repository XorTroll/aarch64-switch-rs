#![no_std]
#![no_main]

#![feature(global_asm)]

#[macro_use]
extern crate nx;

#[macro_use]
extern crate alloc;

extern crate paste;

use nx::result::*;
use nx::util;
use nx::wait;
use nx::thread;
use nx::diag::assert;
use nx::ipc::sf;
use nx::ipc::server;
use nx::service;
use nx::service::psc;
use nx::service::psc::IPmModule;
use nx::service::psc::IPmService;
use core::panic;

mod ipc;
mod logger;

static mut STACK_HEAP: [u8; 0x20000] = [0; 0x20000];

#[no_mangle]
pub fn initialize_heap(_hbl_heap: util::PointerAndSize) -> util::PointerAndSize {
    unsafe {
        util::PointerAndSize::new(STACK_HEAP.as_mut_ptr(), STACK_HEAP.len())
    }
}

#[allow(unreachable_code)]
pub fn pm_module_main() -> Result<()> {
    thread::get_current_thread().set_name("rust.lm.PmModule")?;

    let psc = service::new_service_object::<psc::PmService>()?;
    let module = psc.get().get_pm_module()?.to::<psc::PmModule>();

    let event_handle = module.get().initialize(psc::ModuleId::Lm, sf::Buffer::new())?;
    loop {
        wait::wait_handles(&[event_handle.handle], -1)?;

        let (state, _flags) = module.get().get_request()?;
        match state {
            psc::State::Awake | psc::State::ReadyAwaken | psc::State::ReadyAwakenCritical => logger::set_log_enabled(true),
            _ => logger::set_log_enabled(false)
        };

        module.get().acknowledge_ex(state)?;
    }

    Ok(())
}

pub fn pm_module_thread_fn(_: *mut u8) {
    match pm_module_main() {
        Err(rc) => assert::assert(assert::AssertMode::FatalThrow, rc),
        _ => {}
    }
}

pub fn server_main() -> Result<()> {
    thread::get_current_thread().set_name("rust.lm.IpcServer")?;

    logger::initialize()?;

    let mut pm_module_thread = thread::Thread::new(pm_module_thread_fn, core::ptr::null_mut(), core::ptr::null_mut(), 0x2000, "rust.lm.PmModuleThread")?;
    pm_module_thread.create_and_start(38, -2)?;

    let mut manager = server::ServerManager::new();
    manager.register_service_server::<ipc::LogService>()?;
    manager.loop_process()?;

    pm_module_thread.join()?;
    logger::exit();
    Ok(())
}

#[no_mangle]
pub fn main() -> Result<()> {
    server_main().unwrap();

    Ok(())
}

#[panic_handler]
fn panic_handler(_info: &panic::PanicInfo) -> ! {
    // TODO
    loop {}
    // util::on_panic_handler::<nx::diag::log::LmLogger>(info, assert::AssertMode::FatalThrow, results::lib::assert::ResultAssertionFailed::make())
}