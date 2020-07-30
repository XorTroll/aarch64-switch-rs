#![no_std]
#![no_main]
#![feature(const_generics)]

#[macro_use]
extern crate alloc;

use core::panic;
use nx::svc;
use nx::util;
use nx::thread;

macro_rules! log_debug_fmt {
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

/*
macro_rules! module_name {
    ($name:literal) => {
        pub struct ModuleName {
            reserved: u32,
            length: u32,
            name: [u8; ($name).len()],
        }
        
        #[link_section = ".module_name"]
        #[used]
        pub static MODULE_NAME: ModuleName = ModuleName { reserved: 0, length: ($name).len() as u32, name: *$name };
    };
}

module_name!(b"Demo");
*/

macro_rules! set_module_name {
    ($lit:literal) => {
        const __INTERNAL_MODULE_LEN: usize = $lit.len();
        #[link_section = ".module_name"]
        pub static __MODULE_NAME: ModuleName<__INTERNAL_MODULE_LEN> =
            ModuleName::new($lit);
    };
}

#[repr(packed)]
#[allow(unused_variables)]
pub struct ModuleName<const LEN: usize> {
    pub unk: u32,
    pub name_length: u32,
    pub name: [u8; LEN],
}

impl<const LEN: usize> ModuleName<LEN> {
    pub const fn new(bytes: &[u8; LEN]) -> Self {
        Self {
            unk: 0,
            name_length: LEN as u32,
            name: *bytes,
        }
    }
}

set_module_name!(b"test_name\0");

#[no_mangle]
pub fn main() {
    log_debug_fmt!("Hello {} from {}!", "world", thread::get_current_thread().get_name().unwrap());
}

#[panic_handler]
fn on_panic(info: &panic::PanicInfo) -> ! {
    let location = info.location().unwrap();
    let thread_name = match thread::get_current_thread().get_name() {
        Ok(name) => name,
        _ => "<unknown>",
    };
    log_debug_fmt!("Panic! at thread '{}', at '{}' -> {:?}", thread_name, location, info.payload());
    loop {}
}