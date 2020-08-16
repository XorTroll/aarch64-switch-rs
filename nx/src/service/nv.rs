use crate::result::*;
use crate::results;
use crate::ipc::sf;
use crate::ipc::server;
use crate::service;

pub use crate::ipc::sf::nv::*;

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

// NvDrvService is the base trait for all the different services, since the only difference is their service names :P
pub trait NvDrvService {}

impl<S: NvDrvService + service::ISessionObject> INvDrvService for S {
    fn open(&mut self, path: sf::InMapAliasBuffer) -> Result<(Fd, ErrorCode)> {
        ipc_client_send_request_command!([self.get_inner_session(); 0] (path) => (fd: Fd, error_code: ErrorCode))
    }

    fn ioctl(&mut self, fd: Fd, id: IoctlId, in_buf: sf::InAutoSelectBuffer, out_buf: sf::OutAutoSelectBuffer) -> Result<ErrorCode> {
        ipc_client_send_request_command!([self.get_inner_session(); 1] (fd, id, in_buf, out_buf) => (error_code: ErrorCode))
    }

    fn close(&mut self, fd: Fd) -> Result<ErrorCode> {
        ipc_client_send_request_command!([self.get_inner_session(); 2] (fd) => (error_code: ErrorCode))
    }

    fn initialize(&mut self, transfer_mem_size: u32, self_process_handle: sf::CopyHandle, transfer_mem_handle: sf::CopyHandle) -> Result<ErrorCode> {
        ipc_client_send_request_command!([self.get_inner_session(); 3] (transfer_mem_size, self_process_handle, transfer_mem_handle) => (error_code: ErrorCode))
    }
}

impl<S: NvDrvService + service::ISessionObject> server::IServer for S {
    fn get_command_table(&self) -> server::CommandMetadataTable {
        ipc_server_make_command_table! {
            open: 0,
            ioctl: 1,
            close: 2,
            initialize: 3
        }
    }
}

pub struct AppletNvDrvService {
    session: service::Session
}

impl service::ISessionObject for AppletNvDrvService {
    fn new(session: service::Session) -> Self {
        Self { session: session }
    }
    
    fn get_session(&mut self) -> &mut service::Session {
        &mut self.session
    }
}

impl NvDrvService for AppletNvDrvService {}

impl service::IService for AppletNvDrvService {
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