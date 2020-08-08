use crate::result::*;
use crate::ipc;
use crate::service;
use crate::service::SessionObject;
use enumflags2::BitFlags;

#[derive(BitFlags, Copy, Clone, PartialEq, Debug)]
#[repr(u32)]
pub enum LogDestination {
    TMA = 0b1,
    UART = 0b10,
    UARTSleeping = 0b100,
}

pub trait ILogger {
    fn log(&mut self, buf: *const u8, buf_size: usize) -> Result<()>;

    fn set_destination(&mut self, log_destination: BitFlags<LogDestination>) -> Result<()>;
}

session_object_define!(Logger);

impl ILogger for Logger {
    fn log(&mut self, buf: *const u8, buf_size: usize) -> Result<()> {
        ipc_client_session_send_request_command!([self.session; 0; false] => {
            In {};
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {
                (buf, buf_size) => ipc::BufferAttribute::In | ipc::BufferAttribute::AutoSelect
            };
            Out {};
            OutHandles {};
            OutObjects {};
            OutSessions {};
        });
        Ok(())
    }

    fn set_destination(&mut self, log_destination: BitFlags<LogDestination>) -> Result<()> {
        ipc_client_session_send_request_command!([self.session; 1; false] => {
            In {
                log_dest: BitFlags<LogDestination> = log_destination
            };
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {};
            Out {};
            OutHandles {};
            OutObjects {};
            OutSessions {};
        });
        Ok(())
    }
}

pub trait ILogService {
    fn open_logger<S: SessionObject>(&mut self) -> Result<S>;
}

session_object_define!(LogService);

impl service::Service for LogService {
    fn get_name() -> &'static str {
        nul!("lm")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}

impl ILogService for LogService {
    fn open_logger<S: SessionObject>(&mut self) -> Result<S> {
        let logger: ipc::Session;
        ipc_client_session_send_request_command!([self.session; 0; true] => {
            In {
                process_id_holder: u64 = 0
            };
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {};
            Out {};
            OutHandles {};
            OutObjects {};
            OutSessions {
                logger
            };
        });
        Ok(S::new(logger))
    }
}