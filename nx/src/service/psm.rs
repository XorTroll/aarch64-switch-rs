use crate::result::*;
// use crate::ipc::sf;
use crate::ipc::server;
use crate::service;

pub use crate::ipc::sf::psm::*;

pub struct PsmServer {
    session: service::Session
}

impl service::ISessionObject for PsmServer {
    fn new(session: service::Session) -> Self {
        Self { session: session }
    }
    
    fn get_session(&mut self) -> &mut service::Session {
        &mut self.session
    }
}

impl IPsmServer for PsmServer {
    fn get_battery_charge_percentage(&mut self) -> Result<u32> {
        ipc_client_send_request_command!([self.session.session; 0] () => (charge: u32))
    }
}

impl server::IServer for PsmServer {
    fn get_command_table(&self) -> server::CommandMetadataTable {
        ipc_server_make_command_table! {
            get_battery_charge_percentage: 0
        }
    }
}

impl service::IService for PsmServer {
    fn get_name() -> &'static str {
        nul!("psm")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}