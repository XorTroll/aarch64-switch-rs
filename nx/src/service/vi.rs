use crate::result::*;
use crate::ipc::sf;
use crate::ipc::server;
use crate::service;
use crate::mem;
use crate::service::dispdrv;

pub use crate::ipc::sf::vi::*;

pub struct ApplicationDisplayService {
    session: service::Session
}

impl service::ISessionObject for ApplicationDisplayService {
    fn new(session: service::Session) -> Self {
        Self { session: session }
    }
    
    fn get_session(&mut self) -> &mut service::Session {
        &mut self.session
    }
}

impl IApplicationDisplayService for ApplicationDisplayService {
    fn get_relay_service(&mut self) -> Result<mem::Shared<dyn service::ISessionObject>> {
        ipc_client_send_request_command!([self.session.session; 100] () => (relay_service: mem::Shared<dispdrv::HOSBinderDriver>))
    }

    fn open_display(&mut self, name: DisplayName) -> Result<DisplayId> {
        ipc_client_send_request_command!([self.session.session; 1010] (name) => (id: DisplayId))
    }

    fn close_display(&mut self, display_id: DisplayId) -> Result<()> {
        ipc_client_send_request_command!([self.session.session; 1020] (display_id) => ())
    }

    fn open_layer(&mut self, name: DisplayName, id: LayerId, aruid: sf::ProcessId, out_native_window: sf::OutMapAliasBuffer) -> Result<usize> {
        ipc_client_send_request_command!([self.session.session; 2020] (name, id, aruid, out_native_window) => (native_window_size: usize))
    }

    fn create_stray_layer(&mut self, flags: LayerFlags, display_id: DisplayId, out_native_window: sf::OutMapAliasBuffer) -> Result<(LayerId, usize)> {
        ipc_client_send_request_command!([self.session.session; 2030] (flags, display_id, out_native_window) => (id: LayerId, native_window_size: usize))
    }

    fn destroy_stray_layer(&mut self, id: LayerId) -> Result<()> {
        ipc_client_send_request_command!([self.session.session; 2031] (id) => ())
    }

    fn get_display_vsync_event(&mut self, display_id: DisplayId) -> Result<sf::CopyHandle> {
        ipc_client_send_request_command!([self.session.session; 5202] (display_id) => (event_handle: sf::CopyHandle))
    }
}

impl server::IServer for ApplicationDisplayService {
    fn get_command_table(&self) -> server::CommandMetadataTable {
        ipc_server_make_command_table! {
            get_relay_service: 100,
            open_display: 1010,
            close_display: 1020,
            open_layer: 2020,
            create_stray_layer: 2030,
            destroy_stray_layer: 2031,
            get_display_vsync_event: 5202
        }
    }
}

pub struct SystemRootService {
    session: service::Session
}

impl service::ISessionObject for SystemRootService {
    fn new(session: service::Session) -> Self {
        Self { session: session }
    }
    
    fn get_session(&mut self) -> &mut service::Session {
        &mut self.session
    }
}

impl IRootService for SystemRootService {
    fn get_display_service(&mut self, mode: DisplayServiceMode) -> Result<mem::Shared<dyn service::ISessionObject>> {
        ipc_client_send_request_command!([self.session.session; 1] (mode) => (display_service: mem::Shared<ApplicationDisplayService>))
    }
}

impl server::IServer for SystemRootService {
    fn get_command_table(&self) -> server::CommandMetadataTable {
        ipc_server_make_command_table! {
            get_display_service: 1
        }
    }
}

impl service::IService for SystemRootService {
    fn get_name() -> &'static str {
        nul!("vi:s")
    }

    fn as_domain() -> bool {
        true
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}