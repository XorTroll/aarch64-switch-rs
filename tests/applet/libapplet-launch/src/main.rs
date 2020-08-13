#![no_std]
#![no_main]

#[macro_use]
extern crate nx;

#[macro_use]
extern crate alloc;

use nx::svc;
use nx::arm;
use nx::result::*;
use nx::results;
use nx::util;
use nx::diag::assert;
use nx::diag::log;
use nx::service;
use nx::service::applet;
use nx::service::applet::IAllSystemAppletProxiesService;
use nx::service::applet::ILibraryAppletProxy;
use nx::service::applet::ILibraryAppletCreator;
use nx::service::applet::ILibraryAppletAccessor;
use nx::service::applet::IStorage;
use nx::service::applet::IStorageAccessor;

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

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CommonArguments {
    version: u32,
    size: u32,
    la_api_version: u32,
    theme_color: u32,
    play_startup_sound: bool,
    pad: [u8; 7],
    system_tick: u64
}

pub fn applet_test() -> Result<()> {
    let mut applet_proxy_srv = service::new_service_object::<applet::AllSystemAppletProxiesService>()?;
    let attr: applet::AppletAttribute = unsafe { core::mem::zeroed() };
    let mut lib_applet_proxy: applet::LibraryAppletProxy = applet_proxy_srv.open_library_applet_proxy(attr)?;
    let mut lib_applet_creator: applet::LibraryAppletCreator = lib_applet_proxy.get_library_applet_creator()?;
    let mut lib_applet_accessor: applet::LibraryAppletAccessor = lib_applet_creator.create_library_applet(applet::AppletId::PlayerSelect, applet::LibraryAppletMode::AllForeground)?;

    {
        let storage_size = core::mem::size_of::<CommonArguments>();
        let common_args = CommonArguments { version: 1, size: storage_size as u32, la_api_version: 0x20000, theme_color: 0, play_startup_sound: false, pad: [0; 7], system_tick: arm::get_system_tick() };
        let mut storage: applet::Storage = lib_applet_creator.create_storage(core::mem::size_of::<CommonArguments>())?;
        {
            let mut storage_accessor: applet::StorageAccessor = storage.open()?;
            storage_accessor.write(0, &common_args as *const _ as *const u8, storage_size)?;
        }
        lib_applet_accessor.push_in_data(&storage)?;
    }

    {
        let mut data: [u8; 0xA0] = [0; 0xA0];
        data[0x96] = 1;
        let mut storage: applet::Storage = lib_applet_creator.create_storage(0xA0)?;
        {
            let mut storage_accessor: applet::StorageAccessor = storage.open()?;
            storage_accessor.write(0, &data as *const _ as *const u8, 0xA0)?;
        }
        lib_applet_accessor.push_in_data(&storage)?;
    }

    let event_handle = lib_applet_accessor.get_applet_state_changed_event()?;
    lib_applet_accessor.start()?;
    svc::wait_synchronization(&event_handle, 1, -1)?;

    svc::close_handle(event_handle)?;

    Ok(())
}

#[no_mangle]
pub fn main() -> Result<()> {
    if let Err(rc) = applet_test() {
        assert::assert(assert::AssertMode::FatalThrow, rc);
    }

    Ok(())
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::on_panic_handler::<log::LmLogger>(info, assert::AssertMode::FatalThrow, results::lib::assert::ResultAssertionFailed::make())
}