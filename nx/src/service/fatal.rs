use crate::result::*;
use crate::service;
use crate::service::SessionObject;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum Policy {
    ErrorReportAndErrorScreen,
    ErrorReport,
    ErrorScreen,
}

pub trait IService {
    fn throw_with_policy(&mut self, rc: ResultCode, policy: Policy) -> Result<()>;
}

session_object_define!(Service);

impl service::Service for Service {
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

impl IService for Service {
    fn throw_with_policy(&mut self, rc: ResultCode, policy: Policy) -> Result<()> {
        ipc_client_session_send_request_command!([self.session; 1; true] => {
            In {
                rc: ResultCode = rc,
                policy: Policy = policy,
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
}