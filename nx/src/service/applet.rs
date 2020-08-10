use crate::result::*;
use crate::ipc;
use crate::svc;
use crate::service;
use crate::service::SessionObject;
use core::mem;

pub type AppletResourceUserId = u64;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct AppletAttribute {
    flag: u8,
    reserved: [u8; 0x7F]
}

#[derive(Copy, Clone, PartialEq)]
#[repr(u32)]
pub enum ScreenShotPermission {
    Inherit,
    Enable,
    Disable
}

pub trait IWindowController {
    fn acquire_foreground_rights(&mut self) -> Result<()>;
}

session_object_define!(WindowController);

impl IWindowController for WindowController {
    fn acquire_foreground_rights(&mut self) -> Result<()> {
        ipc_client_session_send_request_command!([self.session; 10; false] => {
            In {};
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

pub trait ISelfController {
    fn set_screenshot_permission(&mut self, permission: ScreenShotPermission) -> Result<()>;
}

session_object_define!(SelfController);

impl ISelfController for SelfController {
    fn set_screenshot_permission(&mut self, permission: ScreenShotPermission) -> Result<()> {
        ipc_client_session_send_request_command!([self.session; 10; false] => {
            In {
                permission: ScreenShotPermission = permission
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

pub trait ILibraryAppletProxy {
    fn get_self_controller<S: service::SessionObject>(&mut self) -> Result<S>;
    fn get_window_controller<S: service::SessionObject>(&mut self) -> Result<S>;
}

session_object_define!(LibraryAppletProxy);

impl ILibraryAppletProxy for LibraryAppletProxy {
    fn get_self_controller<S: service::SessionObject>(&mut self) -> Result<S> {
        let self_controller: ipc::Session;
        ipc_client_session_send_request_command!([self.session; 1; false] => {
            In {};
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {};
            Out {};
            OutHandles {};
            OutObjects {};
            OutSessions {
                self_controller
            };
        });
        Ok(S::new(self_controller))
    }

    fn get_window_controller<S: service::SessionObject>(&mut self) -> Result<S> {
        let self_controller: ipc::Session;
        ipc_client_session_send_request_command!([self.session; 2; false] => {
            In {};
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {};
            Out {};
            OutHandles {};
            OutObjects {};
            OutSessions {
                self_controller
            };
        });
        Ok(S::new(self_controller))
    }
}

pub trait IAllSystemAppletProxiesService {
    fn open_library_applet_proxy<S: service::SessionObject>(&mut self, attr: AppletAttribute) -> Result<S>;
}

session_object_define!(AllSystemAppletProxiesService);

impl service::Service for AllSystemAppletProxiesService {
    fn get_name() -> &'static str {
        nul!("appletAE")
    }

    fn as_domain() -> bool {
        true
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}

impl IAllSystemAppletProxiesService for AllSystemAppletProxiesService {
    fn open_library_applet_proxy<S: service::SessionObject>(&mut self, attr: AppletAttribute) -> Result<S> {
        let library_applet_proxy: ipc::Session;
        ipc_client_session_send_request_command!([self.session; 201; true] => {
            In {
                process_id_holder: u64 = 0
            };
            InHandles {
                svc::CURRENT_PROCESS_PSEUDO_HANDLE => ipc::HandleMode::Copy
            };
            InObjects {};
            InSessions {};
            Buffers {
                (&attr as *const _ as *const u8, mem::size_of::<AppletAttribute>()) => ipc::BufferAttribute::In | ipc::BufferAttribute::MapAlias
            };
            Out {};
            OutHandles {};
            OutObjects {};
            OutSessions {
                library_applet_proxy
            };
        });
        Ok(S::new(library_applet_proxy))
    }
}