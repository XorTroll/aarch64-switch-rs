use crate::result::*;
use crate::svc;
use crate::crt0;
use crate::service;
use crate::service::fatal;
use crate::service::fatal::IService;
use core::ptr;
use enumflags2::BitFlags;

pub const RESULT_SUBMODULE: u32 = 6;

result_lib_define_group!(RESULT_SUBMODULE => {
    ResultAssertionFailed: 1
});

#[derive(BitFlags, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum AssertMode {
    ProcessExit = 0b1,
    FatalThrow = 0b10,
    SvcBreak = 0b100,
    Panic = 0b1000,
}

pub fn assert(mode: AssertMode, rc: ResultCode) -> ! {
    if rc.is_failure() {
        match mode {
            AssertMode::ProcessExit => {
                crt0::exit(rc);
            },
            AssertMode::FatalThrow => {
                match service::new_service_object::<fatal::Service>() {
                    Ok(mut fatal) => {
                        let _ = fatal.throw_with_policy(rc, fatal::Policy::ErrorScreen);
                    },
                    _ => {}
                }
            },
            AssertMode::SvcBreak => {
                let _ = svc::break_(svc::BreakReason::Assert, ptr::null_mut(), 0);
            },
            AssertMode::Panic => {
                let res: Result<()> = Err(rc);
                res.unwrap();
            },
        }
    }
    loop {}
}