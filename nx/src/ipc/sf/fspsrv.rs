use crate::result::*;
use crate::ipc::sf;
use crate::mem;
use crate::util;

bit_enum! {
    FileOpenMode (u32) {
        Read = bit!(0),
        Write = bit!(1),
        Append = bit!(2)
    }
}

bit_enum! {
    FileAttribute (u32) {
        None = 0,
        ConcatenationFile = bit!(0)
    }
}

bit_enum! {
    FileReadOption (u32) {
        None = 0
    }
}

bit_enum! {
    FileWriteOption (u32) {
        None = 0,
        Flush = bit!(0)
    }
}

pub struct Path {
    pub path: [u8; 0x301]
}

impl Path {
    pub fn from(path: &str) -> Result<Self> {
        let mut path_var = Self { path: [0; 0x301] };
        util::copy_str_to_pointer(path, &mut path_var.path as *mut _ as *mut u8)?;
        Ok(path_var)
    }
}

pub trait IFile {
    ipc_interface_define_command!(read: (option: FileReadOption, offset: usize, size: usize, buf: sf::OutNonSecureMapAliasBuffer) => (read_size: usize));
    ipc_interface_define_command!(write: (option: FileWriteOption, offset: usize, size: usize, buf: sf::InNonSecureMapAliasBuffer) => ());
    ipc_interface_define_command!(get_size: () => (size: usize));
}

pub trait IFileSystem {
    ipc_interface_define_command!(create_file: (attribute: FileAttribute, size: usize, path_buf: sf::InPointerBuffer) => ());
    ipc_interface_define_command!(delete_file: (path_buf: sf::InPointerBuffer) => ());
    ipc_interface_define_command!(create_directory: (path_buf: sf::InPointerBuffer) => ());
    ipc_interface_define_command!(delete_directory: (path_buf: sf::InPointerBuffer) => ());
    ipc_interface_define_command!(delete_directory_recursively: (path_buf: sf::InPointerBuffer) => ());
    ipc_interface_define_command!(open_file: (mode: FileOpenMode, path_buf: sf::InPointerBuffer) => (file: mem::Shared<dyn sf::IObject>));
}

pub trait IFileSystemProxy {
    ipc_interface_define_command!(set_current_process: (process_id: sf::ProcessId) => ());
    ipc_interface_define_command!(open_sd_card_filesystem: () => (sd_filesystem: mem::Shared<dyn sf::IObject>));
    ipc_interface_define_command!(output_access_log_to_sd_card: (access_log: sf::InMapAliasBuffer) => ());
}