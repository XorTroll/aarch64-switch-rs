use enumflags2::BitFlags;
use crate::result::*;
use crate::ipc;
use crate::service;
use crate::service::SessionObject;
use crate::service::applet;
use crate::svc;

#[derive(BitFlags, Copy, Clone, PartialEq, Debug)]
#[repr(u32)]
pub enum NpadStyleTag {
    ProController = 0b1,
    Handheld = 0b10,
    JoyconPair = 0b100,
    JoyconLeft = 0b1000,
    JoyconRight = 0b10000,
    SystemExt = 0b100000000000000000000000000000,
    System = 0b1000000000000000000000000000000,
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(i64)]
pub enum NpadJoyDeviceType {
    Left,
    Right
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(u32)]
pub enum ControllerId {
    Player1 = 0,
    Player2 = 1,
    Player3 = 2,
    Player4 = 3,
    Player5 = 4,
    Player6 = 5,
    Player7 = 6,
    Player8 = 7,
    Handheld = 0x20
}

pub trait IAppletResource {
    fn get_shared_memory_handle(&mut self) -> Result<svc::Handle>;
}

session_object_define!(AppletResource);

impl IAppletResource for AppletResource {
    fn get_shared_memory_handle(&mut self) -> Result<svc::Handle> {
        let handle: svc::Handle;
        ipc_client_session_send_request_command!([self.session; 0; false] => {
            In {};
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {};
            Out {};
            OutHandles {
                handle => ipc::HandleMode::Copy
            };
            OutObjects {};
            OutSessions {};
        });
        Ok(handle)
    }
}

pub trait IHidServer {
    fn create_applet_resource<S: service::SessionObject>(&mut self, aruid: applet::AppletResourceUserId) -> Result<S>;
    fn set_supported_npad_style_set(&mut self, aruid: applet::AppletResourceUserId, npad_style_tag: BitFlags<NpadStyleTag>) -> Result<()>;
    fn set_supported_npad_id_type(&mut self, aruid: applet::AppletResourceUserId, controllers: *const u8, controllers_size: usize) -> Result<()>;
    fn activate_npad(&mut self, aruid: applet::AppletResourceUserId) -> Result<()>;
    fn deactivate_npad(&mut self, aruid: applet::AppletResourceUserId) -> Result<()>;
    fn set_npad_joy_assignment_mode_single(&mut self, aruid: applet::AppletResourceUserId, controller: ControllerId, joy_type: NpadJoyDeviceType) -> Result<()>;
    fn set_npad_joy_assignment_mode_dual(&mut self, aruid: applet::AppletResourceUserId, controller: ControllerId) -> Result<()>;
}

session_object_define!(HidServer);

impl service::Service for HidServer {
    fn get_name() -> &'static str {
        nul!("hid")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}

impl IHidServer for HidServer {
    fn create_applet_resource<S: service::SessionObject>(&mut self, aruid: applet::AppletResourceUserId) -> Result<S> {
        let applet_resource: ipc::Session;
        ipc_client_session_send_request_command!([self.session; 0; true] => {
            In {
                aruid: applet::AppletResourceUserId = aruid
            };
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {};
            Out {};
            OutHandles {};
            OutObjects {};
            OutSessions {
                applet_resource
            };
        });
        Ok(S::new(applet_resource))
    }

    fn set_supported_npad_style_set(&mut self, aruid: applet::AppletResourceUserId, npad_style_tag: BitFlags<NpadStyleTag>) -> Result<()> {
        ipc_client_session_send_request_command!([self.session; 100; true] => {
            In {
                npad_style_tag: BitFlags<NpadStyleTag> = npad_style_tag,
                aruid: applet::AppletResourceUserId = aruid
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

    fn set_supported_npad_id_type(&mut self, aruid: u64, controllers: *const u8, controllers_size: usize) -> Result<()> {
        ipc_client_session_send_request_command!([self.session; 102; true] => {
            In {
                aruid: applet::AppletResourceUserId = aruid
            };
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {
                (controllers, controllers_size) => ipc::BufferAttribute::In | ipc::BufferAttribute::Pointer
            };
            Out {};
            OutHandles {};
            OutObjects {};
            OutSessions {};
        });
        Ok(())
    }

    fn activate_npad(&mut self, aruid: applet::AppletResourceUserId) -> Result<()> {
        ipc_client_session_send_request_command!([self.session; 103; true] => {
            In {
                aruid: applet::AppletResourceUserId = aruid
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

    fn deactivate_npad(&mut self, aruid: applet::AppletResourceUserId) -> Result<()> {
        ipc_client_session_send_request_command!([self.session; 104; true] => {
            In {
                aruid: applet::AppletResourceUserId = aruid
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

    fn set_npad_joy_assignment_mode_single(&mut self, aruid: applet::AppletResourceUserId, controller: ControllerId, joy_type: NpadJoyDeviceType) -> Result<()> {
        ipc_client_session_send_request_command!([self.session; 123; true] => {
            In {
                controller: ControllerId = controller,
                aruid: applet::AppletResourceUserId = aruid,
                joy_type: NpadJoyDeviceType = joy_type
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

    fn set_npad_joy_assignment_mode_dual(&mut self, aruid: applet::AppletResourceUserId, controller: ControllerId) -> Result<()> {
        ipc_client_session_send_request_command!([self.session; 124; true] => {
            In {
                controller: ControllerId = controller,
                aruid: applet::AppletResourceUserId = aruid
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