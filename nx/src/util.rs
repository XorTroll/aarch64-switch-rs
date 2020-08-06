use crate::result::*;
use crate::thread;
use crate::diag::assert;
use crate::diag::log::Logger;
use core::str;
use core::ptr;
use core::panic;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct PointerAndSize {
    pub address: *mut u8,
    pub size: usize
}

impl PointerAndSize {
    pub const fn new(address: *mut u8, size: usize) -> Self {
        Self { address: address, size: size }
    }

    pub fn is_valid(&self) -> bool {
        !self.address.is_null() && (self.size != 0)
    }
}

pub const RESULT_SUBMODULE: u32 = 3;

result_lib_define_group!(RESULT_SUBMODULE => {
    ResultInvalidPointer: 1,
    ResultInvalidSize: 2,
    ResultInvalidConversion: 3
});

pub fn get_str_from_pointer(ptr: *mut u8, ptr_size: usize) -> Result<&'static str> {
    result_return_if!(ptr.is_null(), ResultInvalidPointer);
    result_return_if!(ptr_size == 0, ResultInvalidSize);

    unsafe {
        match core::str::from_utf8(core::slice::from_raw_parts_mut(ptr, ptr_size)) {
            Ok(name) => Ok(name.trim_matches('\0')),
            Err(_) => Err(ResultCode::from::<ResultInvalidConversion>())
        }
    }
}

pub fn copy_str_to_pointer(string: &str, ptr: *mut u8) -> Result<()> {
    result_return_if!(ptr.is_null(), ResultInvalidPointer);
    result_return_if!(string.is_empty(), ResultInvalidSize);

    unsafe {
        ptr::copy(string.as_ptr(), ptr, string.len());
    }
    Ok(())
}

pub fn on_panic_handler<L: Logger>(info: &panic::PanicInfo, assert_mode: assert::AssertMode, rc: ResultCode) -> ! {
    let thread_name = match thread::get_current_thread().get_name() {
        Ok(name) => name,
        _ => "<unknown>",
    };
    diag_log!(L { crate::diag::log::LogSeverity::Fatal, true } => "Panic! at thread '{}' -> {}", thread_name, info);
    assert::assert(assert_mode, rc)
}