use crate::result::*;
use crate::svc;
use crate::ipc;
use crate::service;
use crate::service::SessionObject;

#[derive(Copy, Clone, PartialEq, Debug)]
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

pub const RESULT_SUBMODULE: u32 = 7;

result_lib_define_group!(RESULT_SUBMODULE => {
    ResultErrorCodeInvalid: 1,
    ResultErrorCodeNotImplemented: 2,
    ResultErrorCodeNotSupported: 3,
    ResultErrorCodeNotInitialized: 4,
    ResultErrorCodeInvalidParameter: 5,
    ResultErrorCodeTimeOut: 6,
    ResultErrorCodeInsufficientMemory: 7,
    ResultErrorCodeReadOnlyAttribute: 8,
    ResultErrorCodeInvalidState: 9,
    ResultErrorCodeInvalidAddress: 10,
    ResultErrorCodeInvalidSize: 11,
    ResultErrorCodeInvalidValue: 12,
    ResultErrorCodeAlreadyAllocated: 13,
    ResultErrorCodeBusy: 14,
    ResultErrorCodeResourceError: 15,
    ResultErrorCodeCountMismatch: 16,
    ResultErrorCodeSharedMemoryTooSmall: 17,
    ResultErrorCodeFileOperationFailed: 18,
    ResultErrorCodeIoctlFailed: 19
});

#[allow(unreachable_patterns)]
pub fn convert_error_code(err: ErrorCode) -> Result<()> {
    match err {
        ErrorCode::Success => Ok(()),
        ErrorCode::NotImplemented => Err(ResultCode::from::<ResultErrorCodeNotImplemented>()),
        ErrorCode::NotSupported => Err(ResultCode::from::<ResultErrorCodeNotSupported>()),
        ErrorCode::NotInitialized => Err(ResultCode::from::<ResultErrorCodeNotInitialized>()),
        ErrorCode::InvalidParameter => Err(ResultCode::from::<ResultErrorCodeInvalidParameter>()),
        ErrorCode::TimeOut => Err(ResultCode::from::<ResultErrorCodeTimeOut>()),
        ErrorCode::InsufficientMemory => Err(ResultCode::from::<ResultErrorCodeInsufficientMemory>()),
        ErrorCode::ReadOnlyAttribute => Err(ResultCode::from::<ResultErrorCodeReadOnlyAttribute>()),
        ErrorCode::InvalidState => Err(ResultCode::from::<ResultErrorCodeInvalidState>()),
        ErrorCode::InvalidAddress => Err(ResultCode::from::<ResultErrorCodeInvalidAddress>()),
        ErrorCode::InvalidSize => Err(ResultCode::from::<ResultErrorCodeInvalidSize>()),
        ErrorCode::InvalidValue => Err(ResultCode::from::<ResultErrorCodeInvalidValue>()),
        ErrorCode::AlreadyAllocated => Err(ResultCode::from::<ResultErrorCodeAlreadyAllocated>()),
        ErrorCode::Busy => Err(ResultCode::from::<ResultErrorCodeBusy>()),
        ErrorCode::ResourceError => Err(ResultCode::from::<ResultErrorCodeResourceError>()),
        ErrorCode::CountMismatch => Err(ResultCode::from::<ResultErrorCodeCountMismatch>()),
        ErrorCode::SharedMemoryTooSmall => Err(ResultCode::from::<ResultErrorCodeSharedMemoryTooSmall>()),
        ErrorCode::FileOperationFailed => Err(ResultCode::from::<ResultErrorCodeFileOperationFailed>()),
        ErrorCode::IoctlFailed => Err(ResultCode::from::<ResultErrorCodeIoctlFailed>()),
        _ => Err(ResultCode::from::<ResultErrorCodeInvalid>()),
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
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

pub trait INvDrvService {
    fn open_fd(&mut self, path: *const u8, path_len: usize) -> Result<(u32, ErrorCode)>;

    fn ioctl(&mut self, fd: u32, ioctl_id: IoctlId, in_buf: *const u8, in_buf_size: usize, out_buf: *const u8, out_buf_size: usize) -> Result<ErrorCode>;

    fn close_fd(&mut self, fd: u32) -> Result<ErrorCode>;

    fn initialize(&mut self, transfer_mem_handle: svc::Handle, transfer_mem_size: u32) -> Result<ErrorCode>;
}

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