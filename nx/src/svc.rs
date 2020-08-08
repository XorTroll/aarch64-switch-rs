use crate::result::*;
use core::ptr;
use core::mem;
use enumflags2::BitFlags;

global_asm!(include_str!("svc.s"));

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

#[derive(Copy, Clone, PartialEq)]
#[repr(u32)]
pub enum InfoId {
    CoreMask = 0,
    PriorityMask = 1,
    AliasRegionAddress = 2,
    AliasRegionSize = 3,
    HeapRegionAddress = 4,
    HeapRegionSize = 5,
    TotalMemorySize = 6,
    UsedMemorySize = 7,
    DebuggerAttached = 8,
    ResourceLimit = 9,
    IdleTickCount = 10,
    RandomEntropy = 11,
    AslrRegionAddress = 12,
    AslrRegionSize = 13,
    StackRegionAddress = 14,
    StackRegionSize = 15,
    SystemResourceSizeTotal = 16,
    SystemResourceSizeUsed = 17,
    ProgramId = 18,
    InitialProcessIdRange = 19,
    UserExceptionContextAddress = 20,
    TotalNonSystemMemorySize = 21,
    UsedNonSystemMemorySize = 22,
    IsApplication = 23,
}

pub type PageInfo = u32;
pub type Address = *const u8;
pub type Size = usize;
pub type ThreadEntrypointFn = fn(*mut u8);
pub type Handle = u32;

pub const CURRENT_THREAD_PSEUDO_HANDLE: Handle = 0xFFFF8000;
pub const CURRENT_PROCESS_PSEUDO_HANDLE: Handle = 0xFFFF8001;

// TODO: move all svcs to svc.s

pub fn set_heap_size(size: Size) -> Result<*mut u8> {
    extern "C" {
        fn __nx_svc_set_heap_size(out_address: *mut *mut u8, size: Size) -> ResultCode;
    }

    unsafe {
        let mut address: *mut u8 = ptr::null_mut();

        let rc = __nx_svc_set_heap_size(&mut address, size);
        wrap(rc, address)
    }
}

pub fn set_memory_attribute(address: Address, size: Size, mask: u32, value: BitFlags<MemoryAttribute>) -> Result<()> {
    extern "C" {
        fn __nx_svc_set_memory_attribute(address: Address, size: Size, mask: u32, value: BitFlags<MemoryAttribute>) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_set_memory_attribute(address, size, mask, value);
        wrap(rc, ())
    }
}

pub fn query_memory(address: *const u8) -> Result<(MemoryInfo, PageInfo)> {
    extern "C" {
        fn __nx_svc_query_memory(out_info: *mut MemoryInfo, out_page_info: *mut PageInfo, address: *const u8) -> ResultCode;
    }

    unsafe {
        let mut memory_info: MemoryInfo = mem::zeroed();
        let mut page_info: PageInfo = 0;

        let rc = __nx_svc_query_memory(&mut memory_info, &mut page_info, address);
        wrap(rc, (memory_info, page_info))
    }
}

pub fn exit_process() {
    extern "C" {
        fn __nx_svc_exit_process();
    }

    unsafe {
        __nx_svc_exit_process();
    }
}

pub fn map_shared_memory(handle: Handle, address: Address, size: Size, permission: BitFlags<MemoryPermission>) -> Result<()> {
    extern "C" {
        fn __nx_svc_map_shared_memory(handle: Handle, address: Address, size: Size, permission: BitFlags<MemoryPermission>) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_map_shared_memory(handle, address, size, permission);
        wrap(rc, ())
    }
}

pub fn unmap_shared_memory(handle: Handle, address: Address, size: Size) -> Result<()> {
    extern "C" {
        fn __nx_svc_unmap_shared_memory(handle: Handle, address: Address, size: Size) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_unmap_shared_memory(handle, address, size);
        wrap(rc, ())
    }
}

pub fn create_transfer_memory(address: Address, size: Size, permissions: BitFlags<MemoryPermission>) -> Result<Handle> {
    extern "C" {
        fn __nx_svc_create_transfer_memory(out_handle: *mut Handle, address: Address, size: Size, permissions: BitFlags<MemoryPermission>) -> ResultCode;
    }

    unsafe {
        let mut handle: Handle = 0;

        let rc = __nx_svc_create_transfer_memory(&mut handle, address, size, permissions);
        wrap(rc, handle)
    }
}

pub fn close_handle(handle: Handle) -> Result<()> {
    extern "C" {
        fn __nx_svc_close_handle(handle: Handle) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_close_handle(handle);
        wrap(rc, ())
    }
}

pub fn reset_signal(handle: Handle) -> Result<()> {
    extern "C" {
        fn __nx_svc_reset_signal(handle: Handle) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_reset_signal(handle);
        wrap(rc, ())
    }
}

pub fn wait_synchronization(handles: *const Handle, handle_count: u32, timeout: i64) -> Result<i32> {
    extern "C" {
        fn __nx_svc_wait_synchronization(out_index: *mut i32, handles: *const Handle, handle_count: u32, timeout: i64) -> ResultCode;
    }

    unsafe {
        let mut index: i32 = 0;

        let rc = __nx_svc_wait_synchronization(&mut index, handles, handle_count, timeout);
        wrap(rc, index)
    }
}

pub fn arbitrate_lock(thread_handle: Handle, tag_location: Address, tag: u32) -> Result<()> {
    extern "C" {
        fn __nx_svc_arbitrate_lock(thread_handle: Handle, tag_location: Address, tag: u32) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_arbitrate_lock(thread_handle, tag_location, tag);
        wrap(rc, ())
    }
}

pub fn arbitrate_unlock(tag_location: Address) -> Result<()> {
    extern "C" {
        fn __nx_svc_arbitrate_unlock(tag_location: Address) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_arbitrate_unlock(tag_location);
        wrap(rc, ())
    }
}

pub fn connect_to_named_port(name: Address) -> Result<Handle> {
    extern "C" {
        fn __nx_svc_connect_to_named_port(out_handle: *mut Handle, name: Address) -> ResultCode;
    }

    unsafe {
        let mut handle: Handle = 0;

        let rc = __nx_svc_connect_to_named_port(&mut handle, name);
        wrap(rc, handle)
    }
}

pub fn send_sync_request(handle: Handle) -> Result<()> {
    extern "C" {
        fn __nx_svc_send_sync_request(handle: Handle) -> ResultCode;
    }
    
    unsafe {
        let rc = __nx_svc_send_sync_request(handle);
        wrap(rc, ())
    }
}

pub fn get_process_id(process_handle: Handle) -> Result<u64> {
    extern "C" {
        fn __nx_svc_get_process_id(out_process_id: *mut u64, process_handle: Handle) -> ResultCode;
    }
    
    unsafe {
        let mut process_id: u64 = 0;

        let rc = __nx_svc_get_process_id(&mut process_id, process_handle);
        wrap(rc, process_id)
    }
}

pub fn break_(reason: BreakReason, arg: Address, size: Size) -> Result<()> {
    extern "C" {
        fn __nx_svc_break(reason: BreakReason, arg: Address, size: Size) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_break(reason, arg, size);
        wrap(rc, ())
    }
}

pub fn output_debug_string(msg: Address, len: Size) -> Result<()> {
    extern "C" {
        fn __nx_svc_output_debug_string(msg: Address, len: Size) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_output_debug_string(msg, len);
        wrap(rc, ())
    }
}

pub fn return_from_exception(res: ResultCode) {
    extern "C" {
        fn __nx_svc_return_from_exception(res: ResultCode);
    }

    unsafe {
        __nx_svc_return_from_exception(res);
    }
}

pub fn get_info(id: InfoId, handle: Handle, sub_id: u64) -> Result<u64> {
    extern "C" {
        fn __nx_svc_get_info(out_info: *mut u64, id: InfoId, handle: Handle, sub_id: u64) -> ResultCode;
    }
    
    unsafe {
        let mut info: u64 = 0;

        let rc = __nx_svc_get_info(&mut info, id, handle, sub_id);
        wrap(rc, info)
    }
}

result_define_group!(1 => {
    ResultInvalidSize: 101,
    ResultInvalidAddress: 102,
    ResultInvalidHandle: 114,
    ResultUnhandledException: 124,
    ResultFatalException: 128
});