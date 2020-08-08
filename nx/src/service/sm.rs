use crate::result::*;
use crate::ipc;
use crate::service;
use crate::service::SessionObject;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ServiceName {
    pub value: u64,
}

impl ServiceName {
    pub const fn new(name: &str) -> Self {
        let value = unsafe { *(name.as_ptr() as *const u64) };
        Self { value: value }
    }
}

pub trait IUserInterface {
    fn initialize(&mut self) -> Result<()>;
    fn get_service(&mut self, name: ServiceName) -> Result<ipc::Session>;
}

session_object_define!(UserInterface);

impl service::NamedPort for UserInterface {
    fn get_name() -> &'static str {
        nul!("sm:")
    }

    fn post_initialize(&mut self) -> Result<()> {
        self.initialize()
    }
}

impl IUserInterface for UserInterface {
    fn initialize(&mut self) -> Result<()> {
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
            OutSessions {};
        });
        Ok(())
    }

    fn get_service(&mut self, name: ServiceName) -> Result<ipc::Session> {
        let session: ipc::Session;
        ipc_client_session_send_request_command!([self.session; 1; false] => {
            In {
                service_name: u64 = name.value
            };
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {};
            Out {};
            OutHandles {};
            OutObjects {};
            OutSessions {
                session
            };
        });
        Ok(session)
    }
}