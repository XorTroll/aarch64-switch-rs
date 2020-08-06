use crate::result::*;
use crate::ipc;
use crate::service;
use crate::service::SessionObject;

pub trait IFileSystem {
    fn create_directory(&mut self, path: *const u8, path_len: usize) -> Result<()>;
}

session_object_define!(FileSystem);

impl IFileSystem for FileSystem {
    fn create_directory(&mut self, path: *const u8, path_len: usize) -> Result<()> {
        ipc_client_session_send_request_command!([self.session; 2; false] => {
            In {};
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {
                (path, path_len) => ipc::BufferAttribute::In | ipc::BufferAttribute::Pointer
            };
            Out {};
            OutHandles {};
            OutObjects {};
            OutSessions {};
        });
        Ok(())
    }
}

pub trait IFileSystemProxy {
    fn set_current_process(&mut self) -> Result<()>;

    fn open_sd_card_filesystem<S: service::SessionObject>(&mut self) -> Result<S>;

    fn output_access_log_to_sd_card(&mut self, buf: *const u8, buf_size: usize) -> Result<()>;
}

session_object_define!(FileSystemProxy);

impl service::Service for FileSystemProxy {
    fn get_name() -> &'static str {
        nul!("fsp-srv")
    }

    fn as_domain() -> bool {
        true
    }

    fn post_initialize(&mut self) -> Result<()> {
        self.set_current_process()
    }
}

impl IFileSystemProxy for FileSystemProxy {
    fn set_current_process(&mut self) -> Result<()> {
        ipc_client_session_send_request_command!([self.session; 1; true] => {
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

    fn open_sd_card_filesystem<S: service::SessionObject>(&mut self) -> Result<S> {
        let fs: ipc::Session;
        ipc_client_session_send_request_command!([self.session; 18; false] => {
            In {};
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {};
            Out {};
            OutHandles {};
            OutObjects {};
            OutSessions {
                fs
            };
        });
        Ok(S::new(fs))
    }

    fn output_access_log_to_sd_card(&mut self, buf: *const u8, buf_size: usize) -> Result<()> {
        ipc_client_session_send_request_command!([self.session; 1006; false] => {
            In {};
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {
                (buf, buf_size) => ipc::BufferAttribute::In | ipc::BufferAttribute::MapAlias
            };
            Out {};
            OutHandles {};
            OutObjects {};
            OutSessions {};
        });
        Ok(())
    }
}