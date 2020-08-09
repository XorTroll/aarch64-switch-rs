use crate::result::*;
use crate::util;
use crate::svc;
use crate::ipc;
use crate::service;
use crate::service::SessionObject;
use enumflags2::BitFlags;

pub struct DisplayName {
    name: [u8; 0x40]
}

impl DisplayName {
    pub fn from(name: &str) -> Result<Self> {
        let mut display_name = Self { name: [0; 0x40] };
        util::copy_str_to_pointer(name, &mut display_name.name as *mut _ as *mut u8)?;
        Ok(display_name)
    }
}

#[derive(BitFlags, Copy, Clone, PartialEq, Debug)]
#[repr(u32)]
pub enum LayerFlags {
    Default = 0b1,
}

pub type DisplayId = u64;

pub type LayerId = u64;

pub trait IApplicationDisplayService {
    fn get_relay_service<S: SessionObject>(&mut self) -> Result<S>;
    fn open_display(&mut self, name: DisplayName) -> Result<DisplayId>;
    fn close_display(&mut self, display_id: DisplayId) -> Result<()>;
    fn open_layer(&mut self, name: DisplayName, layer_id: LayerId, aruid: u64, out_native_window_buf: *const u8, out_native_window_size: usize) -> Result<usize>;
    fn create_stray_layer(&mut self, flags: BitFlags<LayerFlags>, display_id: DisplayId, out_native_window_buf: *const u8, out_native_window_size: usize) -> Result<(LayerId, usize)>;
    fn destroy_stray_layer(&mut self, layer_id: LayerId) -> Result<()>;
    fn get_display_vsync_event(&mut self, display_id: DisplayId) -> Result<svc::Handle>;
}

session_object_define!(ApplicationDisplayService);

impl IApplicationDisplayService for ApplicationDisplayService {
    fn get_relay_service<S: SessionObject>(&mut self) -> Result<S> {
        let relay_srv: ipc::Session;
        ipc_client_session_send_request_command!([self.session; 100; false] => {
            In {};
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {};
            Out {};
            OutHandles {};
            OutObjects {};
            OutSessions {
                relay_srv
            };
        });
        Ok(S::new(relay_srv))
    }

    fn open_display(&mut self, name: DisplayName) -> Result<DisplayId> {
        let display_id: DisplayId;
        ipc_client_session_send_request_command!([self.session; 1010; false] => {
            In {
                display_name: DisplayName = name
            };
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {};
            Out {
                display_id: DisplayId => display_id
            };
            OutHandles {};
            OutObjects {};
            OutSessions {};
        });
        Ok(display_id)
    }

    fn close_display(&mut self, display_id: DisplayId) -> Result<()> {
        ipc_client_session_send_request_command!([self.session; 1020; false] => {
            In {
                display_id: DisplayId = display_id
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

    fn open_layer(&mut self, name: DisplayName, layer_id: LayerId, aruid: u64, out_native_window_buf: *const u8, out_native_window_size: usize) -> Result<usize> {
        let native_window_size: usize;
        ipc_client_session_send_request_command!([self.session; 2020; true] => {
            In {
                display_name: DisplayName = name,
                layer_id: LayerId = layer_id,
                aruid: u64 = aruid
            };
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {
                (out_native_window_buf, out_native_window_size) => ipc::BufferAttribute::Out | ipc::BufferAttribute::MapAlias
            };
            Out {
                native_window_size: usize => native_window_size
            };
            OutHandles {};
            OutObjects {};
            OutSessions {};
        });
        Ok(native_window_size)
    }

    fn create_stray_layer(&mut self, flags: BitFlags<LayerFlags>, display_id: DisplayId, out_native_window_buf: *const u8, out_native_window_size: usize) -> Result<(LayerId, usize)> {
        let layer_id: LayerId;
        let native_window_size: usize;
        ipc_client_session_send_request_command!([self.session; 2030; false] => {
            In {
                layer_flags: BitFlags<LayerFlags> = flags,
                pad: u32 = 0,
                display_id: DisplayId = display_id
            };
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {
                (out_native_window_buf, out_native_window_size) => ipc::BufferAttribute::Out | ipc::BufferAttribute::MapAlias
            };
            Out {
                layer_id: LayerId => layer_id,
                native_window_size: usize => native_window_size
            };
            OutHandles {};
            OutObjects {};
            OutSessions {};
        });
        Ok((layer_id, native_window_size))
    }

    fn destroy_stray_layer(&mut self, layer_id: LayerId) -> Result<()> {
        ipc_client_session_send_request_command!([self.session; 2031; false] => {
            In {
                layer_id: LayerId = layer_id
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

    fn get_display_vsync_event(&mut self, display_id: DisplayId) -> Result<svc::Handle> {
        let event_handle: svc::Handle;
        ipc_client_session_send_request_command!([self.session; 5202; false] => {
            In {
                display_id: DisplayId = display_id
            };
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {};
            Out {};
            OutHandles {
                event_handle => ipc::HandleMode::Copy
            };
            OutObjects {};
            OutSessions {};
        });
        Ok(event_handle)
    }
}

pub trait IRootService {
    fn get_display_service<S: SessionObject>(&mut self, is_privileged: bool) -> Result<S>;
}

session_object_define!(SystemRootService);

impl service::Service for SystemRootService {
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

impl IRootService for SystemRootService {
    fn get_display_service<S: SessionObject>(&mut self, is_privileged: bool) -> Result<S> {
        let display_srv: ipc::Session;
        ipc_client_session_send_request_command!([self.session; 1; false] => {
            In {
                is_privileged: u32 = is_privileged as u32
            };
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {};
            Out {};
            OutHandles {};
            OutObjects {};
            OutSessions {
                display_srv
            };
        });
        Ok(S::new(display_srv))
    }
}