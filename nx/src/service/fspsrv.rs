use crate::service::*;
use crate::ipc;
use crate::alloc;
use crate::session_object_define;
use crate::ipc::client::*;
use crate::ipc_client_session_send_request_command;

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
    fn open_sd_card_filesystem<S: IFileSystem + SessionObject>(&mut self) -> Result<S>;
}

session_object_define!(FileSystemProxy);

impl Service for FileSystemProxy {
    fn get_name() -> &'static str {
        "fsp-srv"
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

    fn open_sd_card_filesystem<S: IFileSystem + SessionObject>(&mut self) -> Result<S> {
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
}