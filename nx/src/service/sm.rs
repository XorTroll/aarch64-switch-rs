use crate::result::*;
use crate::ipc;
use crate::service;
use crate::service::SessionObject;

pub union ServiceName {
    name: [u8; 8],
    value: u64,
}

impl ServiceName {
    pub fn new(name: &str) -> Self {
        let bytes = name.as_bytes();
        // TODO: less hacky version? this can even be contexpr in C++...
        Self { name: [*bytes.get(0).unwrap_or(&0), *bytes.get(1).unwrap_or(&0), *bytes.get(2).unwrap_or(&0), *bytes.get(3).unwrap_or(&0), *bytes.get(4).unwrap_or(&0), *bytes.get(5).unwrap_or(&0), *bytes.get(6).unwrap_or(&0), *bytes.get(7).unwrap_or(&0)] }
    }

    pub fn encode(&self) -> u64 {
        unsafe {
            self.value
        }
    }
}

pub trait IUserInterface {
    fn initialize(&mut self) -> Result<()>;
    fn get_service(&mut self, name: ServiceName) -> Result<ipc::Session>;
}

session_object_define!(UserInterface);

impl service::NamedPort for UserInterface {
    fn get_name() -> &'static str {
        "sm:"
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
                service_name: u64 = name.encode()
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