use crate::result::*;
use crate::results;
use crate::svc;
use crate::ipc;
use crate::service;
use crate::service::SessionObject;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum ErrorCode {
    Success = 0,
    NotImplemented = 1,
    NotSupported = 2,
    NotInitialized = 3,
    InvalidParameter = 4,
    TimeOut = 5,
    InsufficientMemory = 6,
    ReadOnlyAttribute = 7,
    InvalidState = 8,
    InvalidAddress = 9,
    InvalidSize = 10,
    InvalidValue = 11,
    AlreadyAllocated = 13,
    Busy = 14,
    ResourceError = 15,
    CountMismatch = 16,
    SharedMemoryTooSmall = 0x1000,
    FileOperationFailed = 0x30003,
    IoctlFailed = 0x3000F,
}

#[allow(unreachable_patterns)]
pub fn convert_error_code(err: ErrorCode) -> Result<()> {
    match err {
        ErrorCode::Success => Ok(()),
        ErrorCode::NotImplemented => Err(results::lib::gpu::ResultNvErrorCodeNotImplemented::make()),
        ErrorCode::NotSupported => Err(results::lib::gpu::ResultNvErrorCodeNotSupported::make()),
        ErrorCode::NotInitialized => Err(results::lib::gpu::ResultNvErrorCodeNotInitialized::make()),
        ErrorCode::InvalidParameter => Err(results::lib::gpu::ResultNvErrorCodeInvalidParameter::make()),
        ErrorCode::TimeOut => Err(results::lib::gpu::ResultNvErrorCodeTimeOut::make()),
        ErrorCode::InsufficientMemory => Err(results::lib::gpu::ResultNvErrorCodeInsufficientMemory::make()),
        ErrorCode::ReadOnlyAttribute => Err(results::lib::gpu::ResultNvErrorCodeReadOnlyAttribute::make()),
        ErrorCode::InvalidState => Err(results::lib::gpu::ResultNvErrorCodeInvalidState::make()),
        ErrorCode::InvalidAddress => Err(results::lib::gpu::ResultNvErrorCodeInvalidAddress::make()),
        ErrorCode::InvalidSize => Err(results::lib::gpu::ResultNvErrorCodeInvalidSize::make()),
        ErrorCode::InvalidValue => Err(results::lib::gpu::ResultNvErrorCodeInvalidValue::make()),
        ErrorCode::AlreadyAllocated => Err(results::lib::gpu::ResultNvErrorCodeAlreadyAllocated::make()),
        ErrorCode::Busy => Err(results::lib::gpu::ResultNvErrorCodeBusy::make()),
        ErrorCode::ResourceError => Err(results::lib::gpu::ResultNvErrorCodeResourceError::make()),
        ErrorCode::CountMismatch => Err(results::lib::gpu::ResultNvErrorCodeCountMismatch::make()),
        ErrorCode::SharedMemoryTooSmall => Err(results::lib::gpu::ResultNvErrorCodeSharedMemoryTooSmall::make()),
        ErrorCode::FileOperationFailed => Err(results::lib::gpu::ResultNvErrorCodeFileOperationFailed::make()),
        ErrorCode::IoctlFailed => Err(results::lib::gpu::ResultNvErrorCodeIoctlFailed::make()),
        _ => Err(results::lib::gpu::ResultNvErrorCodeInvalid::make()),
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum IoctlId {
    NvMapCreate = 0xC0080101,
    NvMapFromId = 0xC0080103,
    NvMapAlloc = 0xC0200104,
    NvMapFree = 0xC0180105,
    NvMapParam = 0xC00C0109,
    NvMapGetId = 0xC008010E,

    NvHostCtrlSyncptWait = 0xC00C0016,
} 

// Note: open_fd and close_fd's original names don't contain the "_fd" bit, but those were changed to avoid the ambiguity with SessionObject::close
pub trait INvDrvService {
    fn open_fd(&mut self, path: *const u8, path_len: usize) -> Result<(u32, ErrorCode)>;
    fn ioctl(&mut self, fd: u32, ioctl_id: IoctlId, in_buf: *const u8, in_buf_size: usize, out_buf: *const u8, out_buf_size: usize) -> Result<ErrorCode>;
    fn close_fd(&mut self, fd: u32) -> Result<ErrorCode>;
    fn initialize(&mut self, transfer_mem_handle: svc::Handle, transfer_mem_size: u32) -> Result<ErrorCode>;
}

// NvDrvService is the base trait for all the different services whose only difference is their service names :P
pub trait NvDrvService {}

impl<T: NvDrvService + SessionObject> INvDrvService for T {
    fn open_fd(&mut self, path: *const u8, path_len: usize) -> Result<(u32, ErrorCode)> {
        let fd: u32;
        let err_code: ErrorCode;
        ipc_client_session_send_request_command!([self.get_session(); 0; false] => {
            In {};
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {
                (path, path_len) => ipc::BufferAttribute::In | ipc::BufferAttribute::MapAlias
            };
            Out {
                fd: u32 => fd,
                err_code: ErrorCode => err_code
            };
            OutHandles {};
            OutObjects {};
            OutSessions {};
        });
        Ok((fd, err_code))
    }

    fn ioctl(&mut self, fd: u32, ioctl_id: IoctlId, in_buf: *const u8, in_buf_size: usize, out_buf: *const u8, out_buf_size: usize) -> Result<ErrorCode> {
        let err_code: ErrorCode;
        ipc_client_session_send_request_command!([self.get_session(); 1; false] => {
            In {
                fd: u32 = fd,
                ioctl_id: IoctlId = ioctl_id
            };
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {
                (in_buf, in_buf_size) => ipc::BufferAttribute::In | ipc::BufferAttribute::AutoSelect,
                (out_buf, out_buf_size) => ipc::BufferAttribute::Out | ipc::BufferAttribute::AutoSelect
            };
            Out {
                err_code: ErrorCode => err_code
            };
            OutHandles {};
            OutObjects {};
            OutSessions {};
        });
        Ok(err_code)
    }

    fn close_fd(&mut self, fd: u32) -> Result<ErrorCode> {
        let err_code: ErrorCode;
        ipc_client_session_send_request_command!([self.get_session(); 2; false] => {
            In {
                fd: u32 = fd
            };
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {};
            Out {
                err_code: ErrorCode => err_code
            };
            OutHandles {};
            OutObjects {};
            OutSessions {};
        });
        Ok(err_code)
    }

    fn initialize(&mut self, transfer_mem_handle: svc::Handle, transfer_mem_size: u32) -> Result<ErrorCode> {
        let err_code: ErrorCode;
        ipc_client_session_send_request_command!([self.get_session(); 3; false] => {
            In {
                transfer_mem_size: u32 = transfer_mem_size
            };
            InHandles {
                svc::CURRENT_PROCESS_PSEUDO_HANDLE => ipc::HandleMode::Copy,
                transfer_mem_handle => ipc::HandleMode::Copy
            };
            InObjects {};
            InSessions {};
            Buffers {};
            Out {
                err_code: ErrorCode => err_code
            };
            OutHandles {};
            OutObjects {};
            OutSessions {};
        });
        Ok(err_code)
    }
}

session_object_define!(AppletNvDrvService);

impl service::Service for AppletNvDrvService {
    fn get_name() -> &'static str {
        nul!("nvdrv:a")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}

impl NvDrvService for AppletNvDrvService {}