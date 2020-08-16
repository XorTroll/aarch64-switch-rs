use crate::result::*;
use crate::ipc::sf;
use crate::ipc::server;
use crate::service;
use crate::mem;

pub use crate::ipc::sf::hid::*;

pub struct AppletResource {
    session: service::Session
}

impl service::ISessionObject for AppletResource {
    fn new(session: service::Session) -> Self {
        Self { session: session }
    }
    
    fn get_session(&mut self) -> &mut service::Session {
        &mut self.session
    }
}

impl IAppletResource for AppletResource {
    fn get_shared_memory_handle(&mut self) -> Result<sf::CopyHandle> {
        ipc_client_send_request_command!([self.session.session; 0] () => (shmem_handle: sf::CopyHandle))
    }
}

impl server::IServer for AppletResource {
    fn get_command_table(&self) -> server::CommandMetadataTable {
        ipc_server_make_command_table! {
            get_shared_memory_handle: 0
        }
    }
}

pub struct HidServer {
    session: service::Session
}

impl service::ISessionObject for HidServer {
    fn new(session: service::Session) -> Self {
        Self { session: session }
    }
    
    fn get_session(&mut self) -> &mut service::Session {
        &mut self.session
    }
}

impl IHidServer for HidServer {
    fn create_applet_resource(&mut self, aruid: sf::ProcessId) -> Result<mem::Shared<dyn service::ISessionObject>> {
        ipc_client_send_request_command!([self.session.session; 0] (aruid) => (applet_resource: mem::Shared<AppletResource>))
    }

    fn set_supported_npad_style_set(&mut self, aruid: sf::ProcessId, npad_style_tag: NpadStyleTag) -> Result<()> {
        ipc_client_send_request_command!([self.session.session; 100] (npad_style_tag, aruid) => ())
    }

    fn set_supported_npad_id_type(&mut self, aruid: sf::ProcessId, controllers: sf::InPointerBuffer) -> Result<()> {
        ipc_client_send_request_command!([self.session.session; 102] (aruid, controllers) => ())
    }

    fn activate_npad(&mut self, aruid: sf::ProcessId) -> Result<()> {
        ipc_client_send_request_command!([self.session.session; 103] (aruid) => ())
    }

    fn deactivate_npad(&mut self, aruid: sf::ProcessId) -> Result<()> {
        ipc_client_send_request_command!([self.session.session; 104] (aruid) => ())
    }

    fn set_npad_joy_assignment_mode_single(&mut self, aruid: sf::ProcessId, controller: ControllerId, joy_type: NpadJoyDeviceType) -> Result<()> {
        ipc_client_send_request_command!([self.session.session; 123] (controller, aruid, joy_type) => ())
    }

    fn set_npad_joy_assignment_mode_dual(&mut self, aruid: sf::ProcessId, controller: ControllerId) -> Result<()> {
        ipc_client_send_request_command!([self.session.session; 124] (controller, aruid) => ())
    }
}

impl server::IServer for HidServer {
    fn get_command_table(&self) -> server::CommandMetadataTable {
        ipc_server_make_command_table! {
            create_applet_resource: 0,
            set_supported_npad_style_set: 100,
            set_supported_npad_id_type: 102,
            activate_npad: 103,
            deactivate_npad: 104,
            set_npad_joy_assignment_mode_single: 123,
            set_npad_joy_assignment_mode_dual: 124
        }
    }
}

impl service::IService for HidServer {
    fn get_name() -> &'static str {
        nul!("hid")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}