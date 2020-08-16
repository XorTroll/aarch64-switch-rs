use crate::result::*;
use crate::ipc::sf;
use crate::ipc::server;
use crate::service;
use crate::mem;

pub use crate::ipc::sf::fspsrv::*;

pub struct FileSystem {
    session: service::Session
}

impl service::ISessionObject for FileSystem {
    fn new(session: service::Session) -> Self {
        Self { session: session }
    }
    
    fn get_session(&mut self) -> &mut service::Session {
        &mut self.session
    }
}

impl IFileSystem for FileSystem {
    fn create_directory(&mut self, path: sf::InPointerBuffer) -> Result<()> {
        ipc_client_send_request_command!([self.session.session; 2] (path) => ())
    }
}

impl server::IServer for FileSystem {
    fn get_command_table(&self) -> server::CommandMetadataTable {
        ipc_server_make_command_table! {
            create_directory: 2
        }
    }
}

pub struct FileSystemProxy {
    session: service::Session
}

impl service::ISessionObject for FileSystemProxy {
    fn new(session: service::Session) -> Self {
        Self { session: session }
    }
    
    fn get_session(&mut self) -> &mut service::Session {
        &mut self.session
    }
}

impl IFileSystemProxy for FileSystemProxy {
    fn set_current_process(&mut self, process_id: sf::ProcessId) -> Result<()> {
        ipc_client_send_request_command!([self.session.session; 1] (process_id) => ())
    }

    fn open_sd_card_filesystem(&mut self) -> Result<mem::Shared<dyn service::ISessionObject>> {
        ipc_client_send_request_command!([self.session.session; 18] () => (sd_filesystem: mem::Shared<FileSystem>))
    }

    fn output_access_log_to_sd_card(&mut self, access_log: sf::InMapAliasBuffer) -> Result<()> {
        ipc_client_send_request_command!([self.session.session; 1006] (access_log) => ())
    }
}

impl server::IServer for FileSystemProxy {
    fn get_command_table(&self) -> server::CommandMetadataTable {
        ipc_server_make_command_table! {
            set_current_process: 1,
            open_sd_card_filesystem: 18,
            output_access_log_to_sd_card: 1006
        }
    }
}

impl service::IService for FileSystemProxy {
    fn get_name() -> &'static str {
        nul!("fsp-srv")
    }

    fn as_domain() -> bool {
        true
    }

    fn post_initialize(&mut self) -> Result<()> {
        self.set_current_process(sf::ProcessId::new())
    }
}