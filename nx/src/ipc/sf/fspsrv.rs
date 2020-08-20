use crate::result::*;
use crate::ipc::sf;
use crate::mem;

pub trait IFileSystem {
    ipc_interface_define_command!(create_directory: (path: sf::InPointerBuffer) => ());
}

pub trait IFileSystemProxy {
    ipc_interface_define_command!(set_current_process: (process_id: sf::ProcessId) => ());
    ipc_interface_define_command!(open_sd_card_filesystem: () => (sd_filesystem: mem::Shared<dyn sf::IObject>));
    ipc_interface_define_command!(output_access_log_to_sd_card: (access_log: sf::InMapAliasBuffer) => ());
}