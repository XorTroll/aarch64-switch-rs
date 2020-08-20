use crate::result::*;
use crate::ipc::sf;
use crate::service;
use crate::mem;

pub use crate::ipc::sf::fspsrv::*;

pub struct FileSystem {
    session: sf::Session
}

impl sf::IObject for FileSystem {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        ipc_server_make_command_table! {
            create_directory: 2
        }
    }
}

impl service::IClientObject for FileSystem {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }
}

impl IFileSystem for FileSystem {
    fn create_directory(&mut self, path: sf::InPointerBuffer) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 2] (path) => ())
    }
}

pub struct FileSystemProxy {
    session: sf::Session
}

impl sf::IObject for FileSystemProxy {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        ipc_server_make_command_table! {
            set_current_process: 1,
            open_sd_card_filesystem: 18,
            output_access_log_to_sd_card: 1006
        }
    }
}

impl service::IClientObject for FileSystemProxy {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }
}

impl IFileSystemProxy for FileSystemProxy {
    fn set_current_process(&mut self, process_id: sf::ProcessId) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 1] (process_id) => ())
    }

    fn open_sd_card_filesystem(&mut self) -> Result<mem::Shared<dyn sf::IObject>> {
        ipc_client_send_request_command!([self.session.object_info; 18] () => (sd_filesystem: mem::Shared<FileSystem>))
    }

    fn output_access_log_to_sd_card(&mut self, access_log: sf::InMapAliasBuffer) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 1006] (access_log) => ())
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