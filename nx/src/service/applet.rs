use crate::result::*;
use crate::ipc::sf;
use crate::ipc::server;
use crate::service;
use crate::mem;

pub use crate::ipc::sf::applet::*;

pub struct StorageAccessor {
    session: service::Session
}

impl service::ISessionObject for StorageAccessor {
    fn new(session: service::Session) -> Self {
        Self { session: session }
    }
    
    fn get_session(&mut self) -> &mut service::Session {
        &mut self.session
    }
}

impl IStorageAccessor for StorageAccessor {
    fn get_size(&mut self) -> Result<usize> {
        ipc_client_send_request_command!([self.session.session; 0] () => (size: usize))
    }

    fn write(&mut self, offset: usize, buf: sf::InAutoSelectBuffer) -> Result<()> {
        ipc_client_send_request_command!([self.session.session; 10] (offset, buf) => ())
    }

    fn read(&mut self, offset: usize, buf: sf::OutAutoSelectBuffer) -> Result<()> {
        ipc_client_send_request_command!([self.session.session; 11] (offset, buf) => ())
    }
}

impl server::IServer for StorageAccessor {
    fn get_command_table(&self) -> server::CommandMetadataTable {
        ipc_server_make_command_table! {
            get_size: 0,
            write: 10,
            read: 11
        }
    }
}

pub struct Storage {
    session: service::Session
}

impl service::ISessionObject for Storage {
    fn new(session: service::Session) -> Self {
        Self { session: session }
    }
    
    fn get_session(&mut self) -> &mut service::Session {
        &mut self.session
    }
}

impl IStorage for Storage {
    fn open(&mut self) -> Result<mem::Shared<dyn service::ISessionObject>> {
        ipc_client_send_request_command!([self.session.session; 0] () => (storage_accessor: mem::Shared<StorageAccessor>))
    }
}

impl server::IServer for Storage {
    fn get_command_table(&self) -> server::CommandMetadataTable {
        ipc_server_make_command_table! {
            open: 0
        }
    }
}

pub struct LibraryAppletAccessor {
    session: service::Session
}

impl service::ISessionObject for LibraryAppletAccessor {
    fn new(session: service::Session) -> Self {
        Self { session: session }
    }
    
    fn get_session(&mut self) -> &mut service::Session {
        &mut self.session
    }
}

impl ILibraryAppletAccessor for LibraryAppletAccessor {
    fn get_applet_state_changed_event(&mut self) -> Result<sf::CopyHandle> {
        ipc_client_send_request_command!([self.session.session; 0] () => (applet_state_changed_event: sf::CopyHandle))
    }

    fn start(&mut self) -> Result<()> {
        ipc_client_send_request_command!([self.session.session; 10] () => ())
    }

    fn push_in_data(&mut self, storage: mem::Shared<dyn service::ISessionObject>) -> Result<()> {
        ipc_client_send_request_command!([self.session.session; 100] (storage) => ())
    }
}

impl server::IServer for LibraryAppletAccessor {
    fn get_command_table(&self) -> server::CommandMetadataTable {
        ipc_server_make_command_table! {
            get_applet_state_changed_event: 0,
            start: 10,
            push_in_data: 100
        }
    }
}

pub struct LibraryAppletCreator {
    session: service::Session
}

impl service::ISessionObject for LibraryAppletCreator {
    fn new(session: service::Session) -> Self {
        Self { session: session }
    }
    
    fn get_session(&mut self) -> &mut service::Session {
        &mut self.session
    }
}

impl ILibraryAppletCreator for LibraryAppletCreator {
    fn create_library_applet(&mut self, id: AppletId, mode: LibraryAppletMode) -> Result<mem::Shared<dyn service::ISessionObject>> {
        ipc_client_send_request_command!([self.session.session; 0] (id, mode) => (library_applet_accessor: mem::Shared<LibraryAppletAccessor>))
    }

    fn create_storage(&mut self, size: usize) -> Result<mem::Shared<dyn service::ISessionObject>> {
        ipc_client_send_request_command!([self.session.session; 10] (size) => (storage: mem::Shared<Storage>))
    }
}

impl server::IServer for LibraryAppletCreator {
    fn get_command_table(&self) -> server::CommandMetadataTable {
        ipc_server_make_command_table! {
            create_library_applet: 0,
            create_storage: 10
        }
    }
}

pub struct WindowController {
    session: service::Session
}

impl service::ISessionObject for WindowController {
    fn new(session: service::Session) -> Self {
        Self { session: session }
    }
    
    fn get_session(&mut self) -> &mut service::Session {
        &mut self.session
    }
}

impl IWindowController for WindowController {
    fn acquire_foreground_rights(&mut self) -> Result<()> {
        ipc_client_send_request_command!([self.session.session; 10] () => ())
    }
}

impl server::IServer for WindowController {
    fn get_command_table(&self) -> server::CommandMetadataTable {
        ipc_server_make_command_table! {
            acquire_foreground_rights: 10
        }
    }
}

pub struct SelfController {
    session: service::Session
}

impl service::ISessionObject for SelfController {
    fn new(session: service::Session) -> Self {
        Self { session: session }
    }
    
    fn get_session(&mut self) -> &mut service::Session {
        &mut self.session
    }
}

impl ISelfController for SelfController {
    fn set_screenshot_permission(&mut self, permission: ScreenShotPermission) -> Result<()> {
        ipc_client_send_request_command!([self.session.session; 10] (permission) => ())
    }
}

impl server::IServer for SelfController {
    fn get_command_table(&self) -> server::CommandMetadataTable {
        ipc_server_make_command_table! {
            set_screenshot_permission: 10
        }
    }
}

pub struct LibraryAppletProxy {
    session: service::Session
}

impl service::ISessionObject for LibraryAppletProxy {
    fn new(session: service::Session) -> Self {
        Self { session: session }
    }
    
    fn get_session(&mut self) -> &mut service::Session {
        &mut self.session
    }
}

impl ILibraryAppletProxy for LibraryAppletProxy {
    fn get_self_controller(&mut self) -> Result<mem::Shared<dyn service::ISessionObject>> {
        ipc_client_send_request_command!([self.session.session; 1] () => (self_controller: mem::Shared<SelfController>))
    }

    fn get_window_controller(&mut self) -> Result<mem::Shared<dyn service::ISessionObject>> {
        ipc_client_send_request_command!([self.session.session; 2] () => (window_controller: mem::Shared<WindowController>))
    }

    fn get_library_applet_creator(&mut self) -> Result<mem::Shared<dyn service::ISessionObject>> {
        ipc_client_send_request_command!([self.session.session; 11] () => (library_applet_creator: mem::Shared<LibraryAppletCreator>))
    }
}

impl server::IServer for LibraryAppletProxy {
    fn get_command_table(&self) -> server::CommandMetadataTable {
        ipc_server_make_command_table! {
            get_self_controller: 1,
            get_window_controller: 2,
            get_library_applet_creator: 11
        }
    }
}

pub struct AllSystemAppletProxiesService {
    session: service::Session
}

impl service::ISessionObject for AllSystemAppletProxiesService {
    fn new(session: service::Session) -> Self {
        Self { session: session }
    }
    
    fn get_session(&mut self) -> &mut service::Session {
        &mut self.session
    }
}

impl IAllSystemAppletProxiesService for AllSystemAppletProxiesService {
    fn open_library_applet_proxy(&mut self, process_id: sf::ProcessId, self_process_handle: sf::CopyHandle, applet_attribute: sf::InMapAliasBuffer) -> Result<mem::Shared<dyn service::ISessionObject>> {
        ipc_client_send_request_command!([self.session.session; 201] (process_id, self_process_handle, applet_attribute) => (library_applet_proxy: mem::Shared<LibraryAppletProxy>))
    }
}

impl server::IServer for AllSystemAppletProxiesService {
    fn get_command_table(&self) -> server::CommandMetadataTable {
        ipc_server_make_command_table! {
            open_library_applet_proxy: 201
        }
    }
}

impl service::IService for AllSystemAppletProxiesService {
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