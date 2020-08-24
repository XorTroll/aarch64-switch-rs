use crate::result::*;
use crate::results;
use crate::svc;
use crate::arm;

pub enum WaiterType {
    Handle,
    HandleWithClear
}

pub const MAX_OBJECT_COUNT: u32 = 0x40;

#[allow(dead_code)]
pub struct Waiter {
    handle: svc::Handle,
    wait_type: WaiterType
}

impl Waiter {
    pub const fn from(handle: svc::Handle, wait_type: WaiterType) -> Self {
        Self { handle: handle, wait_type: wait_type }
    }
    
    pub const fn from_handle(handle: svc::Handle) -> Self {
        Self::from(handle, WaiterType::Handle)
    }

    pub const fn from_handle_with_clear(handle: svc::Handle) -> Self {
        Self::from(handle, WaiterType::HandleWithClear)
    }
}

type WaitFn<W> = fn(&[W], i64) -> Result<usize>;

fn handles_wait_fn(handles: &[svc::Handle], timeout: i64) -> Result<usize> {
    Ok(svc::wait_synchronization(handles.as_ptr(), handles.len() as u32, timeout)? as usize)
}

fn waiters_wait_fn(_waiters: &[Waiter], _timeout: i64) -> Result<usize> {
    todo!();
}

fn wait_impl<W>(wait_objects: &[W], timeout: i64, wait_fn: WaitFn<W>) -> Result<usize> {
    let has_timeout = timeout != -1;
    let mut deadline: u64 = 0;
    if has_timeout {
        deadline = arm::get_system_tick() - arm::nanoseconds_to_ticks(timeout as u64);
    }

    loop {
        let this_timeout = match has_timeout {
            true => {
                let remaining = deadline - arm::get_system_tick();
                arm::ticks_to_nanoseconds(remaining) as i64
            },
            false => -1
        };
        match (wait_fn)(wait_objects, this_timeout) {
            Ok(index) => return Ok(index),
            Err(rc) => {
                if results::os::ResultTimeout::matches(rc) {
                    if has_timeout {
                        return Err(rc);
                    }
                }
                else if !results::os::ResultOperationCanceled::matches(rc) {
                    return Err(rc);
                }
            }
        }
    }

    // Err(ResultCode::new(0x2345))
}

pub fn wait(waiters: &[Waiter], timeout: i64) -> Result<usize> {
    wait_impl(waiters, timeout, waiters_wait_fn)
}

pub fn wait_handles(handles: &[svc::Handle], timeout: i64) -> Result<usize> {
    wait_impl(handles, timeout, handles_wait_fn)
}