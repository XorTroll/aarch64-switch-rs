use crate::result::*;
use crate::sync;
use crate::svc;
use crate::crt0;
use core::ptr;
use enumflags2::BitFlags;

#[derive(BitFlags, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum AssertMode {
    ProcessExit = 0b1,
    FatalThrow = 0b10,
    SvcBreak = 0b100,
    Panic = 0b1000,
}

static mut G_DEFAULT_ASSERT_MODE: sync::Locked<AssertMode> = sync::Locked::new(false, AssertMode::ProcessExit);

pub fn set_default_assert_mode(mode: AssertMode) {
    unsafe {
        G_DEFAULT_ASSERT_MODE.set(mode);
    }
}

pub fn get_default_assert_mode() -> AssertMode {
    unsafe {
        *G_DEFAULT_ASSERT_MODE.get()
    }
}

pub fn assert(mode: AssertMode, rc: ResultCode) {
    if rc.is_failure() {
        match mode {
            AssertMode::ProcessExit => {
                crt0::exit(rc);
            },
            AssertMode::FatalThrow => {
                todo!();
            },
            AssertMode::SvcBreak => {
                // TODO: handle result...?
                let _ = svc::break_(svc::BreakReason::Assert, ptr::null_mut(), 0);
            },
            AssertMode::Panic => {
                let res: Result<()> = Err(rc);
                res.unwrap();
            },
        }
    }
}

pub fn assert_default(rc: ResultCode) {
    assert(get_default_assert_mode(), rc);
}