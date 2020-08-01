use crate::result::*;
use crate::service;
use crate::service::SessionObject;

pub trait IPsmServer {
    fn get_battery_charge_percentage(&mut self) -> Result<u32>;
}

session_object_define!(PsmServer);

impl service::Service for PsmServer {
    fn get_name() -> &'static str {
        "psm"
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}

impl IPsmServer for PsmServer {
    fn get_battery_charge_percentage(&mut self) -> Result<u32> {
        let charge: u32;
        ipc_client_session_send_request_command!([self.session; 0; false] => {
            In {};
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {};
            Out {
                charge: u32 => charge
            };
            OutHandles {};
            OutObjects {};
            OutSessions {};
        });
        Ok(charge)
    }
}