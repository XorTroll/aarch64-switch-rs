enum_define!(ThreadState(u8) {
    NotInitialized = 0,
    Initialized = 1,
    DestroyedBeforeStarted = 2,
    Started = 3,
    Terminated = 4
});

use crate::result::*;
use core::ptr;
use crate::util;

pub type ThreadName = [u8; 0x20];

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Thread {
    pub self_ref: *mut Thread,
    pub state: ThreadState,
    pub owns_stack: bool,
    pub pad: [u8; 2],
    pub handle: u32,
    pub stack: *mut u8,
    pub stack_size: usize,
    pub entry: *mut u8,
    pub entry_arg: *mut u8,
    pub tls_slots: [*mut u8; 0x20],
    pub reserved: [u8; 0x54],
    pub name_len: u32,
    pub name: ThreadName,
    pub name_addr: *mut u8,
    pub reserved_2: [u8; 0x20],
}

impl Thread {
    pub const fn new() -> Self {
        Self {
            self_ref: ptr::null_mut(),
            state: ThreadState::NotInitialized,
            owns_stack: false,
            pad: [0; 2],
            handle: 0,
            stack: ptr::null_mut(),
            stack_size: 0,
            entry: ptr::null_mut(),
            entry_arg: ptr::null_mut(),
            tls_slots: [ptr::null_mut(); 0x20],
            reserved: [0; 0x54],
            name_len: 0,
            name: [0; 0x20],
            name_addr: ptr::null_mut(),
            reserved_2: [0; 0x20],
        }
    }

    pub fn existing(handle: u32, name: &str, stack: *mut u8, stack_size: usize, owns_stack: bool) -> Result<Self> {
        let mut thread = Self {
            self_ref: ptr::null_mut(),
            state: ThreadState::Started,
            owns_stack: owns_stack,
            pad: [0; 2],
            handle: handle,
            stack: stack,
            stack_size: stack_size,
            entry: ptr::null_mut(),
            entry_arg: ptr::null_mut(),
            tls_slots: [ptr::null_mut(); 0x20],
            reserved: [0; 0x54],
            name_len: 0,
            name: [0; 0x20],
            name_addr: ptr::null_mut(),
            reserved_2: [0; 0x20],
        };
        thread.self_ref = &mut thread;
        thread.name_addr = &mut thread.name as *mut ThreadName as *mut u8;
        thread.set_name(name)?;
        Ok(thread)
    }

    pub fn set_name(&mut self, name: &str) -> Result<()> {
        util::copy_str_to_pointer(name, self.name_addr)
    }

    pub fn get_name(&self) -> Result<&'static str> {
        util::get_str_from_pointer(self.name_addr, 0x20)
    }

    pub fn get_handle(&self) -> u32 {
        self.handle
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Tls {
    pub ipc_buffer: [u8; 0x100],
    pub preemption_state: u32,
    pub unk: [u8; 0xF4],
    pub thread_ref: *mut Thread,
}

pub fn get_thread_local_storage() -> *mut Tls {
    let tls: *mut Tls;
    unsafe {
        llvm_asm!("mrs x0, tpidrro_el0" : "={x0}"(tls) ::: "volatile");
    }
    tls
}

pub fn get_current_thread() -> &'static Thread {
    unsafe {
        &*(*get_thread_local_storage()).thread_ref
    }
}