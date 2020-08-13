use crate::result::*;
use crate::ipc;
use crate::svc;
use crate::service;
use crate::service::SessionObject;
use crate::input;
use enumflags2::BitFlags;

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

    pub const fn empty() -> Self {
        Self { value: 0 }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct MitmProcessInfo {
    pub process_id: u64,
    pub program_id: u64,
    pub keys_held: BitFlags<input::Key>,
    pub override_flags: u64
}

pub trait IUserInterface {
    fn initialize(&mut self) -> Result<()>;
    fn get_service(&mut self, name: ServiceName) -> Result<ipc::Session>;
    fn register_service(&mut self, name: ServiceName, is_light: bool, max_sessions: i32) -> Result<svc::Handle>;
    fn atmosphere_install_mitm(&mut self, name: ServiceName) -> Result<(svc::Handle, svc::Handle)>;
    fn atmosphere_uninstall_mitm(&mut self, name: ServiceName) -> Result<()>;
    fn atmosphere_acknowledge_mitm_session(&mut self, name: ServiceName) -> Result<(MitmProcessInfo, svc::Handle)>;
    fn atmosphere_has_service(&mut self, name: ServiceName) -> Result<bool>;
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

    fn register_service(&mut self, name: ServiceName, is_light: bool, max_sessions: i32) -> Result<svc::Handle> {
        let handle: svc::Handle;
        ipc_client_session_send_request_command!([self.session; 2; false] => {
            In {
                service_name: u64 = name.value,
                is_light: bool = is_light,
                max_sessions: i32 = max_sessions
            };
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {};
            Out {};
            OutHandles {
                handle => ipc::HandleMode::Move
            };
            OutObjects {};
            OutSessions {};
        });
        Ok(handle)
    }

    fn atmosphere_install_mitm(&mut self, name: ServiceName) -> Result<(svc::Handle, svc::Handle)> {
        let mitm_handle: svc::Handle;
        let query_handle: svc::Handle;
        ipc_client_session_send_request_command!([self.session; 65000; false] => {
            In {
                service_name: u64 = name.value
            };
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {};
            Out {};
            OutHandles {
                mitm_handle => ipc::HandleMode::Move,
                query_handle => ipc::HandleMode::Move
            };
            OutObjects {};
            OutSessions {};
        });
        Ok((mitm_handle, query_handle))
    }

    fn atmosphere_uninstall_mitm(&mut self, name: ServiceName) -> Result<()> {
        ipc_client_session_send_request_command!([self.session; 65001; false] => {
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
            OutSessions {};
        });
        Ok(())
    }
    fn atmosphere_acknowledge_mitm_session(&mut self, name: ServiceName) -> Result<(MitmProcessInfo, svc::Handle)> {
        let info: MitmProcessInfo;
        let handle: svc::Handle;
        ipc_client_session_send_request_command!([self.session; 65003; false] => {
            In {
                service_name: u64 = name.value
            };
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {};
            Out {
                info: MitmProcessInfo => info
            };
            OutHandles {
                handle => ipc::HandleMode::Move
            };
            OutObjects {};
            OutSessions {};
        });
        Ok((info, handle))
    }

    fn atmosphere_has_service(&mut self, name: ServiceName) -> Result<bool> {
        let has: bool;
        ipc_client_session_send_request_command!([self.session; 65100; false] => {
            In {
                service_name: u64 = name.value
            };
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {};
            Out {
                has: bool => has
            };
            OutHandles {};
            OutObjects {};
            OutSessions {};
        });
        Ok(has)
    }
}