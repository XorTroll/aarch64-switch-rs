use crate::result::*;
use crate::ipc::sf;
use crate::ipc::server;
use crate::service;

pub use crate::ipc::sf::fatal::*;

pub struct Service {
    session: service::Session
}

impl service::ISessionObject for Service {
    fn new(session: service::Session) -> Self {
        Self { session: session }
    }
    
    fn get_session(&mut self) -> &mut service::Session {
        &mut self.session
    }
}

impl IService for Service {
    fn throw_with_policy(&mut self, rc: ResultCode, policy: Policy, process_id: sf::ProcessId) -> Result<()> {
        ipc_client_send_request_command!([self.session.session; 1] (rc, policy, process_id) => ())
    }
}

impl server::IServer for Service {
    fn get_command_table(&self) -> server::CommandMetadataTable {
        ipc_server_make_command_table! {
            throw_with_policy: 1
        }
    }
}

impl service::IService for Service {
    fn get_name() -> &'static str {
        nul!("fatal:u")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}