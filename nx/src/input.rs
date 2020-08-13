extern crate alloc;
use alloc::vec::Vec;

use enumflags2::BitFlags;
use crate::result::*;
use crate::service::applet;
use crate::service::hid;
use crate::service::hid::IAppletResource;
use crate::service::hid::IHidServer;
use crate::svc;
use crate::mem;
use crate::vmem;
use crate::service;
use core::mem as cmem;

#[derive(BitFlags, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u64)]
pub enum Key {
    A = 0b1,
    B = 0b10,
    X = 0b100,
    Y = 0b1000,
    LStick = 0b10000,
    RStick = 0b100000,
    L = 0b1000000,
    R = 0b10000000,
    ZL = 0b100000000,
    ZR = 0b1000000000,
    Plus = 0b10000000000,
    Minus = 0b100000000000,
    Left = 0b1000000000000,
    Right = 0b10000000000000,
    Up = 0b100000000000000,
    Down = 0b1000000000000000,
    LStickLeft = 0b10000000000000000,
    LStickUp = 0b100000000000000000,
    LStickRight = 0b1000000000000000000,
    LStickDown = 0b10000000000000000000,
    RStickLeft = 0b100000000000000000000,
    RStickUp = 0b1000000000000000000000,
    RStickRight = 0b10000000000000000000000,
    RStickDown = 0b100000000000000000000000,
    SLLeft = 0b1000000000000000000000000,
    SRLeft = 0b10000000000000000000000000,
    SLRight = 0b100000000000000000000000000,
    SRRight = 0b1000000000000000000000000000,
    Touch = 0b10000000000000000000000000000,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TouchData {
    pub timestamp: u64,
    pub pad: u32,
    pub index: u32,
    pub x: u32,
    pub y: u32,
    pub diameter_x: u32,
    pub diameter_y: u32,
    pub angle: u32,
    pub pad_2: u32
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TouchEntry {
    pub timestamp: u64,
    pub count: u64,
    pub touches: [TouchData; 16],
    pub pad: u64
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TouchState {
    pub timestamp_ticks: u64,
    pub entry_count: u64,
    pub latest_index: u64,
    pub max_index: u64,
    pub timestamp: u64,
    pub entries: [TouchEntry; 17]
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct JoystickPosition {
    pub x: u32,
    pub y: u32
}

#[derive(BitFlags, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u64)]
pub enum ConnectionState {
    Connected = 0b1,
    Wired = 0b10,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ControllerStateEntry {
    pub timestamp: u64,
    pub timestamp_2: u64,
    pub button_state: u64,
    pub left_position: JoystickPosition,
    pub right_position: JoystickPosition,
    pub connection_state: BitFlags<ConnectionState>
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ControllerState {
    pub timestamp: u64,
    pub entry_count: u64,
    pub latest_index: u64,
    pub max_index: u64,
    pub entries: [ControllerStateEntry; 17]
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ControllerMacAddress {
    pub address: [u8; 0x10]
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ControllerColor {
    pub body: u32,
    pub buttons: u32
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ControllerData {
    pub status: u32,
    pub is_joycon_half: bool,
    pub pad: [u8; 3],
    pub color_descriptor_single: u32,
    pub color_single: ControllerColor,
    pub color_descriptor_split: u32,
    pub color_right: ControllerColor,
    pub color_left: ControllerColor,
    pub pro_controller_state: ControllerState,
    pub handheld_state: ControllerState,
    pub joined_state: ControllerState,
    pub left_state: ControllerState,
    pub right_state: ControllerState,
    pub main_no_analog_state: ControllerState,
    pub main_state: ControllerState,
    pub unk: [u8; 0x2A78],
    pub mac_addresses: [ControllerMacAddress; 2],
    pub unk_2: [u8; 0xE10]
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct SharedMemoryData {
    pub header: [u8; 0x400],
    pub touch_state: TouchState,
    pub pad: [u8; 0x3C0],
    pub mouse: [u8; 0x400],
    pub keyboard: [u8; 0x400],
    pub unk: [u8; 0x400],
    pub unk_2: [u8; 0x400],
    pub unk_3: [u8; 0x400],
    pub unk_4: [u8; 0x400],
    pub unk_5: [u8; 0x200],
    pub unk_6: [u8; 0x200],
    pub unk_7: [u8; 0x200],
    pub unk_8: [u8; 0x800],
    pub controller_serials: [u8; 0x4000],
    pub controllers: [ControllerData; 10],
    pub unk_9: [u8; 0x4600]
}

pub struct Player {
    controller: hid::ControllerId,
    data: *const ControllerData,
    prev_button_state: u64
}

impl Player {
    pub fn new(controller: hid::ControllerId, data: *const ControllerData) -> Self {
        Self { controller: controller, data: data, prev_button_state: 0 }
    }

    fn get_button_state(&self) -> u64 {
        let last_entry = unsafe { (*self.data).main_state.entries[(*self.data).main_state.latest_index as usize] };
        last_entry.button_state
    }

    pub fn get_button_state_held(&mut self) -> BitFlags<Key> {
        let button_state = self.get_button_state();
        self.prev_button_state = button_state;
        BitFlags::from_bits_truncate(button_state)
    }

    pub fn get_button_state_down(&mut self) -> BitFlags<Key> {
        let button_state = self.get_button_state();
        let down_state = (!self.prev_button_state) & button_state;
        self.prev_button_state = button_state;
        BitFlags::from_bits_truncate(down_state)
    }

    pub fn get_button_state_up(&mut self) -> BitFlags<Key> {
        let button_state = self.get_button_state();
        let up_state = self.prev_button_state & (!button_state);
        self.prev_button_state = button_state;
        BitFlags::from_bits_truncate(up_state)
    }

    pub fn get_controller(&self) -> hid::ControllerId {
        self.controller
    }
}

#[allow(dead_code)]
pub struct InputContext {
    hid_service: mem::SharedObject<hid::HidServer>,
    applet_resource: mem::SharedObject<hid::AppletResource>,
    shared_mem_handle: svc::Handle,
    aruid: applet::AppletResourceUserId,
    shared_mem_data: *const SharedMemoryData
}

macro_rules! set_all_controllers_mode_dual_impl {
    ([Result] $srv:expr, $aruid:expr, $( $id:expr ),*) => {
        $( $srv.borrow_mut().set_npad_joy_assignment_mode_dual($aruid, $id)?; )*
    };
    ([NoResult] $srv:expr, $aruid:expr, $( $id:expr ),*) => {
        $( let _ = $srv.borrow_mut().set_npad_joy_assignment_mode_dual($aruid, $id); )*
    };
}

#[allow(unreachable_patterns)]
fn get_index_for_controller(controller: hid::ControllerId) -> Result<usize> {
    match controller {
        hid::ControllerId::Player1 | hid::ControllerId::Player2 | hid::ControllerId::Player3 | hid::ControllerId::Player4 | hid::ControllerId::Player5 | hid::ControllerId::Player6 | hid::ControllerId::Player7 | hid::ControllerId::Player8 => Ok(controller as usize),
        hid::ControllerId::Handheld => Ok(8),
        _ => Err(ResultCode::new(0xBAAF))
    }
}

impl InputContext {
    pub fn new(aruid: applet::AppletResourceUserId, supported_tags: BitFlags<hid::NpadStyleTag>, controllers: Vec<hid::ControllerId>) -> Result<Self> {
        let hid_srv = service::new_shared_service_object::<hid::HidServer>()?;
        let applet_res: mem::SharedObject<hid::AppletResource> = hid_srv.borrow_mut().create_applet_resource(aruid)?;
        let shmem_handle = applet_res.borrow_mut().get_shared_memory_handle()?;
        let shmem_size = cmem::size_of::<SharedMemoryData>();
        let shmem_data = vmem::allocate(shmem_size)?;
        svc::map_shared_memory(shmem_handle, shmem_data, shmem_size, BitFlags::from(svc::MemoryPermission::Read))?;
        hid_srv.borrow_mut().activate_npad(aruid)?;
        hid_srv.borrow_mut().set_supported_npad_style_set(aruid, supported_tags)?;
        hid_srv.borrow_mut().set_supported_npad_id_type(aruid, controllers.as_ptr() as *const u8, controllers.len() * cmem::size_of::<hid::ControllerId>())?;
        hid_srv.borrow_mut().activate_npad(aruid)?;
        set_all_controllers_mode_dual_impl!([Result] hid_srv, aruid, hid::ControllerId::Player1, hid::ControllerId::Player2, hid::ControllerId::Player3, hid::ControllerId::Player4, hid::ControllerId::Player5, hid::ControllerId::Player6, hid::ControllerId::Player7, hid::ControllerId::Player8, hid::ControllerId::Handheld);
        Ok(Self { hid_service: hid_srv, applet_resource: applet_res, shared_mem_handle: shmem_handle, aruid: aruid, shared_mem_data: shmem_data as *const SharedMemoryData })
    }

    pub fn is_controller_connected(&mut self, controller: hid::ControllerId) -> bool {
        if let Ok(index) = get_index_for_controller(controller) {
            let controller_data = unsafe { &(*self.shared_mem_data).controllers[index] };
            let last_entry = controller_data.main_state.entries[controller_data.main_state.latest_index as usize];
            last_entry.connection_state.contains(ConnectionState::Connected)
        }
        else {
            false
        }
    }

    pub fn get_player(&mut self, controller: hid::ControllerId) -> Result<Player> {
        let index = get_index_for_controller(controller)?;
        let controller_data: *const ControllerData = unsafe { &(*self.shared_mem_data).controllers[index] };
        Ok(Player::new(controller, controller_data))
    }
}

impl Drop for InputContext {
    fn drop(&mut self) {
        set_all_controllers_mode_dual_impl!([NoResult] self.hid_service, self.aruid, hid::ControllerId::Player1, hid::ControllerId::Player2, hid::ControllerId::Player3, hid::ControllerId::Player4, hid::ControllerId::Player5, hid::ControllerId::Player6, hid::ControllerId::Player7, hid::ControllerId::Player8, hid::ControllerId::Handheld);
        let _ = self.hid_service.borrow_mut().deactivate_npad(self.aruid);
        let _ = svc::unmap_shared_memory(self.shared_mem_handle, self.shared_mem_data as *mut u8, cmem::size_of::<SharedMemoryData>());
        let _ = svc::close_handle(self.shared_mem_handle);
    }
}