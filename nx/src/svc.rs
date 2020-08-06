use crate::result::*;
use enumflags2::BitFlags;

#[derive(Copy, Clone, PartialEq)]
#[repr(u32)]
pub enum BreakReason {
    Panic = 0,
    Assert = 1,
    User = 2,
    PreLoadDll = 3,
    PostLoadDll = 4,
    PreUnloadDll = 5,
    PostUnloadDll = 6,
    CppException = 7,
    NotificationOnlyFlag = 0x80000000
}

#[derive(Copy, Clone, PartialEq)]
#[repr(u32)]
pub enum MemoryState {
    Free = 0x0,
    Io = 0x1,
    Static = 0x2,
    Code = 0x3,
    CodeData = 0x4,
    Normal = 0x5,
    Shared = 0x6,
    Alias = 0x7,
    AliasCode = 0x8,
    AliasCodeData = 0x9,
    Ipc = 0xA,
    Stack = 0xB,
    ThreadLocal = 0xC,
    Transfered = 0xD,
    SharedTransfered = 0xE,
    SharedCode = 0xF,
    Inaccessible = 0x10,
    NonSecureIpc = 0x11,
    NonDeviceIpc = 0x12,
    Kernel = 0x13,
    GeneratedCode = 0x14,
    CodeOut = 0x15
}

#[derive(BitFlags, Copy, Clone, PartialEq)]
#[repr(u32)]
pub enum MemoryPermission {
    Read = 0b1,
    Write = 0b10,
    Execute = 0b100,
    DontCare = 0b10000000000000000000000000000,
}

#[derive(BitFlags, Copy, Clone, PartialEq)]
#[repr(u32)]
pub enum MemoryAttribute {
    Locked = 0b1,
    IpcLocked = 0b10,
    DeviceShared = 0b100,
    Uncached = 0b1000,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct MemoryInfo {
    pub base_address: *mut u8,
    pub size: u64,
    pub memory_state: MemoryState,
    pub memory_attribute: MemoryAttribute,
    pub memory_permission: MemoryPermission,
    pub ipc_refcount: u32,
    pub device_refcount: u32,
    pub pad: u32,
}

pub type PageInfo = u32;
pub type Address = *mut u8;
pub type Size = usize;
pub type ThreadEntrypointFn = fn(*mut u8);
pub type Handle = u32;

pub const CURRENT_THREAD_PSEUDO_HANDLE: Handle = 0xFFFF8000;
pub const CURRENT_PROCESS_PSEUDO_HANDLE: Handle = 0xFFFF8001;

pub fn set_heap_size(size: Size) -> Result<Address> {
    let mut rc: ResultCode;
    let address: *mut u8;
    unsafe {
        llvm_asm!("svc 0x1" : "={w0}"(rc), "={x1}"(address) : "{x1}"(size) :: "volatile");
    }
    wrap(rc, address)
}

pub fn set_memory_attribute(address: Address, size: Size, mask: u32, value: BitFlags<MemoryAttribute>) -> Result<()> {
    let mut rc: ResultCode;
    unsafe {
        llvm_asm!("svc 0x3" : "={w0}"(rc) : "{x0}"(address), "{x1}"(size), "{w2}"(mask), "{w3}"(value) :: "volatile");
    }
    wrap(rc, ())
}

pub fn query_memory(out_info: *mut MemoryInfo, address: *const u8) -> Result<PageInfo> {
    let rc: ResultCode;
    let info: PageInfo;
    unsafe {
        llvm_asm!("svc 0x6" : "={w0}"(rc), "={w1}"(info) : "{x0}"(out_info), "{x2}"(address) :: "volatile");
    }
    wrap(rc, info)
}

pub fn exit_process() {
    unsafe {
        llvm_asm!("svc 0x7" :::: "volatile");
    }
}

pub fn create_transfer_memory(address: Address, size: Size, permissions: BitFlags<MemoryPermission>) -> Result<Handle> {
    let rc: ResultCode;
    let handle: Handle;
    unsafe {
        llvm_asm!("svc 0x15" : "={w0}"(rc), "={w1}"(handle) : "{x1}"(address), "{x2}"(size), "{w3}"(permissions) :: "volatile");
    }
    wrap(rc, handle)
}

pub fn close_handle(handle: Handle) -> Result<()> {
    let rc: ResultCode;
    unsafe {
        llvm_asm!("svc 0x16" : "={w0}"(rc) : "{w0}"(handle) :: "volatile");
    }
    wrap(rc, ())
}

pub fn arbitrate_lock(thread_handle: u32, address: Address, tag: u32) -> Result<()> {
    let rc: ResultCode;
    unsafe {
        llvm_asm!("svc 0x1A" : "={w0}"(rc) : "{w0}"(thread_handle), "{x1}"(address), "{w2}"(tag) :: "volatile");
    }
    wrap(rc, ())
}

pub fn arbitrate_unlock(address: Address) -> Result<()> {
    let rc: ResultCode;
    unsafe {
        llvm_asm!("svc 0x1B" : "={w0}"(rc) : "{x0}"(address) :: "volatile");
    }
    wrap(rc, ())
}

pub fn connect_to_named_port(name: *const u8) -> Result<Handle> {
    let rc: ResultCode;
    let handle: Handle;
    unsafe {
        llvm_asm!("svc 0x1F" : "={w0}"(rc), "={w1}"(handle) : "{x1}"(name) :: "volatile");
    }
    wrap(rc, handle)
}

pub fn send_sync_request(handle: Handle) -> Result<()> {
    let rc: ResultCode;
    unsafe {
        llvm_asm!("svc 0x21" : "={w0}"(rc) : "{w0}"(handle) :: "volatile");
    }
    wrap(rc, ())
}

pub fn get_process_id(process_handle: Handle) -> Result<u64> {
    let rc: ResultCode;
    let process_id: u64;
    unsafe {
        llvm_asm!("svc 0x24" : "={w0}"(rc), "={x1}"(process_id) : "{w1}"(process_handle) :: "volatile");
    }
    wrap(rc, process_id)
}

pub fn break_(reason: BreakReason, arg: Address, size: Size) -> Result<()> {
    let rc: ResultCode;
    unsafe {
        llvm_asm!("svc 0x26" : "={w0}"(rc) : "{x0}"(reason), "{x1}"(arg), "{x2}"(size) :: "volatile");
    }
    wrap(rc, ())
}

pub fn output_debug_string(msg: *const u8, len: Size) -> Result<()> {
    let rc: ResultCode;
    unsafe {
        llvm_asm!("svc 0x27" : "={w0}"(rc) : "{x0}"(msg), "{x1}"(len) :: "volatile");
    }
    wrap(rc, ())
}

pub fn return_from_exception(rc: ResultCode) {
    unsafe {
        llvm_asm!("svc 0x28" :: "{w0}"(rc) :: "volatile");
    }
}

result_define_group!(1 => {
    ResultInvalidSize: 101,
    ResultInvalidAddress: 102,
    ResultInvalidHandle: 114,
    ResultUnhandledException: 124,
    ResultFatalException: 128
});