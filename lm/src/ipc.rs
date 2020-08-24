use nx::result::*;
use nx::mem;
use nx::ipc::sf;
use nx::ipc::server;
use nx::ipc::sf::lm;
use nx::ipc::sf::lm::ILogger;
use nx::ipc::sf::lm::ILogService;

use crate::logger;

pub struct Logger {
    session: sf::Session,
    log_destination: lm::LogDestination,
    program_id: u64
}

impl Logger {
    pub fn new(program_id: u64) -> Self {
        Self { session: sf::Session::new(), log_destination: lm::LogDestination::TMA(), program_id: program_id }
    }
}

impl sf::IObject for Logger {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        ipc_server_make_command_table! {
            log: 0,
            set_destination: 1
        }
    }
}

impl ILogger for Logger {
    fn log(&mut self, log_buf: sf::InAutoSelectBuffer) -> Result<()> {
        logger::log_packet_buf(log_buf.buf, log_buf.size, self.program_id);
        Ok(())
    }

    fn set_destination(&mut self, log_destination: lm::LogDestination) -> Result<()> {
        // TODO: make use of log destination?
        self.log_destination = log_destination;
        Ok(())
    }
}

pub struct LogService {
    session: sf::Session
}

impl sf::IObject for LogService {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        ipc_server_make_command_table! {
            open_logger: 0
        }
    }
}

use crate::service;
use crate::service::pm;
use crate::service::pm::IInformationInterface;

impl ILogService for LogService {
    fn open_logger(&mut self, process_id: sf::ProcessId) -> Result<mem::Shared<dyn sf::IObject>> {
        let pminfo = service::new_service_object::<pm::InformationInterface>()?;
        let program_id = pminfo.get().get_program_id(process_id.process_id)?;
        
        Ok(mem::Shared::new(Logger::new(program_id)))
    }
}

impl server::IServerObject for LogService {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }
}

impl server::IService for LogService {
    fn get_name() -> &'static str {
        nul!("lm")
    }

    fn get_max_sesssions() -> i32 {
        42
    }
}