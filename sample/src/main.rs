#![no_std]
#![no_main]

#[macro_use]
extern crate alloc;

use core::panic;
use nx::svc;
use nx::result::*;
use nx::util;
use nx::thread;
use nx::crt0;
use nx::service;
use nx::service::SessionObject;
use nx::service::fspsrv;
use nx::service::fspsrv::IFileSystemProxy;
use nx::service::fspsrv::IFileSystem;

macro_rules! log_debug_fmt {
    ($msg:literal) => {
        svc::output_debug_string($msg.as_ptr(), $msg.len()).unwrap();
    };
    ($fmt:literal, $( $param:expr ),*) => {
        let log_str = format!($fmt, $( $param ),*);
        svc::output_debug_string(log_str.as_ptr(), log_str.len()).unwrap();
    };
}

#[no_mangle]
pub fn initialize_heap(hbl_heap: util::PointerAndSize) -> util::PointerAndSize {
    if hbl_heap.is_valid() {
        hbl_heap
    }
    else {
        let heap_size: usize = 0x1000000;
        let heap_address = svc::set_heap_size(heap_size).unwrap();
        util::PointerAndSize::new(heap_address, heap_size)
    }
}

#[no_mangle]
pub fn main() -> Result<()> {
    log_debug_fmt!("Hello from Rust and from thread '{}'!", thread::get_current_thread().get_name()?);

    // Not using shared objects here since this is a quick test and the objects are only used inside this function
    let mut fspsrv = service::new_service_object::<fspsrv::FileSystemProxy>()?;
    log_debug_fmt!("Accessed fsp-srv service: {:?}", fspsrv.get_session());

    let mut sd_fs = fspsrv.open_sd_card_filesystem::<fspsrv::FileSystem>()?;
    log_debug_fmt!("Opened SD filesystem: {:?}", sd_fs.get_session());

    let path = "/sample_dir";
    sd_fs.create_directory(path.as_ptr(), path.len())?;
    log_debug_fmt!("Directory created");
    
    log_debug_fmt!("Test succeeded!");
    Ok(())
}

#[panic_handler]
fn on_panic(info: &panic::PanicInfo) -> ! {
    let thread_name = match thread::get_current_thread().get_name() {
        Ok(name) => name,
        _ => "<unknown>",
    };
    log_debug_fmt!("Panic! at thread '{}' -> {}", thread_name, info);

    // TODO: assertion system...?
    crt0::exit(ResultCode::new(0xBABE))
}