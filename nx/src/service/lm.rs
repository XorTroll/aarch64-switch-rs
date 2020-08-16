use crate::result::*;
use crate::ipc::sf;
use crate::ipc::server;
use crate::service;
use crate::mem;

pub use crate::ipc::sf::lm::*;

pub struct Logger {
    session: service::Session
}

impl service::ISessionObject for Logger {
    fn new(session: service::Session) -> Self {
        Self { session: session }
    }
    
    fn get_session(&mut self) -> &mut service::Session {
        &mut self.session
    }
}

impl ILogger for Logger {
    fn log(&mut self, log_buf: sf::InAutoSelectBuffer) -> Result<()> {
        ipc_client_send_request_command!([self.session.session; 0] (log_buf) => ())
    }

    fn set_destination(&mut self, log_destination: LogDestination) -> Result<()> {
        ipc_client_send_request_command!([self.session.session; 1] (log_destination) => ())
    }
}

impl server::IServer for Logger {
    fn get_command_table(&self) -> server::CommandMetadataTable {
        ipc_server_make_command_table! {
            log: 0,
            set_destination: 1
        }
    }
}

pub struct LogService {
    session: service::Session
}

impl service::ISessionObject for LogService {
    fn new(session: service::Session) -> Self {
        Self { session: session }
    }
    
    fn get_session(&mut self) -> &mut service::Session {
        &mut self.session
    }
}

impl ILogService for LogService {
    fn open_logger(&mut self, process_id: sf::ProcessId) -> Result<mem::Shared<dyn service::ISessionObject>> {
        ipc_client_send_request_command!([self.session.session; 0] (process_id) => (logger: mem::Shared<Logger>))
    }
}

impl server::IServer for LogService {
    fn get_command_table(&self) -> server::CommandMetadataTable {
        ipc_server_make_command_table! {
            open_logger: 0
        }
    }
}

impl service::IService for LogService {
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