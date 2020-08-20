use crate::result::*;
use crate::ipc::sf;
use crate::mem;
use crate::util;
use crate::ipc::sf::applet;

#[derive(Copy, Clone)]
#[repr(C)]
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

bit_enum! {
    LayerFlags (u32) {
        None = 0,
        Default = bit!(0)
    }
}

pub type DisplayId = u64;

pub type LayerId = u64;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum DisplayServiceMode {
    User = 0,
    Privileged = 1
}

pub trait IManagerDisplayService {
    ipc_interface_define_command!(create_managed_layer: (flags: LayerFlags, display_id: DisplayId, aruid: applet::AppletResourceUserId) => (id: LayerId));
    ipc_interface_define_command!(destroy_managed_layer: (id: LayerId) => ());
}

pub trait ISystemDisplayService {
    ipc_interface_define_command!(get_z_order_count_min: (display_id: DisplayId) => (z: i64));
    ipc_interface_define_command!(get_z_order_count_max: (display_id: DisplayId) => (z: i64));
    ipc_interface_define_command!(set_layer_position: (x: f32, y: f32, id: LayerId) => ());
    ipc_interface_define_command!(set_layer_size: (id: LayerId, width: u64, height: u64) => ());
    ipc_interface_define_command!(set_layer_z: (id: LayerId, z: i64) => ());
    ipc_interface_define_command!(set_layer_visibility: (visible: bool, id: LayerId) => ());
}

pub trait IApplicationDisplayService {
    ipc_interface_define_command!(get_relay_service: () => (relay_service: mem::Shared<dyn sf::IObject>));
    ipc_interface_define_command!(get_system_display_service: () => (relay_service: mem::Shared<dyn sf::IObject>));
    ipc_interface_define_command!(get_manager_display_service: () => (relay_service: mem::Shared<dyn sf::IObject>));
    ipc_interface_define_command!(open_display: (name: DisplayName) => (id: DisplayId));
    ipc_interface_define_command!(close_display: (id: DisplayId) => ());
    ipc_interface_define_command!(open_layer: (name: DisplayName, id: LayerId, aruid: sf::ProcessId, out_native_window: sf::OutMapAliasBuffer) => (native_window_size: usize));
    ipc_interface_define_command!(create_stray_layer: (flags: LayerFlags, display_id: DisplayId, out_native_window: sf::OutMapAliasBuffer) => (id: LayerId, native_window_size: usize));
    ipc_interface_define_command!(destroy_stray_layer: (id: LayerId) => ());
    ipc_interface_define_command!(get_display_vsync_event: (id: DisplayId) => (event_handle: sf::CopyHandle));
}

pub trait IRootService {
    ipc_interface_define_command!(get_display_service: (mode: DisplayServiceMode) => (display_service: mem::Shared<dyn sf::IObject>));
}