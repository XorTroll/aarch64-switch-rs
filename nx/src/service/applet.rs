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

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum ScreenShotPermission {
    Inherit,
    Enable,
    Disable
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum AppletId {
    Application = 0x1,
    OverlayDisp = 0x2,
    Qlaunch = 0x3,
    Starter = 0x4,
    Auth = 0xA,
    Cabinet = 0xB,
    Controller = 0xC,
    DataErase = 0xD,
    Error = 0xE,
    NetConnect = 0xF,
    PlayerSelect = 0x10,
    Swkbd = 0x11,
    MiiEdit = 0x12,
    Web = 0x13,
    Shop = 0x14,
    PhotoViewer = 0x15,
    Set = 0x16,
    OfflineWeb = 0x17,
    LoginShare = 0x18,
    WifiWebAuth = 0x19,
    MyPage = 0x1A,
    // TODO: add non-retail IDs too?
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum LibraryAppletMode {
    AllForeground,
    Background,
    NoUi,
    BackgroundIndirectDisplay,
    AllForegroundInitiallyHidden,
}

pub trait IStorageAccessor {
    fn get_size(&mut self) -> Result<usize>;
    fn write(&mut self, offset: usize, buf: *const u8, buf_size: usize) -> Result<()>;
    fn read(&mut self, offset: usize, buf: *const u8, buf_size: usize) -> Result<()>;
}

session_object_define!(StorageAccessor);

impl IStorageAccessor for StorageAccessor {
    fn get_size(&mut self) -> Result<usize> {
        let size: usize;
        ipc_client_session_send_request_command!([self.session; 0; false] => {
            In {};
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {};
            Out {
                size: usize => size
            };
            OutHandles {};
            OutObjects {};
            OutSessions {};
        });
        Ok(size)
    }

    fn write(&mut self, offset: usize, buf: *const u8, buf_size: usize) -> Result<()> {
        ipc_client_session_send_request_command!([self.session; 10; false] => {
            In {
                offset: usize = offset
            };
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {
                (buf, buf_size) => ipc::BufferAttribute::In | ipc::BufferAttribute::AutoSelect
            };
            Out {};
            OutHandles {};
            OutObjects {};
            OutSessions {};
        });
        Ok(())
    }

    fn read(&mut self, offset: usize, buf: *const u8, buf_size: usize) -> Result<()> {
        ipc_client_session_send_request_command!([self.session; 11; false] => {
            In {
                offset: usize = offset
            };
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {
                (buf, buf_size) => ipc::BufferAttribute::Out | ipc::BufferAttribute::AutoSelect
            };
            Out {};
            OutHandles {};
            OutObjects {};
            OutSessions {};
        });
        Ok(())
    }
}

pub trait IStorage {
    fn open<S: service::SessionObject>(&mut self) -> Result<S>;
}

session_object_define!(Storage);

impl IStorage for Storage {
    fn open<S: service::SessionObject>(&mut self) -> Result<S> {
        let storage_accessor: ipc::Session;
        ipc_client_session_send_request_command!([self.session; 0; false] => {
            In {};
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {};
            Out {};
            OutHandles {};
            OutObjects {};
            OutSessions {
                storage_accessor
            };
        });
        Ok(S::new(storage_accessor))
    }
}

pub trait ILibraryAppletAccessor {
    fn get_applet_state_changed_event(&mut self) -> Result<svc::Handle>;
    fn start(&mut self) -> Result<()>;
    fn push_in_data<S: service::SessionObject>(&mut self, storage: &S) -> Result<()>;
}

session_object_define!(LibraryAppletAccessor);

impl ILibraryAppletAccessor for LibraryAppletAccessor {
    fn get_applet_state_changed_event(&mut self) -> Result<svc::Handle> {
        let applet_state_changed_event: svc::Handle;
        ipc_client_session_send_request_command!([self.session; 0; false] => {
            In {};
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {};
            Out {};
            OutHandles {
                applet_state_changed_event => ipc::HandleMode::Copy
            };
            OutObjects {};
            OutSessions {};
        });
        Ok(applet_state_changed_event)
    }

    fn start(&mut self) -> Result<()> {
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

    fn push_in_data<S: service::SessionObject>(&mut self, storage: &S) -> Result<()> {
        ipc_client_session_send_request_command!([self.session; 100; false] => {
            In {};
            InHandles {};
            InObjects {};
            InSessions {
                storage.get_session()
            };
            Buffers {};
            Out {};
            OutHandles {};
            OutObjects {};
            OutSessions {};
        });
        Ok(())
    }
}

pub trait ILibraryAppletCreator {
    fn create_library_applet<S: service::SessionObject>(&mut self, id: AppletId, mode: LibraryAppletMode) -> Result<S>;
    fn create_storage<S: service::SessionObject>(&mut self, size: usize) -> Result<S>;
}

session_object_define!(LibraryAppletCreator);

impl ILibraryAppletCreator for LibraryAppletCreator {
    fn create_library_applet<S: service::SessionObject>(&mut self, id: AppletId, mode: LibraryAppletMode) -> Result<S> {
        let library_applet_accessor: ipc::Session;
        ipc_client_session_send_request_command!([self.session; 0; false] => {
            In {
                id: AppletId = id,
                mode: LibraryAppletMode = mode
            };
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {};
            Out {};
            OutHandles {};
            OutObjects {};
            OutSessions {
                library_applet_accessor
            };
        });
        Ok(S::new(library_applet_accessor))
    }

    fn create_storage<S: service::SessionObject>(&mut self, size: usize) -> Result<S> {
        let storage: ipc::Session;
        ipc_client_session_send_request_command!([self.session; 10; false] => {
            In {
                size: usize = size
            };
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {};
            Out {};
            OutHandles {};
            OutObjects {};
            OutSessions {
                storage
            };
        });
        Ok(S::new(storage))
    }
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
    fn get_library_applet_creator<S: service::SessionObject>(&mut self) -> Result<S>;
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
        let window_controller: ipc::Session;
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
                window_controller
            };
        });
        Ok(S::new(window_controller))
    }

    fn get_library_applet_creator<S: service::SessionObject>(&mut self) -> Result<S> {
        let library_applet_creator: ipc::Session;
        ipc_client_session_send_request_command!([self.session; 11; false] => {
            In {};
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {};
            Out {};
            OutHandles {};
            OutObjects {};
            OutSessions {
                library_applet_creator
            };
        });
        Ok(S::new(library_applet_creator))
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